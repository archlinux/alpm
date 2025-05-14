//! The error types used in the scope of `PKGBUILD` bridge logic.

use alpm_types::{Architecture, Name};
use thiserror::Error;
use winnow::error::{ContextError, ParseError};

use super::parser::Keyword;

/// A lower-level error that occurs when converting PKGBUILD bridge output into the SourceInfo
/// format.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BridgeError {
    #[error("No 'pkgname' has been specified. At least one must be given.")]
    NoName,

    #[error("An invalid package '{name}' name has been used:\n{error}")]
    InvalidPackageName {
        name: String,
        error: alpm_types::Error,
    },

    #[error(
        "Found package function for split package '{0}', which hasn't been declared in package base."
    )]
    UndeclaredPackageName(String),

    #[error("Extra package() function for split package '{0}'")]
    ExtraPackageFunction(Name),

    /// A type parser fails on a certain keyword.
    #[error("Missing keyword '{keyword}'.")]
    MissingRequiredKeyword { keyword: Keyword },

    /// A type parser fails on a certain keyword.
    #[error("Failed to parse input for keyword '{keyword}':\n{error}")]
    ParseError { keyword: Keyword, error: String },

    /// A variable is expected to be of a different type.
    /// E.g. `String` when an `Array` is expected.
    #[error(
        "Got wrong variable type for keyword '{keyword}'. Expected a {expected}, got a {actual}"
    )]
    WrongVariableType {
        keyword: String,
        expected: String,
        actual: String,
    },

    /// A keyword has a architecture suffix even though it shouldn't have one.
    #[error("Found unexpected architecture suffix '{suffix}' for keyword '{keyword}'")]
    UnexpectedArchitecture {
        keyword: Keyword,
        suffix: Architecture,
    },

    /// A keyword has a architecture suffix even though it shouldn't have one.
    #[error("Tried to clear value for keyword '{keyword}', which is not allowed.")]
    UnclearableValue { keyword: Keyword },

    /// A keyword should have only a single value, but an array is found.
    #[error(
        "Found array of values for keyword '{keyword}' that expects a single value:\n{values:?}"
    )]
    UnexpectedArray {
        keyword: Keyword,
        values: Vec<String>,
    },

    // A logical error in the PKGBUILD's content.
    #[error("Found duplicate architecture: {duplicate}.")]
    DuplicateArchitecture { duplicate: Architecture },
}

impl<'a> From<(Keyword, ParseError<&'a str, ContextError>)> for BridgeError {
    /// Converts a tuple of ([`Keyword`] and [`ParseError`]) into a [`BridgeError::ParseError`].
    fn from(value: (Keyword, ParseError<&'a str, ContextError>)) -> Self {
        Self::ParseError {
            keyword: value.0,
            error: value.1.to_string(),
        }
    }
}
