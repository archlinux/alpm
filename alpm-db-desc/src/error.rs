use std::path::PathBuf;

use crate::parser::SectionKeyword;

/// The error that can occur when working with the ALPM database desc files.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A winnow parser for a type didn't work and produced an error.
    #[error("Parser failed with the following error:\n{0}")]
    ParseError(String),

    /// A section is missing in the parsed data.
    #[error("Missing section: {0}")]
    MissingSection(SectionKeyword),

    /// IO error
    #[error("I/O error while {0}:\n{1}")]
    Io(&'static str, std::io::Error),

    /// IO error
    #[error("I/O error at path {0:?} while {1}:\n{2}")]
    IoPathError(PathBuf, &'static str, std::io::Error),

    /// No input file given
    #[error("No input file given.")]
    NoInputFile,

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Failed to parse v1 or v2
    #[error("Failed to parse v1 or v2 format")]
    InvalidFormat,
}

impl<'a> From<winnow::error::ParseError<&'a str, winnow::error::ContextError>> for Error {
    /// Converts a [`winnow::error::ParseError`] into an [`Error::ParseError`].
    fn from(value: winnow::error::ParseError<&'a str, winnow::error::ContextError>) -> Self {
        Self::ParseError(value.to_string())
    }
}
