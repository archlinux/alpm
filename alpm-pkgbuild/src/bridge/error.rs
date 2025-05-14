//! The error types used in the scope of `alpm-pkgbuild-bridge` output logic.

#[cfg(doc)]
use alpm_srcinfo::SourceInfo;
use alpm_types::{Architecture, Name};
use thiserror::Error;
use winnow::error::{ContextError, ParseError};

use super::parser::Keyword;

/// A lower-level error that may occur when converting `alpm-pkgbuild-bridge` script output into the
/// [`SourceInfo`] format.
#[derive(Debug, Error)]
pub enum BridgeError {
    /// No `pkgname` has been specified.
    #[error("No 'pkgname' has been specified. At least one must be given.")]
    NoName,

    /// A package name is not valid.
    #[error("The package name '{name}' is not valid:\n{error}")]
    InvalidPackageName {
        /// The invalid package name.
        name: String,
        /// The source error.
        error: alpm_types::Error,
    },

    /// A `package` function has been declared for a split package, but it is not defined in
    /// `pkgname`.
    #[error(
        "The split package '{0}' is not declared in pkgname, but a package function is present for it."
    )]
    UndeclaredPackageName(String),

    /// An unused package function exists for an undeclared [alpm-split-package].
    ///
    /// [alpm-split-package]: https://alpm.archlinux.page/specifications/alpm-split-package.7.html
    #[error("An unused package function exists for undeclared split package: '{0}'")]
    UnusedPackageFunction(Name),

    /// A type parser fails on a certain keyword.
    #[error("Missing keyword: '{keyword}'")]
    MissingRequiredKeyword {
        /// The keyword that cannot be parsed.
        keyword: Keyword,
    },

    /// A type parser fails on a certain keyword.
    #[error("Failed to parse input for keyword '{keyword}':\n{error}")]
    ParseError {
        /// The keyword that cannot be parsed.
        keyword: Keyword,
        /// The error message.
        error: String,
    },

    /// A variable is expected to be of a different type.
    /// E.g. `String` when an `Array` is expected.
    #[error(
        "Got wrong variable type for keyword '{keyword}'. Expected a {expected}, got a {actual}"
    )]
    WrongVariableType {
        /// The name of the keyword for which a wrong variable type is used.
        keyword: String,
        /// The expected type of variable.
        expected: String,
        /// The actual type of variable.
        actual: String,
    },

    /// A keyword has an architecture suffix even though it shouldn't have one.
    #[error("Found unexpected architecture suffix '{suffix}' for keyword '{keyword}'")]
    UnexpectedArchitecture {
        /// The keyword for which an unexpected architecture suffix is found.
        keyword: Keyword,
        /// The architecture that is found for the `keyword`.
        suffix: Architecture,
    },

    /// A keyword that cannot be cleared is attempted to be cleared.
    #[error("Tried to clear value for keyword '{keyword}', which is not allowed.")]
    UnclearableValue {
        /// The keyword that is attempted to be cleared.
        keyword: Keyword,
    },

    /// A keyword should have only a single value, but an array is found.
    #[error(
        "Found array of values for keyword '{keyword}' that expects a single value:\n{}",
        values.iter().map(|s| format!("\"{s}\"")).collect::<Vec<String>>().join(", ")
    )]
    UnexpectedArray {
        /// The keyword for which a single value should be used.
        keyword: Keyword,
        /// The values that are used for the `keyword`.
        values: Vec<String>,
    },

    /// A duplicate [`Architecture`] is specified.
    #[error("Found duplicate architecture: {duplicate}")]
    DuplicateArchitecture {
        /// The duplicate architecture.
        duplicate: Architecture,
    },
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
