//! Error type

use strum::Display;

/// Error type for voa-core
#[derive(Debug, Display, thiserror::Error)]
pub enum Error {
    /// Illegal data for an identifier (e.g. using illegal characters)
    IllegalIdentifier,

    /// Illegal symlink found during canonicalization
    IllegalSymlink,

    /// Wrapper for a [std::io::Error]
    Ioerror(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Ioerror(value)
    }
}
