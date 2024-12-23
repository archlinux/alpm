use std::{path::PathBuf, string::FromUtf8Error};

/// The Error that can occur when working with PKGINFO files
#[derive(Debug, thiserror::Error)]
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
    #[error("Failed to deserialize PKGINFO file:\n{0}")]
    DeserializeError(#[from] alpm_parsers::custom_ini::Error),

    /// No input file given
    #[error("No input file given.")]
    NoInputFile,

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// An invalid enum variant
    #[error("Invalid variant ({0})")]
    InvalidVariant(#[from] strum::ParseError),

    /// Extra data is missing
    #[error("Extra data is missing.")]
    MissingExtraData,
}
