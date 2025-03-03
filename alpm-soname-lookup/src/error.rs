use std::path::PathBuf;

/// The Error that can occur when working with BUILDINFO files
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// IO error
    #[error("I/O error at path {0:?} while {1}:\n{2}")]
    IoPathError(PathBuf, &'static str, std::io::Error),

    /// ALPM PKGINFO error
    #[error(transparent)]
    AlpmPkginfo(#[from] alpm_pkginfo::Error),

    /// A winnow parser for a type didn't work and produced an error.
    #[error("Parser failed with the following error:\n{0}")]
    ParseError(String),

    /// PKGINFO not found in package
    #[error(".PKGINFO not found in package ({0})")]
    MissingPackageInfo(PathBuf),
}

impl<'a> From<winnow::error::ParseError<&'a str, winnow::error::ContextError>>
    for crate::error::Error
{
    /// Converts a [`winnow::error::ContextError`] into an [`Error::ParseError`].
    fn from(value: winnow::error::ParseError<&'a str, winnow::error::ContextError>) -> Self {
        Self::ParseError(value.to_string())
    }
}
