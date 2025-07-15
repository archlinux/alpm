use std::path::PathBuf;

use rust_i18n::t;

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
    /// An invalid integer
    #[error("{msg}", msg = t!("error.invalid_integer", kind = format!("{kind:?}")))]
    InvalidInteger {
        /// The reason for the invalid integer.
        kind: std::num::IntErrorKind,
    },

    /// An invalid enum variant
    #[error("{msg} ({0})", msg = t!("error.invalid_variant"))]
    InvalidVariant(#[from] strum::ParseError),

    /// An invalid email address
    #[error("{msg} ({0})", msg = t!("error.invalid_email"))]
    InvalidEmail(#[from] email_address::Error),

    /// An invalid URL
    #[error("{msg} ({0})", msg = t!("error.invalid_url"))]
    InvalidUrl(#[from] url::ParseError),

    /// An invalid license
    #[error("{msg} ({0})", msg = t!("error.invalid_license"))]
    InvalidLicense(#[from] spdx::ParseError),

    /// An invalid semantic version string
    ///
    /// This error occurs when a semantic version cannot be parsed from a string
    /// We cannot use `#[source] semver::Error` here because it does not implement `PartialEq`.
    /// See: <https://github.com/dtolnay/semver/issues/326>
    ///
    /// TODO: Use the error source when the issue above is resolved.
    #[error("{msg} ({kind})", msg = t!("error.invalid_semver"))]
    InvalidSemver {
        /// The reason for the invalid semantic version.
        kind: String,
    },

    /// Value contains an invalid character
    #[error("{msg} ({invalid_char})", msg = t!("error.contains_invalid_chars"))]
    ValueContainsInvalidChars {
        /// The invalid character
        invalid_char: char,
    },

    /// Value length is incorrect
    #[error("{msg}", msg = t!("error.incorrect_length", length = length, expected = expected))]
    IncorrectLength {
        /// The incorrect length.
        length: usize,
        /// The expected length.
        expected: usize,
    },

    /// Value is missing a delimiter character
    #[error("{msg}: {delimiter}", msg = t!("error.missing_delimiter"))]
    DelimiterNotFound {
        /// The required delimiter.
        delimiter: char,
    },

    /// Value does not match the restrictions
    #[error("{msg} ({restrictions:?})", msg = t!("error.restriction_mismatch"))]
    ValueDoesNotMatchRestrictions {
        /// The list of restrictions that cannot be met.
        restrictions: Vec<String>,
    },

    /// A validation regex does not match the value
    #[error("{msg}: {regex}", msg = t!("error.regex_mismatch", value = value, regex_type = regex_type))]
    RegexDoesNotMatch {
        /// The value that does not match.
        value: String,
        /// The type of regular expression applied to the `value`.
        regex_type: String,
        /// The regular expression applied to the `value`.
        regex: String,
    },

    /// A winnow parser for a type didn't work and produced an error.
    #[error("{msg}:\n{0}", msg = t!("error.parser_failed"))]
    ParseError(String),

    /// Missing field in a value
    #[error("{msg}: {component}", msg = t!("error.missing_component"))]
    MissingComponent {
        /// The component that is missing.
        component: &'static str,
    },

    /// An invalid absolute path (i.e. does not start with a `/`)
    #[error("{msg}: {0}", msg = t!("error.path_not_absolute"))]
    PathNotAbsolute(PathBuf),

    /// An invalid relative path (i.e. starts with a `/`)
    #[error("{msg}: {0}", msg = t!("error.path_not_relative"))]
    PathNotRelative(PathBuf),

    /// File name contains invalid characters
    #[error("{msg}: {0} ({1:?})", msg = t!("error.invalid_file_name_chars"))]
    FileNameContainsInvalidChars(PathBuf, char),

    /// File name is empty
    #[error("{msg}", msg = t!("error.empty_file_name"))]
    FileNameIsEmpty,

    /// A deprecated license
    #[error("{msg}: {0}", msg = t!("error.deprecated_license"))]
    DeprecatedLicense(String),

    /// An invalid OpenPGP v4 fingerprint
    #[error("{msg}", msg = t!("error.invalid_openpgp_fingerprint"))]
    InvalidOpenPGPv4Fingerprint,

    /// An invalid OpenPGP key ID
    #[error("{msg}: {0}", msg = t!("error.invalid_openpgp_key_id"))]
    InvalidOpenPGPKeyId(String),

    /// An invalid shared object name (v1)
    #[error("{msg}: {0}", msg = t!("error.invalid_soname_v1"))]
    InvalidSonameV1(&'static str),

    /// A package data error.
    #[error("{msg}: {0}", msg = t!("error.package_error"))]
    Package(#[from] crate::PackageError),

    /// A string represents an unknown compression algorithm file extension.
    #[error("{msg}: {value:?}", msg = t!("error.unknown_compression_ext"))]
    UnknownCompressionAlgorithmFileExtension {
        /// A String representing an unknown compression algorithm file extension.
        value: String,
    },

    /// A string represents an unknown file type identifier.
    #[error("{msg}: {value:?}", msg = t!("error.unknown_file_type"))]
    UnknownFileTypeIdentifier {
        /// A String representing an unknown file type identifier.
        value: String,
    },
}

impl From<std::num::ParseIntError> for crate::error::Error {
    /// Converts a [`std::num::ParseIntError`] into an [`Error::InvalidInteger`].
    fn from(e: std::num::ParseIntError) -> Self {
        Self::InvalidInteger {
            kind: e.kind().clone(),
        }
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
    use crate::openpgp::PACKAGER_REGEX;

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
        "Value '€i²' does not match the 'packager' regex: ^(?P<name>[\\w\\s\\-().]+) <(?P<email>.*)>$",
        Error::RegexDoesNotMatch {
            value: "€i²".to_string(),
            regex_type: "packager".to_string(),
            regex: PACKAGER_REGEX.to_string(),
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
