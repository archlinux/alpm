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

    /// No input file given
    #[error("No input file given.")]
    NoInputFile,
    /// An error occurred while unpacking a gzip file.
    #[error("Error while unpacking gzip file:\n{0}")]
    InvalidGzip(std::io::Error),
}
