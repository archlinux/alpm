use std::{path::PathBuf, string::FromUtf8Error};

use alpm_types::SchemaVersion;

/// The Error that can occur when working with BUILDINFO files
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// ALPM type error
    #[error("ALPM type parse error: {0}")]
    AlpmType(#[from] alpm_types::Error),

    /// IO error
    #[error("I/O error at path {0:?} while {1}:\n{2}")]
    IoPathError(PathBuf, &'static str, std::io::Error),

    /// I/O error while reading a buffer.
    #[error("Read error while {context}:\n{source}")]
    IoReadError {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "Read error while ".
        context: &'static str,
        /// The error source.
        source: std::io::Error,
    },

    /// UTF-8 parse error
    #[error(transparent)]
    InvalidUTF8(#[from] FromUtf8Error),

    /// An [`alpm_parsers::custom_ini::Error`].
    #[error("Failed to deserialize BUILDINFO file:\n{0}")]
    DeserializeError(#[from] alpm_parsers::custom_ini::Error),

    /// No input file given
    #[error("No input file given.")]
    NoInputFile,

    /// Unsupported schema version
    #[error("Unsupported schema version: {0}")]
    UnsupportedSchemaVersion(String),

    /// A SchemaVersion with the wrong version is used
    #[error("Wrong schema version used to create a BUILDINFO: {0}")]
    WrongSchemaVersion(SchemaVersion),

    /// BuildInfo file is missing the format field
    #[error("Missing format field")]
    MissingFormatField,

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
