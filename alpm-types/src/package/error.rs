//! Errors related to package sources, contents and files.

use std::path::PathBuf;

use fluent_i18n::t;

use crate::Version;
#[cfg(doc)]
use crate::{MetadataFileName, PackageFileName};

/// The error that can occur when handling types related to package data.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    /// A string is not a valid [`MetadataFileName`].
    #[error("{msg}", msg = t!("error-invalid-metadata-filename", { "name" => name }))]
    InvalidMetadataFilename {
        /// The invalid file name.
        name: String,
    },

    /// A path is not a valid [`PackageFileName`].
    #[error("{msg}", msg = t!("error-invalid-package-filename-path", { "path" => path }))]
    InvalidPackageFileNamePath {
        /// The file path that is not valid.
        path: PathBuf,
    },

    /// A version is not valid for an [`PackageFileName`].
    #[error("{msg}", msg = t!("error-invalid-package-filename-version", { "version" => version.to_string() }))]
    InvalidPackageFileNameVersion {
        /// The version that is not valid.
        version: Version,
    },
}
