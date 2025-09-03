use std::path::PathBuf;

use crate::LintScope;

/// Errors that can occur when using alpm-lint.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error.
    #[error("I/O error while {context}:\n{source}")]
    Io {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O error while ...".
        context: &'static str,
        /// The error source.
        source: std::io::Error,
    },

    /// I/O error with additional path info for more context.
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

    /// No lint scope could be automatically detected for a path.
    #[error(
        "The lint scope could not be detected for path '{path}'. Please set it manually via --scope."
    )]
    NoLintScope {
        /// The path for which no lint scope could be determined.
        path: PathBuf,
    },

    /// The wrong type of path was provided for a lint scope.
    ///
    /// # Example
    ///
    /// The [`LintScope::Package`] scope expects a directory. If a file is provided, this error
    /// will be thrown.
    #[error("Invalid path type for lint scope '{scope}' at {path:?}. Expected a {expected}.")]
    InvalidPathForLintScope {
        /// The path that is invalid.
        path: PathBuf,
        /// The set [`LintScope`]
        scope: LintScope,
        /// The type of path we expected to find.
        expected: &'static str,
    },

    /// An incompatible lint scope has been provided to a function.
    #[error(
        "Incompatible lint scope '{scope}' provided to function '{function}'. Expected a {expected}."
    )]
    InvalidLintScope {
        /// The [`LintScope`] that is encountered.
        scope: LintScope,
        /// The function context for this error.
        ///
        /// Used to complete the sentence:
        /// `Incompatible lint scope '{scope}' provided to function '{function}'`.
        function: &'static str,
        /// Something that is expected instead.
        ///
        /// Used to complete the sentence `Expected a {expected}`.
        expected: &'static str,
    },

    /// An incompatible resource of specific scope is provided to a lint rule.
    #[error(
        "Invalid resources of scope '{scope}' provided to lint rule '{lint_rule}'. Expected a {expected}."
    )]
    InvalidResources {
        /// The [`LintScope`] that was encountered.
        scope: LintScope,
        /// The lint rule that produces the error.
        ///
        /// Use to complete the sentence:
        /// `Invalid resources of scope '{scope}' provided to lint rule '{lint_rule}'`.```
        lint_rule: String,
        /// The expected [`LintScope`] for the `lint_rule`.
        ///
        /// Used to complete the sentence `Expected a {expected}`.
        expected: LintScope,
    },

    /// JSON serialization error.
    #[error("JSON serialization error for {context}: {error}")]
    Json {
        /// The error source
        error: serde_json::Error,
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "JSON serialization error for ...".
        context: String,
    },

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

    /// `alpm-lint-config` error.
    #[error(transparent)]
    LintConfig(#[from] alpm_lint_config::Error),
}
