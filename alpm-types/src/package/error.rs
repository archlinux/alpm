//! Errors related to package sources, contents and files.

use std::path::PathBuf;

use rust_i18n::t;

use crate::Version;
#[cfg(doc)]
use crate::{MetadataFileName, PackageFileName};

/// The error that can occur when handling types related to package data.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    /// A string is not a valid [`MetadataFileName`].
    #[error("{msg}: {name}", msg = t!("error.package.invalid_metadata_name"))]
    InvalidMetadataFilename {
        /// The invalid file name.
        name: String,
    },

    /// A path is not a valid [`PackageFileName`].
    #[error("{msg}: {path:?}", msg = t!("error.package.invalid_package_file_name"))]
    InvalidPackageFileNamePath {
        /// The file path that is not valid.
        path: PathBuf,
    },

    /// A path is not a valid [`PackageFileName`].
    #[error("{msg}: \"{version}\"", msg = t!("error.package.invalid_version"))]
    InvalidPackageFileNameVersion {
        /// The version that is not valid.
        version: Version,
    },
}
