//! Error handling.

use std::{path::PathBuf, string::FromUtf8Error};

use alpm_types::MetadataFileName;

/// An error that can occur when dealing with alpm-package.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An [`alpm_buildinfo::Error`].
    #[error(transparent)]
    AlpmBuildInfo(#[from] alpm_buildinfo::Error),

    /// An [`alpm_common::Error`].
    #[error(transparent)]
    AlpmCommon(#[from] alpm_common::Error),

    /// An [`alpm_mtree::Error`].
    #[error(transparent)]
    AlpmMtree(#[from] alpm_mtree::Error),

    /// An [`alpm_mtree::mtree::path_validation_error::PathValidationError`].
    #[error(transparent)]
    AlpmMtreePathValidation(#[from] alpm_mtree::mtree::path_validation_error::PathValidationError),

    /// An [`alpm_pkginfo::Error`].
    #[error(transparent)]
    AlpmPackageInfo(#[from] alpm_pkginfo::Error),

    /// An [`alpm_types::Error`].
    #[error(transparent)]
    AlpmTypes(#[from] alpm_types::Error),

    /// An [`alpm_types::PackageError`].
    #[error(transparent)]
    AlpmTypesPackage(#[from] alpm_types::PackageError),

    /// An [`alpm_compress::Error`].
    #[error(transparent)]
    AlpmCompress(#[from] alpm_compress::Error),

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
    #[error("The package input directory is also used as the output directory: {path:?}")]
    InputDirIsOutputDir {
        /// The path to the directory that is used as both input and output.
        path: PathBuf,
    },

    /// A package output directory is located inside of a package input directory.
    #[error(
        "The package output directory ({output_path:?}) is located inside of the package input directory ({input_path:?})"
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

    /// An I/O error occurred while reading.
    #[error("I/O read error while {context}:\n{source}")]
    IoRead {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O read error while ".
        context: &'static str,
        /// The source error.
        source: std::io::Error,
    },

    /// UTF-8 parse error.
    #[error("Invalid UTF-8 while {context}:\n{source}")]
    InvalidUTF8 {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "Invalid UTF-8 while {context}".
        context: &'static str,
        /// The source error.
        source: FromUtf8Error,
    },

    /// Metadata file not found in package.
    #[error("Metadata file {name} not found in package")]
    MetadataFileNotFound {
        /// The metadata file that was not found.
        name: MetadataFileName,
    },

    /// Reached the end of known entries while reading a package.
    #[error("Reached the end of known entries while reading a package")]
    EndOfPackageEntries,

    /// A package input directory is located inside of a package output directory.
    #[error(
        "The package input directory ({input_path:?}) is located inside of the output directory ({output_path:?})"
    )]
    OutputDirInInputDir {
        /// The input directory path.
        input_path: PathBuf,
        /// The output directory path.
        output_path: PathBuf,
    },

    /// A [`crate::package::Error`].
    #[error("Package error:\n{0}")]
    Package(#[from] crate::package::Error),

    /// A path does not exist.
    #[error("The path {path:?} does not exist")]
    PathDoesNotExist {
        /// The path that should exist.
        path: PathBuf,
    },

    /// A path does not have a parent.
    #[error("The path {path:?} has no parent")]
    PathHasNoParent {
        /// The path that should have a parent.
        path: PathBuf,
    },

    /// A path is not a file.
    #[error("The path {path:?} is not a file")]
    PathIsNotAFile {
        /// The path that is not a file.
        path: PathBuf,
    },

    /// A path is read only.
    #[error("The path {path:?} is read-only")]
    PathIsReadOnly {
        /// The path that is read only.
        path: PathBuf,
    },
}
