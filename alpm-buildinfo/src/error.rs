use std::{path::PathBuf, string::FromUtf8Error};

use alpm_types::SchemaVersion;
use thiserror::Error;

/// The Error that can occur when working with BUILDINFO files
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// ALPM type error
    #[error("ALPM type parse error: {0}")]
    AlpmType(#[from] alpm_types::Error),

    /// IO error
    #[error("I/O error at path {0:?} while {1}:\n{2}")]
    IoPathError(PathBuf, &'static str, std::io::Error),

    /// UTF-8 parse error
    #[error(transparent)]
    InvalidUTF8(#[from] FromUtf8Error),

    // Deserialize error
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
