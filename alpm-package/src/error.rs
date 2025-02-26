//! Error handling.

use std::path::PathBuf;

/// An error that can occur when dealing with alpm-package.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An [`alpm_buildinfo::Error`].
    #[error("BuildInfo error:\n{0}")]
    AlpmBuildInfo(#[from] alpm_buildinfo::Error),

    /// An alpm-common error.
    #[error(transparent)]
    AlpmCommon(#[from] alpm_common::Error),

    /// An [`alpm_mtree::Error`].
    #[error("Mtree error:\n{0}")]
    AlpmMtree(#[from] alpm_mtree::Error),

    /// An [`alpm_pkginfo::Error`].
    #[error("PackageInfo error:\n{0}")]
    AlpmPackageInfo(#[from] alpm_pkginfo::Error),

    /// An alpm-types error.
    #[error(transparent)]
    AlpmTypes(#[from] alpm_types::Error),

    /// An alpm-types package error.
    #[error("ALPM types package error:\n{0}")]
    AlpmTypesPackage(#[from] alpm_types::PackageError),

    /// A compression error.
    #[error("Compression error:\n{0}")]
    Compression(#[from] crate::compression::Error),

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

    /// A package input error.
    #[error("Package input error:\n{0}")]
    Input(#[from] crate::input::Error),

    /// A package input directory is also used as the package output directory.
    #[error(
        "The package input directory {input_path:?} is also used as the output directory {output_path:?}"
    )]
    InputDirIsOutputDir {
        /// The input directory path.
        input_path: PathBuf,
        /// The output directory path.
        output_path: PathBuf,
    },

    /// A package output directory is located inside of a package input directory.
    #[error(
        "The package output directory {output_path:?} is located inside of the package input directory {input_path:?}"
    )]
    InputDirInOutputDir {
        /// The input directory path.
        input_path: PathBuf,
        /// The output directory path.
        output_path: PathBuf,
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

    /// A package input directory is located inside of a package output directory.
    #[error(
        "The package input directory {input_path:?} is located inside of the output directory {output_path:?}"
    )]
    OutputDirInInputDir {
        /// The input directory path.
        input_path: PathBuf,
        /// The output directory path.
        output_path: PathBuf,
    },

    /// A package error.
    #[error("Package error:\n{0}")]
    Package(#[from] crate::package::Error),

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

    /// A path is not a file.
    #[error("The path {path} is not a file")]
    PathNotAFile {
        /// The path that is not a file.
        path: PathBuf,
    },

    /// A path is read only.
    #[error("The path {path:?} is read-only")]
    PathReadOnly {
        /// The path that is read only.
        path: PathBuf,
    },
}
