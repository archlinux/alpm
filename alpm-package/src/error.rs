//! Error handling.

use std::path::PathBuf;

/// An error that can occur when dealing with alpm-package.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An alpm-common error.
    #[error("ALPM common error:\n{0}")]
    AlpmCommon(#[from] alpm_common::Error),

    /// An alpm-types error.
    #[error("ALPM types error:\n{0}")]
    AlpmTypes(#[from] alpm_types::Error),

    /// An alpm-types package error.
    #[error("ALPM types package error:\n{0}")]
    AlpmTypesPackage(#[from] alpm_types::PackageError),

    /// An [`alpm_buildinfo::Error`].
    #[error("BuildInfo error:\n{0}")]
    BuildInfo(#[from] alpm_buildinfo::Error),

    /// An error with an [alpm-install-scriptlet].
    ///
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    #[error("The alpm-install-scriptlet at {path} is invalid because {context}")]
    InstallScriptlet {
        /// The path to the alpm-install-scriptlet.
        path: PathBuf,
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "The alpm-install-scriptlet at {path} is invalid
        /// because {context}".
        context: String,
    },

    /// An I/O error occurred at a path.
    #[error("I/O error at path {path} while {context}:\n{source}")]
    IoPath {
        /// The path at which the error occurred.
        path: PathBuf,
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O error at path while ".
        context: &'static str,
        /// The source error.
        source: std::io::Error,
    },

    /// An [`alpm_mtree::Error`].
    #[error("Mtree error:\n{0}")]
    Mtree(#[from] alpm_mtree::Error),

    /// A package input error.
    #[error("Package input error:\n{0}")]
    Input(#[from] crate::input::Error),

    /// A package error.
    #[error("Package error:\n{0}")]
    Package(#[from] crate::package::Error),

    /// An [`alpm_pkginfo::Error`].
    #[error("PackageInfo error:\n{0}")]
    PackageInfo(#[from] alpm_pkginfo::Error),

    /// A winnow parser for a type didn't work and produced an error.
    #[error("Parser failed with the following error:\n{0}")]
    ParseError(String),

    /// A path does not exist.
    #[error("The path {path} does not exist")]
    PathDoesNotExist {
        /// The path that should exist.
        path: PathBuf,
    },

    /// A path does not have a parent.
    #[error("The path {path} has no parent")]
    PathNoParent {
        /// The path that should have a parent.
        path: PathBuf,
    },

    /// A path is not a directory.
    #[error("The path {path} is not a directory")]
    PathNotADir {
        /// The path that should be a directory.
        path: PathBuf,
    },

    /// A strum parser error.
    #[error("Strum parser error:\n{0}")]
    StrumParse(#[from] strum::ParseError),
}

impl<'a> From<winnow::error::ParseError<&'a str, winnow::error::ContextError>>
    for crate::error::Error
{
    /// Converts a [`winnow::error::ParseError`] into an [`Error::ParseError`].
    fn from(value: winnow::error::ParseError<&'a str, winnow::error::ContextError>) -> Self {
        Self::ParseError(value.to_string())
    }
}
