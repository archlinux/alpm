use std::path::PathBuf;
use std::string::FromUtf8Error;

use thiserror::Error;

/// The Error that can occur when using types
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// ALPM type error
    #[error("ALPM type parse error: {0}")]
    AlpmType(#[from] alpm_types::Error),

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

    /// A Parsing error that occurred during the winnow file parsing.
    #[error("File parsing error:\n{0}")]
    ParseError(String),

    /// An error occurred during the interpretation phase of the language.
    #[error("Error while interpreting file in line {0}:\nAffected line:\n{1}\n\nReason:\n{2}")]
    InterpreterError(usize, String, String),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
