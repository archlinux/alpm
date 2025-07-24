use std::{path::PathBuf, string::FromUtf8Error};

use crate::LintScope;

/// Errors that can occur during alpm-lint.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// IO error
    #[error("I/O error while {context}:\n{source}")]
    Io {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O error while ...".
        context: &'static str,
        /// The error source.
        source: std::io::Error,
    },

    /// IO error with additional path info for more context.
    #[error("I/O error at path {path:?} while {context}:\n{source}")]
    IoPath {
        /// The path at which the error occurred.
        path: PathBuf,
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O error at path $path while ...".
        context: &'static str,
        /// The error source
        source: std::io::Error,
    },

    /// The lint scope couldn't be automatically detected.
    #[error(
        "The lint scope could not be detected for path '{path}'. Please set it manually via check-scope"
    )]
    NoLintScope {
        /// The path for which no lint scope could be determined.
        path: PathBuf,
    },

    /// The lint scope couldn't be automatically detected.
    #[error("Invalid path type for lint scope '{scope}' at {path:?}. Expected a {expected}.")]
    InvalidPathForLintScope {
        /// The path that is invalid.
        path: PathBuf,
        /// The set [`LintScope`]
        scope: LintScope,
        /// The type of path we expected to find.
        expected: &'static str,
    },

    /// An invalid lint scope has been provided to a function.
    #[error(
        "Invalid lint scope '{scope}' provided to function '{function}'. Expected a {expected}."
    )]
    InvalidLintScope {
        /// The [`LintScope`] that was encountered.
        scope: LintScope,
        /// The function context for this error.
        /// Use to complete the sentence:
        /// `Invalid lint scope '{scope}' provided to function '{function}'`.
        function: &'static str,
        /// The type of [`LintScope`] that was expected.
        /// Used to complete the sentence `Expected a {expected}`.
        expected: &'static str,
    },

    /// UTF-8 parse error
    #[error(transparent)]
    InvalidUTF8(#[from] FromUtf8Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML serialization error
    #[error("TOML error: {0}")]
    Toml(#[from] toml::ser::Error),

    /// `alpm-buildinfo` error.
    #[error(transparent)]
    BuildInfo(#[from] alpm_buildinfo::Error),

    /// `alpm-pkgbuild` error.
    #[error(transparent)]
    PackageBuild(#[from] alpm_pkgbuild::Error),

    /// `alpm-pkginfo` error.
    #[error(transparent)]
    PackageInfo(#[from] alpm_pkginfo::Error),

    /// `alpm-srcinfo` error.
    #[error(transparent)]
    SourceInfo(#[from] alpm_srcinfo::Error),
}
