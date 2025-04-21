use std::{collections::HashSet, path::PathBuf, string::FromUtf8Error};

#[cfg(doc)]
use crate::Mtree;
use crate::mtree::path_validation_error::PathValidationErrors;

/// The Error that can occur when working with ALPM-MTREE
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// An alpm-common error.
    #[error(transparent)]
    AlpmCommon(#[from] alpm_common::Error),

    /// There are duplicate paths.
    #[error("The following file system paths are duplicates:\n{}",
        paths.iter().fold(String::new(), |mut output, path| {
            output.push_str(&format!("{path:?}\n"));
            output
        })
    )]
    DuplicatePaths {
        /// The set of file system paths that are duplicates.
        paths: HashSet<PathBuf>,
    },

    /// File creation error.
    #[cfg(feature = "creation")]
    #[error("File creation error:\n{0}")]
    File(#[from] crate::CreationError),

    /// IO error
    #[error("I/O error while {0}:\n{1}")]
    Io(&'static str, std::io::Error),

    /// IO error with additional path info for more context.
    #[error("I/O error at path {0:?} while {1}:\n{2}")]
    IoPath(PathBuf, &'static str, std::io::Error),

    /// UTF-8 parse error
    #[error(transparent)]
    InvalidUTF8(#[from] FromUtf8Error),

    /// No input file given
    #[error("No input file given.")]
    NoInputFile,

    /// An error occurred while unpacking a gzip file.
    #[error("Error while unpacking gzip file:\n{0}")]
    InvalidGzip(std::io::Error),

    /// Validating paths in a base directory using [`Mtree`] data led to one or more errors.
    #[error(transparent)]
    PathValidation(#[from] PathValidationErrors),

    /// A Parsing error that occurred during the winnow file parsing.
    #[error("File parsing error:\n{0}")]
    ParseError(String),

    /// An error occurred during the interpretation phase of the language.
    #[error("Error while interpreting file in line {0}:\nAffected line:\n{1}\n\nReason:\n{2}")]
    InterpreterError(usize, String, String),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Unsupported schema version
    #[error("Unsupported schema version: {0}")]
    UnsupportedSchemaVersion(String),
}
