use std::path::PathBuf;

use fluent_i18n::t;

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
    #[error("{msg}", msg = t!("error-invalid-architectures", {
        "architectures" => format!("{architectures:?}"),
        "context" => context
    }))]
    InvalidArchitectures {
        /// The invalid architectures combination.
        architectures: Vec<Architecture>,
        /// The reason why the architectures are invalid.
        context: &'static str,
    },

    /// An invalid integer
    #[error("{msg}", msg = t!("error-invalid-integer", { "kind" => format!("{kind:?}") }))]
    InvalidInteger {
        /// The reason for the invalid integer.
        kind: std::num::IntErrorKind,
    },

    /// An invalid enum variant
    #[error("{msg}", msg = t!("error-invalid-variant", { "error" => .0.to_string() }))]
    InvalidVariant(#[from] strum::ParseError),

    /// An invalid email address
    #[error("{msg}", msg = t!("error-invalid-email", { "error" => .0.to_string() }))]
    InvalidEmail(#[from] email_address::Error),

    /// An invalid URL
    #[error("{msg}", msg = t!("error-invalid-url", { "error" => .0.to_string() }))]
    InvalidUrl(#[from] url::ParseError),

    /// An invalid license
    #[error("{msg}", msg = t!("error-invalid-license", { "error" => .0.to_string() }))]
    InvalidLicense(#[from] spdx::ParseError),

    /// An invalid semantic version string
    ///
    /// This error occurs when a semantic version cannot be parsed from a string.
    /// We cannot use `#[source] semver::Error` here because it does not implement `PartialEq`.
    /// See: <https://github.com/dtolnay/semver/issues/326>
    ///
    /// TODO: Use the error source when the issue above is resolved.
    #[error("{msg}", msg = t!("error-invalid-semver", { "kind" => kind }))]
    InvalidSemver {
        /// The reason for the invalid semantic version.
        kind: String,
    },

    /// Value contains invalid characters
    #[error("{msg}", msg = t!("error-invalid-chars", { "invalid_char" => invalid_char.to_string() }))]
    ValueContainsInvalidChars {
        /// The invalid character
        invalid_char: char,
    },

    /// Value length is incorrect
    #[error("{msg}", msg = t!("error-incorrect-length", { "length" => length, "expected" => expected }))]
    IncorrectLength {
        /// The incorrect length.
        length: usize,
        /// The expected length.
        expected: usize,
    },

    /// Value is missing a delimiter character
    #[error("{msg}", msg = t!("error-delimiter-not-found", { "delimiter" => delimiter.to_string() }))]
    DelimiterNotFound {
        /// The required delimiter.
        delimiter: char,
    },

    /// Value does not match the restrictions
    #[error("{msg}", msg = t!("error-restrictions-not-met", {
        "restrictions" => format!("{restrictions:?}")
    }))]
    ValueDoesNotMatchRestrictions {
        /// The list of restrictions that cannot be met.
        restrictions: Vec<String>,
    },

    /// A validation regex does not match the value
    #[error("{msg}", msg = t!("error-regex-mismatch", {
        "value" => value,
        "regex_type" => regex_type,
        "regex" => regex
    }))]
    RegexDoesNotMatch {
        /// The value that does not match.
        value: String,
        /// The type of regular expression applied to the `value`.
        regex_type: String,
        /// The regular expression applied to the `value`.
        regex: String,
    },

    /// A winnow parser for a type didn't work and produced an error.
    #[error("{msg}", msg = t!("error-parse", { "error" => .0 }))]
    ParseError(String),

    /// Missing field in a value
    #[error("{msg}", msg = t!("error-missing-component", { "component" => component }))]
    MissingComponent {
        /// The component that is missing.
        component: &'static str,
    },

    /// An invalid absolute path (i.e. does not start with a `/`)
    #[error("{msg}", msg = t!("error-path-not-absolute", { "path" => .0 }))]
    PathNotAbsolute(PathBuf),

    /// An invalid relative path (i.e. starts with a `/`)
    #[error("{msg}", msg = t!("error-path-not-relative", { "path" => .0 }))]
    PathNotRelative(PathBuf),

    /// Expected a file, but got a directory
    #[error("{msg}", msg = t!("error-path-not-file", { "path" => .0 }))]
    PathIsNotAFile(PathBuf),

    /// File name contains invalid characters
    #[error("{msg}", msg = t!("error-filename-invalid-chars", {
        "path" => .0,
        "invalid_char" => .1.to_string()
    }))]
    FileNameContainsInvalidChars(PathBuf, char),

    /// File name is empty
    #[error("{msg}", msg = t!("error-filename-empty"))]
    FileNameIsEmpty,

    /// A deprecated license
    #[error("{msg}", msg = t!("error-deprecated-license", { "license" => .0 }))]
    DeprecatedLicense(String),

    /// A component is invalid and cannot be used.
    #[error("{msg}", msg = t!("error-invalid-component", {
        "component" => component,
        "context" => context
    }))]
    InvalidComponent {
        /// The invalid component.
        component: &'static str,
        /// The context in which the error occurs.
        ///
        /// This is meant to complete the sentence
        /// "Invalid component {component} encountered while ".
        context: String,
    },

    /// An invalid OpenPGP v4 fingerprint
    #[error("{msg}", msg = t!("error-invalid-pgp-fingerprint"))]
    InvalidOpenPGPv4Fingerprint,

    /// An invalid OpenPGP key ID
    #[error("{msg}", msg = t!("error-invalid-pgp-keyid", { "keyid" => .0 }))]
    InvalidOpenPGPKeyId(String),

    /// An invalid OpenPGP signature
    #[error("{msg}", msg = t!("error-invalid-base64-encoding", { "expected_item" => expected_item }))]
    InvalidBase64Encoding {
        /// The expected item that could not be decoded.
        expected_item: String,
    },

    /// An invalid shared object name (v1)
    #[error("{msg}", msg = t!("error-invalid-soname-v1", { "name" => .0 }))]
    InvalidSonameV1(&'static str),

    /// A package data error.
    #[error("{msg}", msg = t!("error-package", { "error" => .0.to_string() }))]
    Package(#[from] crate::PackageError),

    /// A string represents an unknown compression algorithm file extension.
    #[error("{msg}", msg = t!("error-unknown-compression", { "value" => value }))]
    UnknownCompressionAlgorithmFileExtension {
        /// A string representing an unknown compression algorithm file extension.
        value: String,
    },

    /// A string represents an unknown file type identifier.
    #[error("{msg}", msg = t!("error-unknown-filetype", { "value" => value }))]
    UnknownFileTypeIdentifier {
        /// A string representing an unknown file type identifier.
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
