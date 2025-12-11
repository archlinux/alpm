//! Package filename handling.

use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_parsers::iter_str_context;
use serde::{Deserialize, Serialize};
use strum::VariantNames;
use winnow::{
    ModalResult,
    Parser,
    ascii::alphanumeric1,
    combinator::{cut_err, eof, opt, peek, preceded, repeat},
    error::{AddContext, ContextError, ErrMode, ParserError, StrContext, StrContextValue},
    stream::Stream,
    token::take_until,
};

use crate::{
    Architecture,
    CompressionAlgorithmFileExtension,
    FileTypeIdentifier,
    FullVersion,
    Name,
    PackageError,
};

/// The full filename of a package.
///
/// A package filename tracks its [`Name`], [`FullVersion`], [`Architecture`] and the optional
/// [`CompressionAlgorithmFileExtension`].
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(into = "String")]
#[serde(try_from = "String")]
pub struct PackageFileName {
    pub(crate) name: Name,
    pub(crate) version: FullVersion,
    pub(crate) architecture: Architecture,
    pub(crate) compression: Option<CompressionAlgorithmFileExtension>,
}

impl PackageFileName {
    /// Creates a new [`PackageFileName`].
    ///
    /// # Errors
    ///
    /// Returns an error if the provided `version` does not have the `pkgrel` component.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::PackageFileName;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// assert_eq!(
    ///     "example-1:1.0.0-1-x86_64.pkg.tar.zst",
    ///     PackageFileName::new(
    ///         "example".parse()?,
    ///         "1:1.0.0-1".parse()?,
    ///         "x86_64".parse()?,
    ///         Some("zst".parse()?)
    ///     )
    ///     .to_string()
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        name: Name,
        version: FullVersion,
        architecture: Architecture,
        compression: Option<CompressionAlgorithmFileExtension>,
    ) -> Self {
        Self {
            name,
            version,
            architecture,
            compression,
        }
    }

    /// Returns a reference to the [`Name`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::{Name, PackageFileName};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let file_name = PackageFileName::new(
    ///     "example".parse()?,
    ///     "1:1.0.0-1".parse()?,
    ///     "x86_64".parse()?,
    ///     Some("zst".parse()?),
    /// );
    ///
    /// assert_eq!(file_name.name(), &Name::new("example")?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn name(&self) -> &Name {
        &self.name
    }

    /// Returns a reference to the [`FullVersion`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::{FullVersion, PackageFileName};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let file_name = PackageFileName::new(
    ///     "example".parse()?,
    ///     "1:1.0.0-1".parse()?,
    ///     "x86_64".parse()?,
    ///     Some("zst".parse()?),
    /// );
    ///
    /// assert_eq!(file_name.version(), &FullVersion::from_str("1:1.0.0-1")?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn version(&self) -> &FullVersion {
        &self.version
    }

    /// Returns the [`Architecture`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::{PackageFileName, SystemArchitecture};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let file_name = PackageFileName::new(
    ///     "example".parse()?,
    ///     "1:1.0.0-1".parse()?,
    ///     "x86_64".parse()?,
    ///     Some("zst".parse()?),
    /// );
    ///
    /// assert_eq!(file_name.architecture(), &SystemArchitecture::X86_64.into());
    /// # Ok(())
    /// # }
    /// ```
    pub fn architecture(&self) -> &Architecture {
        &self.architecture
    }

    /// Returns the optional [`CompressionAlgorithmFileExtension`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::{CompressionAlgorithmFileExtension, PackageFileName};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let file_name = PackageFileName::new(
    ///     "example".parse()?,
    ///     "1:1.0.0-1".parse()?,
    ///     "x86_64".parse()?,
    ///     Some("zst".parse()?),
    /// );
    ///
    /// assert_eq!(
    ///     file_name.compression(),
    ///     Some(CompressionAlgorithmFileExtension::Zstd)
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn compression(&self) -> Option<CompressionAlgorithmFileExtension> {
        self.compression
    }

    /// Returns the [`PackageFileName`] as [`PathBuf`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{path::PathBuf, str::FromStr};
    ///
    /// use alpm_types::PackageFileName;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let file_name = PackageFileName::new(
    ///     "example".parse()?,
    ///     "1:1.0.0-1".parse()?,
    ///     "x86_64".parse()?,
    ///     Some("zst".parse()?),
    /// );
    ///
    /// assert_eq!(
    ///     file_name.to_path_buf(),
    ///     PathBuf::from("example-1:1.0.0-1-x86_64.pkg.tar.zst")
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_path_buf(&self) -> PathBuf {
        self.to_string().into()
    }

    /// Sets the compression of the [`PackageFileName`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::{CompressionAlgorithmFileExtension, PackageFileName};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// // Create package file name with compression
    /// let mut file_name = PackageFileName::new(
    ///     "example".parse()?,
    ///     "1:1.0.0-1".parse()?,
    ///     "x86_64".parse()?,
    ///     Some("zst".parse()?),
    /// );
    /// // Remove the compression
    /// file_name.set_compression(None);
    ///
    /// assert!(file_name.compression().is_none());
    ///
    /// // Add other compression
    /// file_name.set_compression(Some(CompressionAlgorithmFileExtension::Gzip));
    ///
    /// assert!(
    ///     file_name
    ///         .compression()
    ///         .is_some_and(|compression| compression == CompressionAlgorithmFileExtension::Gzip)
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_compression(&mut self, compression: Option<CompressionAlgorithmFileExtension>) {
        self.compression = compression
    }

    /// Recognizes a [`PackageFileName`] in a string slice.
    ///
    /// Relies on [`winnow`] to parse `input` and recognize the [`Name`], [`FullVersion`],
    /// [`Architecture`] and [`CompressionAlgorithmFileExtension`] components.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - the [`Name`] component can not be recognized,
    /// - the [`FullVersion`] component can not be recognized,
    /// - the [`Architecture`] component can not be recognized,
    /// - or the [`CompressionAlgorithmFileExtension`] component can not be recognized.
    ///
    /// # Examples
    ///
    /// ```
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
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        // Detect the amount of dashes in input and subsequently in the Name component.
        //
        // Note: This is a necessary step because dashes are used as delimiters between the
        // components of the file name and the Name component (an alpm-package-name) can contain
        // dashes, too.
        // We know that the minimum amount of dashes in a valid alpm-package file name is
        // three (one dash between the Name, FullVersion, PackageRelease, and Architecture
        // component each).
        // We rely on this fact to determine the amount of dashes in the Name component and
        // thereby the cut-off point between the Name and the FullVersion component.
        let dashes: usize = input.chars().filter(|char| char == &'-').count();

        if dashes < 3 {
            let context_error = ContextError::from_input(input)
                .add_context(
                    input,
                    &input.checkpoint(),
                    StrContext::Label("alpm-package file name"),
                )
                .add_context(
                    input,
                    &input.checkpoint(),
                    StrContext::Expected(StrContextValue::Description(
                        concat!(
                        "a package name, followed by an alpm-package-version (full or full with epoch) and an architecture.",
                        "\nAll components must be delimited with a dash ('-')."
                        )
                    ))
                );

            return Err(ErrMode::Cut(context_error));
        }

        // The (zero or more) dashes in the Name component.
        let dashes_in_name = dashes.saturating_sub(3);

        // Advance the parser to the dash just behind the Name component, based on the amount of
        // dashes in the Name, e.g.:
        // "example-package-1:1.0.0-1-x86_64.pkg.tar.zst" -> "-1:1.0.0-1-x86_64.pkg.tar.zst"
        let name = cut_err(
            repeat::<_, _, (), _, _>(
                dashes_in_name + 1,
                // Advances to the next `-`.
                // If multiple `-` are present, the `-` that has been previously advanced to will
                // be consumed in the next itaration via the `opt("-")`. This enables us to go
                // **up to** the last `-`, while still consuming all `-` in between.
                (opt("-"), take_until(0.., "-"), peek("-")),
            )
            .take()
            // example-package
            .and_then(Name::parser),
        )
        .context(StrContext::Label("alpm-package-name"))
        .parse_next(input)?;

        // Consume leading dash in front of FullVersion, e.g.:
        // "-1:1.0.0-1-x86_64.pkg.tar.zst" -> "1:1.0.0-1-x86_64.pkg.tar.zst"
        "-".parse_next(input)?;

        // Advance the parser to beyond the FullVersion component (which contains one dash), e.g.:
        // "1:1.0.0-1-x86_64.pkg.tar.zst" -> "-x86_64.pkg.tar.zst"
        let version: FullVersion = cut_err((take_until(0.., "-"), "-", take_until(0.., "-")))
            .context(StrContext::Label("alpm-package-version"))
            .context(StrContext::Expected(StrContextValue::Description(
                "an alpm-package-version (full or full with epoch) followed by a `-` and an architecture",
            )))
            .take()
            .and_then(cut_err(FullVersion::parser))
            .parse_next(input)?;

        // Consume leading dash, e.g.:
        // "-x86_64.pkg.tar.zst" -> "x86_64.pkg.tar.zst"
        "-".parse_next(input)?;

        // Advance the parser to beyond the Architecture component, e.g.:
        // "x86_64.pkg.tar.zst" -> ".pkg.tar.zst"
        let architecture = take_until(0.., ".")
            .try_map(Architecture::from_str)
            .parse_next(input)?;

        // Consume leading dot, e.g.:
        // ".pkg.tar.zst" -> "pkg.tar.zst"
        ".".parse_next(input)?;

        // Consume the required alpm-package file type identifier, e.g.:
        // "pkg.tar.zst" -> ".tar.zst"
        take_until(0.., ".")
            .and_then(Into::<&str>::into(FileTypeIdentifier::BinaryPackage))
            .context(StrContext::Label("alpm-package file type identifier"))
            .context(StrContext::Expected(StrContextValue::StringLiteral(
                FileTypeIdentifier::BinaryPackage.into(),
            )))
            .parse_next(input)?;

        // Consume leading dot, e.g.:
        // ".tar.zst" -> "tar.zst"
        ".".parse_next(input)?;

        // Consume the required tar suffix, e.g.:
        // "tar.zst" -> ".zst"
        cut_err("tar")
            .context(StrContext::Label("tar suffix"))
            .context(StrContext::Expected(StrContextValue::Description("tar")))
            .parse_next(input)?;

        // Advance the parser to EOF for the CompressionAlgorithmFileExtension component, e.g.:
        // ".zst" -> ""
        // If input is "", we use no compression.
        let compression = opt(preceded(
            ".",
            cut_err(alphanumeric1.try_map(|s| {
                CompressionAlgorithmFileExtension::from_str(s).map_err(|_source| {
                    crate::Error::UnknownCompressionAlgorithmFileExtension {
                        value: s.to_string(),
                    }
                })
            }))
            .context(StrContext::Label("file extension for compression"))
            .context_with(iter_str_context!([
                CompressionAlgorithmFileExtension::VARIANTS
            ])),
        ))
        .parse_next(input)?;

        // Ensure that there are no trailing chars left.
        eof.context(StrContext::Expected(StrContextValue::Description(
            "end of package filename",
        )))
        .parse_next(input)?;

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
            "{}-{}-{}.{}.tar{}",
            self.name,
            self.version,
            self.architecture,
            FileTypeIdentifier::BinaryPackage,
            match self.compression {
                None => "".to_string(),
                Some(suffix) => format!(".{suffix}"),
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

#[cfg(test)]
mod test {
    use log::{LevelFilter, debug};
    use rstest::rstest;
    use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
    use testresult::TestResult;

    use super::*;
    use crate::system::SystemArchitecture;

    fn init_logger() -> TestResult {
        if TermLogger::init(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        )
        .is_err()
        {
            debug!("Not initializing another logger, as one is initialized already.");
        }

        Ok(())
    }

    /// Ensures that common and uncommon cases of package filenames can be created.
    #[rstest]
    #[case::name_with_dashes(Name::new("example-package")?, FullVersion::from_str("1.0.0-1")?, SystemArchitecture::X86_64.into(), Some(CompressionAlgorithmFileExtension::Zstd))]
    #[case::name_with_dashes_version_with_epoch_no_compression(Name::new("example-package")?, FullVersion::from_str("1:1.0.0-1")?, SystemArchitecture::X86_64.into(), None)]
    fn succeed_to_create_package_file_name(
        #[case] name: Name,
        #[case] version: FullVersion,
        #[case] architecture: Architecture,
        #[case] compression: Option<CompressionAlgorithmFileExtension>,
    ) -> TestResult {
        init_logger()?;

        let package_file_name =
            PackageFileName::new(name.clone(), version.clone(), architecture, compression);
        debug!("Package file name: {package_file_name}");

        Ok(())
    }

    /// Tests that common and uncommon cases of package file names can be recognized and
    /// round-tripped.
    #[rstest]
    #[case::name_with_dashes("example-pkg-1.0.0-1-x86_64.pkg.tar.zst")]
    #[case::no_compression("example-pkg-1.0.0-1-x86_64.pkg.tar")]
    #[case::version_as_name("1.0.0-1-1.0.0-1-x86_64.pkg.tar.zst")]
    #[case::version_with_epoch("example-1:1.0.0-1-x86_64.pkg.tar.zst")]
    #[case::version_with_pkgrel_sub_version("example-1.0.0-1.1-x86_64.pkg.tar.zst")]
    fn succeed_to_parse_package_file_name(#[case] s: &str) -> TestResult {
        init_logger()?;

        match PackageFileName::from_str(s) {
            Err(error) => {
                panic!("The parser failed parsing {s} although it should have succeeded:\n{error}");
            }
            Ok(value) => {
                let file_name_string: String = value.clone().into();
                assert_eq!(file_name_string, s);
                assert_eq!(value.to_string(), s);
            }
        };

        Ok(())
    }

    /// Ensures that [`PackageFileName`] can be created from common and uncommon cases of package
    /// file names as [`Path`].
    #[rstest]
    #[case::name_with_dashes("example-pkg-1.0.0-1-x86_64.pkg.tar.zst")]
    #[case::no_compression("example-pkg-1.0.0-1-x86_64.pkg.tar")]
    #[case::version_as_name("1.0.0-1-1.0.0-1-x86_64.pkg.tar.zst")]
    #[case::version_with_epoch("example-1:1.0.0-1-x86_64.pkg.tar.zst")]
    #[case::version_with_pkgrel_sub_version("example-1.0.0-1.1-x86_64.pkg.tar.zst")]
    fn package_file_name_from_path_succeeds(#[case] path: &str) -> TestResult {
        init_logger()?;
        let path = PathBuf::from(path);

        match PackageFileName::try_from(path.as_path()) {
            Err(error) => {
                panic!(
                    "Failed creating PackageFileName from {path:?} although it should have succeeded:\n{error}"
                );
            }
            Ok(value) => assert_eq!(value.to_path_buf(), path),
        };

        Ok(())
    }

    /// Tests that a matching [`Name`] can be derived from a [`PackageFileName`].
    #[test]
    fn package_file_name_name() -> TestResult {
        let name = Name::new("example")?;
        let file_name = PackageFileName::new(
            name.clone(),
            "1:1.0.0-1".parse()?,
            "x86_64".parse()?,
            Some("zst".parse()?),
        );

        assert_eq!(file_name.name(), &name);

        Ok(())
    }

    /// Tests that a matching [`FullVersion`] can be derived from a [`PackageFileName`].
    #[test]
    fn package_file_name_version() -> TestResult {
        let version = FullVersion::from_str("1:1.0.0-1")?;
        let file_name = PackageFileName::new(
            Name::new("example")?,
            version.clone(),
            "x86_64".parse()?,
            Some("zst".parse()?),
        );

        assert_eq!(file_name.version(), &version);

        Ok(())
    }

    /// Tests that a matching [`Architecture`] can be derived from a [`PackageFileName`].
    #[test]
    fn package_file_name_architecture() -> TestResult {
        let architecture: Architecture = SystemArchitecture::X86_64.into();
        let file_name = PackageFileName::new(
            Name::new("example")?,
            "1:1.0.0-1".parse()?,
            architecture.clone(),
            Some("zst".parse()?),
        );

        assert_eq!(file_name.architecture(), &architecture);

        Ok(())
    }

    /// Tests that a matching optional [`CompressionAlgorithmFileExtension`] can be derived from a
    /// [`PackageFileName`].
    #[rstest]
    #[case::with_compression(Some(CompressionAlgorithmFileExtension::Zstd))]
    #[case::no_compression(None)]
    fn package_file_name_compression(
        #[case] compression: Option<CompressionAlgorithmFileExtension>,
    ) -> TestResult {
        let file_name = PackageFileName::new(
            Name::new("example")?,
            "1:1.0.0-1".parse()?,
            "x86_64".parse()?,
            compression,
        );

        assert_eq!(file_name.compression(), compression);

        Ok(())
    }

    /// Tests that a [`PathBuf`] can be derived from a [`PackageFileName`].
    #[rstest]
    #[case::with_compression(Some("zst".parse()?), "example-1:1.0.0-1-x86_64.pkg.tar.zst")]
    #[case::no_compression(None, "example-1:1.0.0-1-x86_64.pkg.tar")]
    fn package_file_name_to_path_buf(
        #[case] compression: Option<CompressionAlgorithmFileExtension>,
        #[case] path: &str,
    ) -> TestResult {
        let file_name = PackageFileName::new(
            "example".parse()?,
            "1:1.0.0-1".parse()?,
            "x86_64".parse()?,
            compression,
        );
        assert_eq!(file_name.to_path_buf(), PathBuf::from(path));

        Ok(())
    }

    /// Tests that an uncompressed [`PackageFileName`] representation can be derived from a
    /// [`PackageFileName`].
    #[rstest]
    #[case::compression_to_no_compression(
        Some(CompressionAlgorithmFileExtension::Zstd),
        None,
        PackageFileName::new(
            "example".parse()?,
            "1:1.0.0-1".parse()?,
            "x86_64".parse()?,
            None,
        ))]
    #[case::no_compression_to_compression(
        None,
        Some(CompressionAlgorithmFileExtension::Zstd),
        PackageFileName::new(
            "example".parse()?,
            "1:1.0.0-1".parse()?,
            "x86_64".parse()?,
            Some(CompressionAlgorithmFileExtension::Zstd),
        ))]
    fn package_file_name_set_compression(
        #[case] initial_compression: Option<CompressionAlgorithmFileExtension>,
        #[case] compression: Option<CompressionAlgorithmFileExtension>,
        #[case] output_file_name: PackageFileName,
    ) -> TestResult {
        let mut file_name = PackageFileName::new(
            "example".parse()?,
            "1:1.0.0-1".parse()?,
            "x86_64".parse()?,
            initial_compression,
        );
        file_name.set_compression(compression);
        assert_eq!(file_name, output_file_name);

        Ok(())
    }
}
