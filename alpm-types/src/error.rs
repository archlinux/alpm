use std::path::PathBuf;

use crate::Architecture;

/// The library's error type
///
/// These errors are usually parsing errors and they each contain a context
/// about why the error has occurred and the value that caused the error.
///
/// The original error is also included in the variants that have the `source` field.
/// You can access it using the `source()` method.
/// See [Error::source](https://doc.rust-lang.org/std/error/trait.Error.html#method.source) for
/// more information.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    /// Combination of architectures that is invalid.
    #[error("The architecture combination is invalid: {architectures:?}. {context}")]
    InvalidArchitectures {
        /// The invalid architectures combination.
        architectures: Vec<Architecture>,
        /// The reason why the architectures are invalid.
        context: &'static str,
    },

    /// An invalid integer
    #[error("Invalid integer (caused by {kind:?})")]
    InvalidInteger {
        /// The reason for the invalid integer.
        kind: std::num::IntErrorKind,
    },

    /// An invalid enum variant
    #[error("Invalid variant ({0})")]
    InvalidVariant(#[from] strum::ParseError),

    /// An invalid email address
    #[error("Invalid e-mail ({0})")]
    InvalidEmail(#[from] email_address::Error),

    /// An invalid URL
    #[error("Invalid URL ({0})")]
    InvalidUrl(#[from] url::ParseError),

    /// An invalid license
    #[error("Invalid license ({0})")]
    InvalidLicense(#[from] spdx::ParseError),

    /// An invalid semantic version string
    ///
    /// This error occurs when a semantic version cannot be parsed from a string
    /// We cannot use `#[source] semver::Error` here because it does not implement `PartialEq`.
    /// See: <https://github.com/dtolnay/semver/issues/326>
    ///
    /// TODO: Use the error source when the issue above is resolved.
    #[error("Invalid semver ({kind})")]
    InvalidSemver {
        /// The reason for the invalid semantic version.
        kind: String,
    },

    /// Value contains an invalid character
    #[error("Value contains invalid characters: {invalid_char:?}")]
    ValueContainsInvalidChars {
        /// The invalid character
        invalid_char: char,
    },

    /// Value length is incorrect
    #[error("Incorrect length, got {length} expected {expected}")]
    IncorrectLength {
        /// The incorrect length.
        length: usize,
        /// The expected length.
        expected: usize,
    },

    /// Value is missing a delimiter character
    #[error("Value is missing the required delimiter: {delimiter}")]
    DelimiterNotFound {
        /// The required delimiter.
        delimiter: char,
    },

    /// Value does not match the restrictions
    #[error("Does not match the restrictions ({restrictions:?})")]
    ValueDoesNotMatchRestrictions {
        /// The list of restrictions that cannot be met.
        restrictions: Vec<String>,
    },

    /// A validation regex does not match the value
    #[error("Value '{value}' does not match the '{regex_type}' regex: {regex}")]
    RegexDoesNotMatch {
        /// The value that does not match.
        value: String,
        /// The type of regular expression applied to the `value`.
        regex_type: String,
        /// The regular expression applied to the `value`.
        regex: String,
    },

    /// A winnow parser for a type didn't work and produced an error.
    #[error("Parser failed with the following error:\n{0}")]
    ParseError(String),

    /// Missing field in a value
    #[error("Missing component: {component}")]
    MissingComponent {
        /// The component that is missing.
        component: &'static str,
    },

    /// An invalid absolute path (i.e. does not start with a `/`)
    #[error("The path is not absolute: {0}")]
    PathNotAbsolute(PathBuf),

    /// An invalid relative path (i.e. starts with a `/`)
    #[error("The path is not relative: {0}")]
    PathNotRelative(PathBuf),

    /// File name contains invalid characters
    #[error("File name ({0}) contains invalid characters: {1:?}")]
    FileNameContainsInvalidChars(PathBuf, char),

    /// File name is empty
    #[error("File name is empty")]
    FileNameIsEmpty,

    /// A deprecated license
    #[error("Deprecated license: {0}")]
    DeprecatedLicense(String),

    /// A component is invalid and cannot be used.
    #[error("Invalid component {component} encountered while {context}")]
    InvalidComponent {
        /// The invalid component
        component: &'static str,
        /// The context in which the error occurs.
        ///
        /// This is meant to complete the sentence "Invalid component {component} encountered
        /// while ".
        context: &'static str,
    },

    /// An invalid OpenPGP v4 fingerprint
    #[error("Invalid OpenPGP v4 fingerprint, only 40 uppercase hexadecimal characters are allowed")]
    InvalidOpenPGPv4Fingerprint,

    /// An invalid OpenPGP key ID
    #[error("The string is not a valid OpenPGP key ID: {0}, must be 16 hexadecimal characters")]
    InvalidOpenPGPKeyId(String),

    /// An invalid shared object name (v1)
    #[error("Invalid shared object name (v1): {0}")]
    InvalidSonameV1(&'static str),

    /// A package data error.
    #[error("Package error: {0}")]
    Package(#[from] crate::PackageError),

    /// A string represents an unknown compression algorithm file extension.
    #[error("Unknown compression algorithm file extension: {value:?}")]
    UnknownCompressionAlgorithmFileExtension {
        /// A String representing an unknown compression algorithm file extension.
        value: String,
    },

    /// A string represents an unknown file type identifier.
    #[error("Unknown file type identifier: {value:?}")]
    UnknownFileTypeIdentifier {
        /// A String representing an unknown file type identifier.
        value: String,
    },
}

impl From<std::num::ParseIntError> for crate::error::Error {
    /// Converts a [`std::num::ParseIntError`] into an [`Error::InvalidInteger`].
    fn from(e: std::num::ParseIntError) -> Self {
        Self::InvalidInteger { kind: *e.kind() }
    }
}

impl<'a> From<winnow::error::ParseError<&'a str, winnow::error::ContextError>>
    for crate::error::Error
{
    /// Converts a [`winnow::error::ParseError`] into an [`Error::ParseError`].
    fn from(value: winnow::error::ParseError<&'a str, winnow::error::ContextError>) -> Self {
        Self::ParseError(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::num::IntErrorKind;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(
        "Invalid integer (caused by InvalidDigit)",
        Error::InvalidInteger {
            kind: IntErrorKind::InvalidDigit
        }
    )]
    #[case(
        "Invalid integer (caused by InvalidDigit)",
        Error::InvalidInteger {
            kind: IntErrorKind::InvalidDigit
        }
    )]
    #[case(
        "Invalid integer (caused by PosOverflow)",
        Error::InvalidInteger {
            kind: IntErrorKind::PosOverflow
        }
    )]
    #[allow(deprecated)]
    #[case(
        "Invalid integer (caused by InvalidDigit)",
        Error::InvalidInteger {
            kind: IntErrorKind::InvalidDigit
        }
    )]
    #[case(
        "Invalid e-mail (Missing separator character '@'.)",
        email_address::Error::MissingSeparator.into()
    )]
    #[case(
        "Invalid integer (caused by InvalidDigit)",
        Error::InvalidInteger {
            kind: IntErrorKind::InvalidDigit
        }
    )]
    fn error_format_string(#[case] error_str: &str, #[case] error: Error) {
        assert_eq!(error_str, format!("{error}"));
    }
}
