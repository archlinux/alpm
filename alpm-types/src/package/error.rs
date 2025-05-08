//! Errors related to package sources, contents and files.

use std::path::PathBuf;

use crate::Version;
#[cfg(doc)]
use crate::{MetadataFileName, PackageFileName};

/// The error that can occur when handling types related to package data.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    /// A string is not a valid [`MetadataFileName`].
    #[error("Invalid package metadata file name: {name}")]
    InvalidMetadataFilename {
        /// The invalid file name.
        name: String,
    },

    /// A path is not a valid [`PackageFileName`].
    #[error("The path {path:?} is not a valid alpm-package file name")]
    InvalidPackageFileNamePath {
        /// The file path that is not valid.
        path: PathBuf,
    },

    /// A path is not a valid [`PackageFileName`].
    #[error("The version \"{version}\" is not valid for an alpm-package file name")]
    InvalidPackageFileNameVersion {
        /// The version that is not valid.
        version: Version,
    },
}
