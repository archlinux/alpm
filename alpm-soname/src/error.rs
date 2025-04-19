//! Error handling.

use std::path::PathBuf;

/// The Error that can occur when working with the library.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// IO path error
    #[error("I/O error at path {path} while {context}:\n{source}")]
    IoPathError {
        /// The path at which the error occurred.
        path: PathBuf,

        /// The context in which the error occurred at `path`.
        ///
        /// This is meant to complete the sentence "I/O error at path {path} while ".
        context: &'static str,

        /// The source of the error.
        source: std::io::Error,
    },

    /// I/O error while writing.
    #[error("IO write error while {context}:\n{source}")]
    IoWriteError {
        /// The context in which the error occurred.
        context: &'static str,

        /// The source of the error.
        source: std::io::Error,
    },

    /// ALPM PKGINFO error
    #[error(transparent)]
    AlpmPkginfo(#[from] alpm_pkginfo::Error),

    /// A winnow parser for a type didn't work and produced an error.
    #[error("Parser failed with the following error:\n{0}")]
    ParseError(String),

    /// PKGINFO not found in package
    #[error(".PKGINFO not found in package ({path})")]
    MissingPackageInfo { path: PathBuf },
}

impl<'a> From<winnow::error::ParseError<&'a str, winnow::error::ContextError>>
    for crate::error::Error
{
    /// Converts a [`winnow::error::ContextError`] into an [`Error::ParseError`].
    fn from(value: winnow::error::ParseError<&'a str, winnow::error::ContextError>) -> Self {
        Self::ParseError(value.to_string())
    }
}
