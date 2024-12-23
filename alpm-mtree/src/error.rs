use std::path::PathBuf;
use std::string::FromUtf8Error;

/// The Error that can occur when working with ALPM-MTREE
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// IO error
    #[error("I/O error while {0}:\n{1}")]
    Io(&'static str, std::io::Error),

    /// IO error with additional path info for more context.
    #[error("I/O error at path {0:?} while {1}:\n{2}")]
    IoPath(PathBuf, &'static str, std::io::Error),

    /// UTF-8 parse error
    #[error(transparent)]
    InvalidUTF8(#[from] FromUtf8Error),

    /// MTREE has its own unicode encoding for non-ascii chars.
    /// This error is thrown if an malformed sequence is found.
    #[error("Malformed mtree unicode encoding has been found in sequence {0}:\nError: {1}\n")]
    InvalidMtreeUnicode(String, String),

    /// No input file given
    #[error("No input file given.")]
    NoInputFile,

    /// An error occurred while unpacking a gzip file.
    #[error("Error while unpacking gzip file:\n{0}")]
    InvalidGzip(std::io::Error),

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
