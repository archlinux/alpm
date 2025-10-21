//! Error handling.

use std::{path::PathBuf, string::FromUtf8Error};

use alpm_types::MetadataFileName;
use fluent_i18n::t;

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
    #[error("{msg}", msg = t!("error-install-scriptlet", {
        "path" => path,
        "context" => context
    }))]
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
    #[error("{msg}", msg = t!("error-package-input", { "source" => 0.to_string() }))]
    Input(#[from] crate::input::Error),

    /// A package input directory is also used as the package output directory.
    #[error("{msg}", msg = t!("error-input-dir-is-output-dir", { "path" => path }))]
    InputDirIsOutputDir {
        /// The path used as both input and output.
        path: PathBuf,
    },

    /// A package output directory is located inside of a package input directory.
    #[error("{msg}", msg = t!("error-input-dir-in-output-dir", {
        "input_path" => input_path,
        "output_path" => output_path,
    }))]
    InputDirInOutputDir {
        /// The input directory path.
        input_path: PathBuf,
        /// The output directory path.
        output_path: PathBuf,
    },

    /// An I/O error occurred at a path.
    #[error("{msg}", msg = t!("error-io-path", {
        "path" => path,
        "context" => context,
        "source" => source.to_string()
    }))]
    IoPath {
        /// The path at which the error occurred.
        path: PathBuf,
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O error at path while ".
        context: String,
        /// The source error.
        source: std::io::Error,
    },

    /// An I/O error occurred while reading.
    #[error("{msg}", msg = t!("error-io-read", {
        "context" => context,
        "source" => source.to_string()
    }))]
    IoRead {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O read error while ".
        context: String,
        /// The source error.
        source: std::io::Error,
    },

    /// UTF-8 parse error.
    #[error("{msg}", msg = t!("error-invalid-utf8", {
        "context" => context,
        "source" => source.to_string()
    }))]
    InvalidUTF8 {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "Invalid UTF-8 while {context}".
        context: String,
        /// The source error.
        source: FromUtf8Error,
    },

    /// Metadata file not found in package.
    #[error("{msg}", msg = t!("error-metadata-not-found", { "name" => name.to_string() }))]
    MetadataFileNotFound {
        /// The metadata file that was not found.
        name: MetadataFileName,
    },

    /// Reached the end of known entries while reading a package.
    #[error("{msg}", msg = t!("error-end-of-entries"))]
    EndOfPackageEntries,

    /// A package input directory is located inside of a package output directory.
    #[error("{msg}", msg = t!("error-output-dir-in-input-dir", {
        "input_path" => input_path,
        "output_path" => output_path
    }))]
    OutputDirInInputDir {
        /// The input directory path.
        input_path: PathBuf,
        /// The output directory path.
        output_path: PathBuf,
    },

    /// A [`crate::package::Error`].
    #[error("{msg}", msg = t!("error-package", { "source" => .0.to_string() }))]
    Package(#[from] crate::package::Error),

    /// A path does not exist.
    #[error("{msg}", msg = t!("error-path-not-exist", { "path" => path }))]
    PathDoesNotExist {
        /// The path that should exist.
        path: PathBuf,
    },

    /// A path does not have a parent.
    #[error("{msg}", msg = t!("error-path-no-parent", { "path" => path }))]
    PathHasNoParent {
        /// The path that should have a parent.
        path: PathBuf,
    },

    /// A path is not a file.
    #[error("{msg}", msg = t!("error-path-not-file", { "path" => path }))]
    PathIsNotAFile {
        /// The path that is not a file.
        path: PathBuf,
    },

    /// A path is read only.
    #[error("{msg}", msg = t!("error-path-read-only", { "path" => path }))]
    PathIsReadOnly {
        /// The path that is read only.
        path: PathBuf,
    },
}
