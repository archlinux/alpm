use thiserror::Error;

/// The Error that can occur when using types
#[derive(Debug, Error, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// A default error
    #[error("Type error: {0}")]
    Default(String),

    /// An invalid absolute path
    ///
    /// This error occurs when a path is not absolute (i.e. does not start with a `/`)
    #[error("Invalid absolute path: {0}")]
    InvalidAbsolutePath(String),

    /// An invalid CPU architecture
    ///
    /// This error occurs when a CPU architecture is not one of the supported variants
    /// See [`crate::system::Architecture`] for supported architectures.
    ///
    /// NOTE: The underlying error type is `strum::ParseError`
    #[error("Invalid architecture: {0}")]
    InvalidArchitecture(String),

    /// An invalid build date (in seconds since the epoch)
    ///
    /// This error occurs while parsing a string as a epoch timestamp
    ///
    /// NOTE: The underlying error type is `std::num::ParseIntError`
    #[error("Invalid build date: {0}")]
    InvalidBuildDate(String),

    /// An invalid build directory
    ///
    /// This error occurs when a build directory is not absolute.
    ///
    /// NOTE: See InvalidAbsolutePath variant
    #[error("Invalid build directory: {0}")]
    InvalidBuildDir(String),

    /// An invalid build option
    ///
    /// This error occurs when a build option contains one of the
    /// invalid characters (`-`, `.`, `_`)
    #[error("Invalid option string: {0}")]
    InvalidBuildOption(String),

    /// An invalid build environment
    ///
    /// This error occurs when a build option is not valid.
    ///
    /// NOTE: See InvalidBuildOption variant
    #[error("Invalid build environment string: {0}")]
    InvalidBuildEnv(String),

    /// An invalid BuildTool
    ///
    /// This error occurs when a build tool contains invalid characters.
    ///
    /// NOTE: See NameError variant
    #[error("Invalid buildtool: {0}")]
    InvalidBuildTool(String),

    /// An invalid BuildToolVer
    ///
    /// This error occurs when a build tool version does not contain a '-'
    #[error("Invalid buildtool version: {0}")]
    InvalidBuildToolVer(String),

    /// An invalid checksum
    ///
    /// This error occurs when a checksum has an invalid length or contains invalid characters
    #[error("Invalid checksum: {0}")]
    InvalidChecksum(String),

    /// An invalid compressed file size (in bytes)
    ///
    /// This error occurs while parsing a string as size (number)
    ///
    /// NOTE: The underlying error type is `std::num::ParseIntError`
    #[error("Invalid compressed size: {0}")]
    InvalidCompressedSize(String),

    /// An invalid epoch in a version string
    ///
    /// This error occurs while parsing a string as a epoch timestamp
    ///
    /// NOTE: The underlying error type is `std::num::ParseIntError`
    #[error("Invalid epoch in string: {0}")]
    InvalidEpoch(String),

    /// An invalid installed package information
    ///
    /// This error occurs when the installed package has one of the components missing.
    #[error("Invalid information on installed package: {0}")]
    InvalidInstalled(String),

    /// An invalid installed package size (in bytes)
    ///
    /// This error occurs while parsing a string as size (number)
    ///
    /// NOTE: The underlying error type is `std::num::ParseIntError`
    #[error("Invalid installed size: {0}")]
    InvalidInstalledSize(String),

    /// An invalid package name
    ///
    /// This error occurs when a package name contains invalid characters (checked via regex)
    #[error("Invalid package name: {0}")]
    InvalidName(String),

    #[error("Invalid md5sum: {0}")]
    #[deprecated(
        since = "0.3.0",
        note = "Error::InvalidMd5Sum(String) is tied to the use of Md5Sum. Users should use Checksum<Md5> and Error::InvalidChecksum(String) instead."
    )]
    InvalidMd5Sum(String),

    /// An invalid packager value
    ///
    /// This error occurs when either a packager value contains invalid characters or missing a
    /// component
    #[error("Invalid packager string: {0}")]
    InvalidPackager(String),

    /// An invalid packager e-mail
    ///
    /// This error occurs when parsing the email address.
    ///
    /// NOTE: The underlying error type is `email_address::Error`
    #[error("Invalid packager e-mail: {0}")]
    InvalidPackagerEmail(String),

    /// An invalid package option
    ///
    /// This error occurs when a build option contains one of the
    /// invalid characters (`-`, `.`, `_`)
    ///
    /// NOTE: See InvalidBuildOption variant
    #[error("Invalid package option: {0}")]
    InvalidPackageOption(String),

    /// An invalid pkgrel in a version string
    ///
    /// This error occurs when a pkgrel contains invalid characters (checked via regex)
    #[error("Invalid pkgrel in string: {0}")]
    InvalidPkgrel(String),

    /// An invalid pkgver in a version string
    ///
    /// This error occurs when a pkgver contains invalid characters (checked via regex)
    #[error("Invalid pkgver in string: {0}")]
    InvalidPkgver(String),

    /// An invalid start directory
    ///
    /// This error occurs when a start directory is not absolute.
    ///
    /// NOTE: See InvalidAbsolutePath variant
    #[error("Invalid start directory: {0}")]
    InvalidStartDir(String),

    /// An invalid version string
    ///
    /// This error occurs when:
    /// - cannot be parsed from String (ParseIntError)
    /// - cannot be parsed via `semver` (semver::Error)
    #[error("Invalid version string: {0}")]
    InvalidVersion(String),

    /// An invalid version comparison
    ///
    /// This error occurs when a version comparison operator is not recognized
    #[error("Invalid version comparison: {0}")]
    InvalidVersionComparison(String),

    /// An invalid version requirement
    ///
    /// This error occurs when a version requirement character is not recognized
    #[error("Invalid version requirement: {0}")]
    InvalidVersionRequirement(String),

    /// An invalid URL
    ///
    /// This error occurs when a URL cannot be parsed
    ///
    /// NOTE: The underlying error type is `url::ParseError`
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// An invalid file name
    ///
    /// This error occurs when a file name contains path separators or null characters
    #[error("Invalid file name: {0}")]
    InvalidFilename(String),
}

impl From<strum::ParseError> for Error {
    fn from(err: strum::ParseError) -> Self {
        Error::Default(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("Invalid build date: -1", Error::InvalidBuildDate(String::from("-1")))]
    #[case(
        "Invalid compressed size: -1",
        Error::InvalidCompressedSize(String::from("-1"))
    )]
    #[case(
        "Invalid installed size: -1",
        Error::InvalidInstalledSize(String::from("-1"))
    )]
    #[case("Invalid package name: -1", Error::InvalidName(String::from("-1")))]
    #[allow(deprecated)]
    #[case("Invalid md5sum: -1", Error::InvalidMd5Sum(String::from("-1")))]
    #[case(
        "Invalid packager string: foo",
        Error::InvalidPackager(String::from("foo"))
    )]
    #[case(
        "Invalid packager e-mail: foo",
        Error::InvalidPackagerEmail(String::from("foo"))
    )]
    #[case(
        "Invalid version string: -1",
        Error::InvalidVersion(String::from("-1"))
    )]
    fn error_format_string(#[case] error_str: &str, #[case] error: Error) {
        assert_eq!(error_str, format!("{}", error));
    }
}
