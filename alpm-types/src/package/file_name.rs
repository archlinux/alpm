//! Package filename handling.

use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

use log::debug;
use winnow::{
    ModalResult,
    Parser,
    ascii::alphanumeric1,
    combinator::{cut_err, eof, opt, preceded, repeat, repeat_till, terminated},
    error::{StrContext, StrContextValue},
    token::{any, take_until},
};

use crate::{Architecture, CompressionType, Name, PackageError, Version};

/// The full filename of a package.
///
/// A package filename tracks its [`Name`], [`Version`], [`Architecture`] and the file specific
/// [`CompressionType`].
#[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq, serde::Serialize)]
#[serde(into = "String")]
#[serde(try_from = "String")]
pub struct PackageFileName {
    name: Name,
    version: Version,
    architecture: Architecture,
    compression: CompressionType,
}

impl PackageFileName {
    /// Creates a new [`PackageFileName`].
    pub fn new(
        name: Name,
        version: Version,
        architecture: Architecture,
        compression: CompressionType,
    ) -> Self {
        Self {
            name,
            version,
            architecture,
            compression,
        }
    }

    /// Returns a reference to the [`Name`].
    pub fn name(&self) -> &Name {
        &self.name
    }

    /// Returns a reference to the [`Version`].
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// Returns the [`Architecture`].
    pub fn architecture(&self) -> Architecture {
        self.architecture
    }

    /// Returns a reference to the [`CompressionType`].
    pub fn compression(&self) -> &CompressionType {
        &self.compression
    }

    /// Returns the [`PackageFileName`] as [`PathBuf`].
    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from(self.to_string())
    }

    /// Returns the uncompressed representation of the [`PackageFileName`].
    pub fn to_uncompressed(&self) -> PackageFileName {
        match self.compression {
            CompressionType::None => self.clone(),
            _ => PackageFileName {
                name: self.name.clone(),
                version: self.version.clone(),
                architecture: self.architecture,
                compression: CompressionType::None,
            },
        }
    }

    /// Recognizes a [`PackageFileName`] in a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - the [`Name`] component can not be recognized,
    /// - the [`Version`] component can not be recognized,
    /// - the [`Architecture`] component can not be recognized,
    /// - or the [`CompressionType`] component can not be recognized.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::PackageFileName;
    /// use winnow::Parser;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let filename = "example-package-1:1.0.0-1-x86_64.pkg.tar.zst";
    /// assert_eq!(
    ///     filename,
    ///     PackageFileName::parser.parse(filename)?.to_string()
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn parser(value: &mut &str) -> ModalResult<Self> {
        debug!("Recognizing PackageFileName in {value}.");
        // Detect the amount of dashes in value and subsequently in the Name component.
        // As we know that the minimum amount of dashes in a package filename is three, we can use
        // this to figure out how far we have to advance the parser to properly detect the Name
        // component.
        let dashes: usize = value
            .chars()
            .fold(0, |acc, char| if char == '-' { acc + 1 } else { acc });
        // The (zero or more) dashes in the Name component.
        let dashes_in_name = dashes.saturating_sub(3);

        // Advance the parser to a dash beyond the Name component, based on the amount of dashes in
        // that component, e.g.:
        // "example-package-1:1.0.0-1-x86_64.pkg.tar.zst" -> "1:1.0.0-1-x86_64.pkg.tar.zst"
        let name = cut_err(
            repeat::<_, _, (), _, _>(
                dashes_in_name + 1,
                repeat_till::<_, _, (), _, _, _, _>(0.., any, "-"),
            )
            .take()
            // example-package-
            .and_then(
                repeat_till(0.., any, ("-", eof))
                    .try_map(|(name, _match): (String, (&str, &str))| Name::from_str(&name)),
            ),
        )
        .context(StrContext::Label("alpm-package-name"))
        .parse_next(value)?;
        debug!("Detected Name: {name}");

        // Advance the parser to beyond the Version component (which contains one dash), e.g.:
        // "1:1.0.0-1-x86_64.pkg.tar.zst" -> "-x86_64.pkg.tar.zst"
        let version: Version = cut_err(
            (take_until(0.., "-"), "-", take_until(0.., "-"))
                .take()
                .and_then(Version::parser),
        )
        .context(StrContext::Label("alpm-package-version"))
        .parse_next(value)?;
        debug!("Detected Version: {version}");

        // Consume leading dash, e.g.:
        // "-x86_64.pkg.tar.zst" -> "x86_64.pkg.tar.zst"
        "-".parse_next(value)?;

        // Advance the parser to beyond the Architecture component, e.g.:
        // "x86_64.pkg.tar.zst" -> ".pkg.tar.zst"
        let architecture = take_until(0.., ".")
            .try_map(Architecture::from_str)
            .parse_next(value)?;
        debug!("Detected Architecture: {architecture}");

        // Consume the required package marker plus tar component, e.g.:
        // ".pkg.tar.zst" -> ".zst"
        cut_err(".pkg.tar")
            .context(StrContext::Label("ALPM package marker"))
            .context(StrContext::Expected(StrContextValue::Description(
                ".pkg.tar",
            )))
            .parse_next(value)?;

        // Advance the parser to EOF for the CompressionType component, e.g.:
        // ".zst" -> ""
        let compression = opt(preceded(
            ".",
            cut_err(terminated(alphanumeric1, eof))
                .context(StrContext::Label("file extension for compression"))
                .context(StrContext::Expected(StrContextValue::StringLiteral("bz2")))
                .context(StrContext::Expected(StrContextValue::StringLiteral("gz")))
                .context(StrContext::Expected(StrContextValue::StringLiteral("xz")))
                .context(StrContext::Expected(StrContextValue::StringLiteral("zst"))),
        )
        .try_map(CompressionType::from_str))
        .parse_next(value)?
        // If value is "", we use no compression.
        .unwrap_or(CompressionType::None);
        debug!("Detected CompressionType: {compression}");

        // Ensure that there are no trailing chars left.
        eof.context(StrContext::Expected(StrContextValue::Description(
            "end of package filename",
        )))
        .parse_next(value)?;

        Ok(Self {
            name,
            version,
            architecture,
            compression,
        })
    }
}

impl Display for PackageFileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}-{}.pkg.tar{}",
            self.name,
            self.version,
            self.architecture,
            match self.compression {
                CompressionType::None => self.compression.to_string(),
                _ => format!(".{}", self.compression),
            }
        )
    }
}

impl From<PackageFileName> for String {
    /// Creates a [`String`] from a [`PackageFileName`].
    fn from(value: PackageFileName) -> Self {
        value.to_string()
    }
}

impl FromStr for PackageFileName {
    type Err = crate::Error;

    /// Creates a [`PackageFileName`] from a string slice.
    ///
    /// Delegates to [`PackageFileName::parser`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`PackageFileName::parser`] fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::PackageFileName;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let filename = "example-package-1:1.0.0-1-x86_64.pkg.tar.zst";
    /// assert_eq!(filename, PackageFileName::from_str(filename)?.to_string());
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

impl TryFrom<&Path> for PackageFileName {
    type Error = crate::Error;

    /// Creates a [`PackageFileName`] from a [`Path`] reference.
    ///
    /// The file name in `value` is extracted and, if valid is turned into a string slice.
    /// The creation of the [`PackageFileName`] is delegated to [`PackageFileName::parser`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - `value` does not contain a valid file name,
    /// - `value` can not be turned into a string slice,
    /// - or [`PackageFileName::parser`] fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// use alpm_types::PackageFileName;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let filename = PathBuf::from("../example-package-1:1.0.0-1-x86_64.pkg.tar.zst");
    /// assert_eq!(
    ///     filename,
    ///     PathBuf::from("..").join(PackageFileName::try_from(filename.as_path())?.to_path_buf()),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let Some(name) = value.file_name() else {
            return Err(PackageError::InvalidPackageFileNamePath {
                path: value.to_path_buf(),
            }
            .into());
        };
        let Some(s) = name.to_str() else {
            return Err(PackageError::InvalidPackageFileNamePath {
                path: value.to_path_buf(),
            }
            .into());
        };
        Ok(Self::parser.parse(s)?)
    }
}

impl TryFrom<String> for PackageFileName {
    type Error = crate::Error;

    /// Creates a [`PackageFileName`] from a String.
    ///
    /// Delegates to [`PackageFileName::parser`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`PackageFileName::parser`] fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::PackageFileName;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let filename = "example-package-1:1.0.0-1-x86_64.pkg.tar.zst".to_string();
    /// assert_eq!(
    ///     filename.clone(),
    ///     PackageFileName::try_from(filename)?.to_string()
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self::parser.parse(&value)?)
    }
}
