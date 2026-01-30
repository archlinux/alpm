use std::{collections::HashSet, path::PathBuf, string::FromUtf8Error};

use fluent_i18n::t;

#[cfg(doc)]
use crate::Mtree;
use crate::mtree::path_validation_error::PathValidationErrors;

/// The Error that can occur when working with ALPM-MTREE.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// An alpm-common error.
    #[error(transparent)]
    AlpmCommon(#[from] alpm_common::Error),

    /// There are duplicate paths.
    #[error("{msg}", msg = t!("error-duplicate-paths", {
        "paths" => paths.iter()
            .map(|p| format!("{p:?}"))
            .collect::<Vec<_>>()
            .join("\n")
    }))]
    DuplicatePaths {
        /// The set of file system paths that are duplicates.
        paths: HashSet<PathBuf>,
    },

    /// File creation error.
    #[cfg(feature = "creation")]
    #[error("{msg}", msg = t!("error-file-creation", { "source" => .0.to_string() }))]
    File(#[from] crate::CreationError),

    /// IO error.
    #[error("{msg}", msg = t!("error-io", {
        "context" => context,
        "source" => source.to_string()
    }))]
    Io {
        /// The context of the error.
        context: String,
        /// The underlying IO error.
        source: std::io::Error,
    },

    /// IO error with additional path info for more context.
    #[error("{msg}", msg = t!("error-io-path", {
        "path" => path,
        "context" => context,
        "source" => source.to_string()
    }))]
    IoPath {
        /// The path related to the error.
        path: PathBuf,
        /// The context of the error.
        context: String,
        /// The underlying IO error.
        source: std::io::Error,
    },

    /// UTF-8 parse error.
    #[error(transparent)]
    InvalidUTF8(#[from] FromUtf8Error),

    /// An error occurred while unpacking a gzip file.
    #[error("{msg}", msg = t!("error-invalid-gzip", { "source" => .0.to_string() }))]
    InvalidGzip(std::io::Error),

    /// Validating paths in a base directory using [`Mtree`] data led to one or more errors.
    #[error(transparent)]
    PathValidation(#[from] PathValidationErrors),

    /// A parsing error that occurred during the winnow file parsing.
    #[error("{msg}", msg = t!("error-parse", { "error" => .0 }))]
    Parse(String),

    /// An error occurred during the interpretation phase of the language.
    #[error("{msg}", msg = t!("error-interpreter", {
        "line_number" => .0.to_string(),
        "line" => .1,
        "reason" => .2,
    }))]
    InterpreterError(usize, String, String),

    /// Unsupported schema version
    #[error("Unsupported schema version: {0}")]
    UnsupportedSchemaVersion(String),
}
