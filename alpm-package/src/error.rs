use std::path::PathBuf;

/// An error that can occur when dealing with alpm-package.
#[derive(Debug, thiserror::Error)]
pub enum Error {
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

    /// A package error.
    #[error("Package error:\n{0}")]
    Package(#[from] crate::package::Error),

    /// An [`alpm_pkginfo::Error`].
    #[error("PackageInfo error:\n{0}")]
    PackageInfo(#[from] alpm_pkginfo::Error),

    /// A path does not exist.
    #[error("The path {path} does not exist")]
    PathDoesNotExist {
        /// The path that should exist.
        path: PathBuf,
    },

    /// A path is not a directory.
    #[error("The path {path} is not a directory")]
    PathNotADir {
        /// The path that should be a directory.
        path: PathBuf,
    },
}
