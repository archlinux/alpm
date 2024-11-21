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
}
