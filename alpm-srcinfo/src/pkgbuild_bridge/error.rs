//! The error types used in the scope of `alpm-pkgbuild-bridge` output logic.

use alpm_pkgbuild::bridge::Keyword;
use alpm_types::{Name, SystemArchitecture};
use fluent_i18n::t;
use thiserror::Error;
use winnow::error::{ContextError, ParseError};

#[cfg(doc)]
use crate::SourceInfo;

/// A lower-level error that may occur when converting `alpm-pkgbuild-bridge` script output into the
/// [`SourceInfo`] format.
#[derive(Debug, Error)]
pub enum BridgeError {
    /// ALPM type parse error
    #[error(transparent)]
    AlpmType(#[from] alpm_types::Error),

    /// No `pkgname` has been specified.
    #[error("{msg}", msg = t!("error-bridge-no-name"))]
    NoName,

    /// A package name is not valid.
    #[error("{msg}", msg = t!("error-bridge-invalid-package-name", {
        "name" => name,
        "error" => error.to_string()
    }))]
    InvalidPackageName {
        /// The invalid package name.
        name: String,
        /// The source error.
        error: alpm_types::Error,
    },

    /// A `package` function has been declared for a split package, but it is not defined in
    /// `pkgname`.
    #[error("{msg}", msg = t!("error-bridge-undeclared-package", { "name" => .0 }))]
    UndeclaredPackageName(String),

    /// An unused package function exists for an undeclared [alpm-split-package].
    ///
    /// [alpm-split-package]: https://alpm.archlinux.page/specifications/alpm-split-package.7.html
    #[error("{msg}", msg = t!("error-bridge-unused-package-function", { "name" => .0.to_string() }))]
    UnusedPackageFunction(Name),

    /// A type parser fails on a certain keyword.
    #[error("{msg}", msg = t!("error-bridge-missing-required-keyword", { "keyword" => keyword.to_string() }))]
    MissingRequiredKeyword {
        /// The keyword that cannot be parsed.
        keyword: Keyword,
    },

    /// A type parser fails on a certain keyword.
    #[error("{msg}", msg = t!("error-bridge-parse-error", {
        "keyword" => keyword.to_string(),
        "error" => error
    }))]
    ParseError {
        /// The keyword that cannot be parsed.
        keyword: Keyword,
        /// The error message.
        error: String,
    },

    /// A variable is expected to be of a different type.
    /// E.g. `String` when an `Array` is expected.
    #[error("{msg}", msg = t!("error-bridge-wrong-variable-type", {
        "keyword" => keyword,
        "expected" => expected,
        "actual" => actual
    }))]
    WrongVariableType {
        /// The name of the keyword for which a wrong variable type is used.
        keyword: String,
        /// The expected type of variable.
        expected: String,
        /// The actual type of variable.
        actual: String,
    },

    /// A keyword has an architecture suffix even though it shouldn't have one.
    #[error("{msg}", msg = t!("error-bridge-unexpected-arch", {
        "keyword" => keyword.to_string(),
        "suffix" => suffix.to_string()
    }))]
    UnexpectedArchitecture {
        /// The keyword for which an unexpected architecture suffix is found.
        keyword: Keyword,
        /// The architecture that is found for the `keyword`.
        suffix: SystemArchitecture,
    },

    /// A keyword that cannot be cleared is attempted to be cleared.
    #[error("{msg}", msg = t!("error-bridge-unclearable-value", { "keyword" => keyword.to_string() }))]
    UnclearableValue {
        /// The keyword that is attempted to be cleared.
        keyword: Keyword,
    },

    /// A keyword should have only a single value, but an array is found.
    #[error("{msg}", msg = t!("error-bridge-unexpected-array", {
        "keyword" => keyword.to_string(),
        "values" => values.iter().map(|s| format!("\"{s}\"")).collect::<Vec<_>>().join(", ")
    }))]
    UnexpectedArray {
        /// The keyword for which a single value should be used.
        keyword: Keyword,
        /// The values that are used for the `keyword`.
        values: Vec<String>,
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
