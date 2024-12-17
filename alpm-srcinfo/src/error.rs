use std::path::PathBuf;
use std::string::FromUtf8Error;

use thiserror::Error;

/// The high-level error that can occur when using this crate.
///
/// Notably, it contains two important enums in the context of parsing:
/// - `ParseError` is a already formatted error generated by the `winnow` parser. This effectively
///   means that some invalid data has been encountered.
/// - `SourceInfoErrors` is a list of all logical or lint errors that're encountered in the final
///   step. This error also contains the original file on which the errors occurred.
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

    /// UTF-8 parse error when reading the input file.
    #[error(transparent)]
    InvalidUTF8(#[from] FromUtf8Error),

    /// No input file given.
    ///
    /// This error only occurs when running the [`crate::commands`] functions.
    #[error("No input file given.")]
    NoInputFile,

    /// A parsing error that occurred during winnow file parsing.
    #[error("File parsing error:\n{0}")]
    ParseError(String),
}
