//! Errors related to package sources, contents and files.

#[cfg(doc)]
use crate::MetadataFileName;

/// The error that can occur when handling types related to package data.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    /// A string is not a valid [`MetadataFileName`].
    #[error("Invalid package metadata file name: {name}")]
    InvalidMetadataFilename {
        /// The invalid file name.
        name: String,
    },
}
