//! Package filename handling.

use std::{fmt::Display, path::PathBuf, str::FromStr};

use alpm_types::{Architecture, Name, Version};
use log::debug;
use winnow::{
    ModalResult,
    Parser,
    ascii::alphanumeric1,
    combinator::{cut_err, eof, opt, preceded, repeat, repeat_till, terminated},
    error::{StrContext, StrContextValue},
    token::{any, take_until},
};

use crate::{compression::PackageCompression, pipeline::PackagePipeline};

/// An error that can occur when dealing with filenames of alpm-packages.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A path can not be used as a [`PackageFileName`].
    #[error("The path {path} is not a valid alpm-package filename")]
    InvalidPath {
        /// The file path that is not valid.
        path: PathBuf,
    },

    /// A string is not a valid [`PackageFileName`].
    #[error("The string {string} is not a valid alpm-package filename")]
    InvalidString {
        /// The file path that is not valid.
        string: String,
    },
}

/// The full filename of a package.
///
/// A package filename tracks its [`Name`], [`Version`], [`Architecture`] and the file specific
/// [`PackageCompression`].
#[derive(Clone, Debug)]
pub struct PackageFileName {
    name: Name,
    version: Version,
    architecture: Architecture,
    compression: PackageCompression,
}

impl PackageFileName {
    /// Creates a new [`PackageFileName`].
    pub fn new(
        name: Name,
        version: Version,
        architecture: Architecture,
        compression: PackageCompression,
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

    /// Returns a reference to the [`PackageCompression`].
    pub fn compression(&self) -> &PackageCompression {
        &self.compression
    }

    /// Returns the [`PackageFileName`] as [`PathBuf`].
    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from(self.to_string())
    }

    /// Recognizes a [`PackageFileName`] in a string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_package::filename::PackageFileName;
    /// use winnow::Parser;
    ///
    /// # fn main() -> testresult::TestResult {
    /// let filename = "example-package-1:1.0.0-1-x86_64.pkg.tar.zst";
    /// assert_eq!(
    ///     filename,
    ///     PackageFileName::parser.parse(filename)?.to_string()
    /// );
    /// let filename = "example-1.0.0-1-x86_64.pkg.tar.zst";
    /// assert_eq!(
    ///     filename,
    ///     PackageFileName::parser.parse(filename)?.to_string()
    /// );
    /// let filename = "example-alphaversion-1-x86_64.pkg.tar.zst";
    /// assert_eq!(
    ///     filename,
    ///     PackageFileName::parser.parse(filename)?.to_string()
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn parser(value: &mut &str) -> ModalResult<Self> {
        debug!("Parsing PackageFileName from {value}");
        // the amount of dashes in value
        let dashes = value
            .chars()
            .fold(0, |acc, char| if char == '-' { acc + 1 } else { acc });
        debug!("dashes in {value}: {dashes}");
        let dashes_in_name = (dashes - 3).max(0);
        debug!("dashes in Name: {dashes_in_name}");

        // example-package-1:1.0.0-1-x86_64.pkg.tar.zst
        // -> 1:1.0.0-1-x86_64.pkg.tar.zst
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

        // 1:1.0.0-1-x86_64.pkg.tar.zst
        // -> -x86_64.pkg.tar.zst
        let version: Version = cut_err(
            (take_until(0.., "-"), "-", take_until(0.., "-"))
                .take()
                // 1:1.0.0-1
                .and_then(Version::parser),
        )
        .context(StrContext::Label("alpm-package-version"))
        .parse_next(value)?;

        // consume trailing dash
        "-".parse_next(value)?;

        // x86_64.pkg.tar.zst
        // -> .pkg.tar.zst
        let architecture = take_until(0.., ".")
            .try_map(Architecture::from_str)
            .parse_next(value)?;

        // .pkg.tar.zst
        // -> .zst
        cut_err(".pkg.tar")
            .context(StrContext::Label("ALPM package marker"))
            .context(StrContext::Expected(StrContextValue::Description(
                ".pkg.tar",
            )))
            .parse_next(value)?;

        // ".zst" || ""
        let compression = opt(preceded(
            ".",
            cut_err(terminated(alphanumeric1, eof))
                .context(StrContext::Label("file extension for compression"))
                // check if StrContextValue::StringLiteral can be used for programmatically adding
                // extensions
                .context(StrContext::Expected(StrContextValue::Description(
                    "bz2, gz, xz, zst",
                ))),
        )
        .try_map(PackageCompression::from_str))
        .parse_next(value)?
        .unwrap_or(PackageCompression::None);

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
                PackageCompression::None => self.compression.to_string(),
                _ => format!(".{}", self.compression),
            }
        )
    }
}

impl From<&PackagePipeline> for PackageFileName {
    /// Creates a [`PackageFileName`] from a reference to a [`PackagePipeline`].
    fn from(value: &PackagePipeline) -> Self {
        Self {
            name: match value.package_input.package_info() {
                alpm_pkginfo::PackageInfo::V1(package_info) => package_info.pkgname().clone(),
                alpm_pkginfo::PackageInfo::V2(package_info) => package_info.pkgname().clone(),
            },
            version: match value.package_input.package_info() {
                alpm_pkginfo::PackageInfo::V1(package_info) => package_info.pkgver().clone(),
                alpm_pkginfo::PackageInfo::V2(package_info) => package_info.pkgver().clone(),
            },
            architecture: match value.package_input.package_info() {
                alpm_pkginfo::PackageInfo::V1(package_info) => *package_info.arch(),
                alpm_pkginfo::PackageInfo::V2(package_info) => *package_info.arch(),
            },
            compression: value.compression.clone(),
        }
    }
}

impl FromStr for PackageFileName {
    type Err = crate::Error;

    /// Creates a [`PackageFileName`] from string slice.
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
    /// use alpm_package::filename::PackageFileName;
    ///
    /// # fn main() -> testresult::TestResult {
    /// let filename = "example-package-1:1.0.0-1-x86_64.pkg.tar.zst";
    /// assert_eq!(filename, PackageFileName::from_str(filename)?.to_string());
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}
