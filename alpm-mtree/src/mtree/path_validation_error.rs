//! Error handling related to path validation based on [`Mtree`] data.

use std::path::PathBuf;

use alpm_types::Sha256Checksum;

#[cfg(doc)]
use crate::Mtree;

/// A list of errors that may occur when comparing [`Mtree`] data with path in a `base_dir`.
///
/// Tracks a `base_dir` below which files are compared to [`Mtree`] data.
/// Also tracks a list of zero or more [`PathValidationError`]s that occurred when validating paths
/// below `base_dir` based on [`Mtree`] data.
///
/// After initialization, this struct is used to [`append`][`PathValidationErrors::append`] any
/// errors that have occurred during validation.
/// After validation (which is considered an infallible action), calling
/// [`fail`][`PathValidationErrors::fail`] returns an error if any errors have been collected during
/// validation.
#[derive(Debug, Default)]
pub struct PathValidationErrors {
    base_dir: PathBuf,
    pub errors: Vec<PathValidationError>,
}

impl PathValidationErrors {
    /// Creates a new [`PathValidationErrors`] for a directory.
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            errors: Vec::new(),
        }
    }

    /// Appends a list of [`PathValidationError`]s to `self.errors`.
    pub fn append(&mut self, other: &mut Vec<PathValidationError>) {
        self.errors.append(other);
    }

    /// Fail if there are any errors.
    ///
    /// # Errors
    ///
    /// Returns an error if one or more errors have been collected during validation.
    pub fn fail(&self) -> Result<(), crate::Error> {
        if !self.errors.is_empty() {
            return Err(crate::Error::PathValidation {
                path: self.base_dir.clone(),
                errors: self.errors.iter().fold(String::new(), |mut output, error| {
                    output.push_str(&format!("{error}\n"));
                    output
                }),
            });
        }

        Ok(())
    }
}

/// The error that can occur when comparing [`Mtree`] paths with paths on a file system.
#[derive(Debug, thiserror::Error)]
pub enum PathValidationError {
    /// Alpm-common error.
    #[error("Alpm-common error:\n{0}")]
    AlpmCommon(#[from] alpm_common::Error),

    /// Unable to create hash digest for a path.
    #[error("Unable to create hash digest for path {path:?}:\n{source}")]
    CreateHashDigest {
        /// The path for which a hash digest can not be created.
        path: PathBuf,
        /// The source error.
        source: std::io::Error,
    },

    /// The hash digest of a path in the ALPM-MTREE data does not match that of the corresponding
    /// on-disk file.
    #[error(
        "The hash digest of {mtree_path:?} in the ALPM-MTREE data is {mtree_digest}, but that of {path:?} is {path_digest}"
    )]
    PathDigestMismatch {
        /// The path in the ALPM-MTREE data that does not have a matching path on disk.
        mtree_path: PathBuf,
        /// The size of the path according to ALPM-MTREE data.
        mtree_digest: Sha256Checksum,
        /// The on-disk path, that does not match the size of the ALPM-MTREE data.
        path: PathBuf,
        /// The on-disk path, that does not match the size of the ALPM-MTREE data.
        path_digest: Sha256Checksum,
    },

    /// The GID of a path in the ALPM-MTREE metadata does not match that of the corresponding
    /// on-disk file.
    #[error(
        "The GID of {mtree_path:?} in the ALPM-MTREE data is {mtree_gid}, but that of path {path:?} is {path_gid}"
    )]
    PathGidMismatch {
        /// The path in the ALPM-MTREE data that has a differing GID from that of the on-disk file.
        mtree_path: PathBuf,
        /// The GID recorded in the ALPM-MTREE data for `mtree_path`.
        mtree_gid: u32,
        /// The on-disk path, that has a differing GID from the ALPM-MTREE data.
        path: PathBuf,
        /// The GID used for `path`.
        path_gid: u32,
    },

    /// The metadata for a path can not be retrieved.
    #[error("The metadata for path {path:?} can not be retrieved:\n{source}")]
    PathMetadata {
        /// The on-disk path for which metadata can not be retrieved.
        path: PathBuf,
        /// The source Error.
        source: std::io::Error,
    },

    /// A path does not match what it is supposed to be.
    #[error("The path {mtree_path:?} in the ALPM-MTREE data does not match the path {path:?}")]
    PathMismatch {
        /// The path in the ALPM-MTREE data that does not have a matching path on disk.
        mtree_path: PathBuf,
        /// The on-disk path, that does not match that of the ALPM-MTREE data.
        path: PathBuf,
    },

    /// An on-disk path does not exist.
    #[error(
        "The path {path:?} does not exist, but the path {mtree_path:?} in the ALPM-MTREE data requires it to."
    )]
    PathMissing {
        /// The path in the ALPM-MTREE data that does not have a matching path on disk.
        mtree_path: PathBuf,
        /// The absolute on-disk path, that does not exist.
        path: PathBuf,
    },

    /// The mode of a path in the ALPM-MTREE metadata does not match that of the corresponding
    /// on-disk file.
    #[error(
        "The mode of {mtree_path:?} in the ALPM-MTREE data is {mtree_mode}, but that of path {path:?} is {path_mode}"
    )]
    PathModeMismatch {
        /// The path in the ALPM-MTREE data that has a differing mode from that of the on-disk
        /// file.
        mtree_path: PathBuf,
        /// The mode recorded in the ALPM-MTREE data for `mtree_path`.
        mtree_mode: String,
        /// The on-disk path, that has a differing mode from that of the ALPM-MTREE data.
        path: PathBuf,
        /// The mode used for `path`.
        path_mode: String,
    },

    /// An on-disk path is not a directory.
    #[error(
        "The path {path:?} is not a directory, but the ALPM-MTREE data for {mtree_path:?} requires it to be."
    )]
    PathNotADir {
        /// The path in the ALPM-MTREE data requiring `path` to be a directory.
        mtree_path: PathBuf,
        /// The absolute on-disk path, that is not a directory.
        path: PathBuf,
    },

    /// An on-disk path is not a file.
    #[error(
        "The path {path:?} is not a file, but the ALPM-MTREE data for {mtree_path:?} requires it to be."
    )]
    PathNotAFile {
        /// The path in the ALPM-MTREE data requiring `path` to be a file.
        mtree_path: PathBuf,
        /// The absolute on-disk path, that is not a file.
        path: PathBuf,
    },

    /// An on-disk path is not a symlink.
    #[error(
        "The path {path:?} is not a symlink, but the ALPM-MTREE data for {mtree_path:?} requires it to be."
    )]
    PathNotASymlink {
        /// The path in the ALPM-MTREE data requiring `path` to be a symlink.
        mtree_path: PathBuf,
        /// The absolute on-disk path, that is not a symlink.
        path: PathBuf,
    },

    /// The size of a path in the ALPM-MTREE metadata does not match the size of the corresponding
    /// on-disk file.
    #[error(
        "The size of {mtree_path:?} in the ALPM-MTREE data is {mtree_size}, but that of path {path:?} is {path_size}"
    )]
    PathSizeMismatch {
        /// The path in the ALPM-MTREE data that does not have a matching path on disk.
        mtree_path: PathBuf,
        /// The size of the path according to ALPM-MTREE data.
        mtree_size: u64,
        /// The on-disk path, that does not match the size of the ALPM-MTREE data.
        path: PathBuf,
        /// The on-disk path, that does not match the size of the ALPM-MTREE data.
        path_size: u64,
    },

    /// A path does not match what it is supposed to be.
    #[error(
        "The symlink {mtree_path:?} in the ALPM-MTREE data points at {mtree_link_path:?}, while {path:?} points at {link_path:?}"
    )]
    PathSymlinkMismatch {
        /// The path in the ALPM-MTREE data that does not have a matching path on disk.
        mtree_path: PathBuf,
        /// The path in the ALPM-MTREE data that does not have a matching path on disk.
        mtree_link_path: PathBuf,
        /// The on-disk path, that does not match that of the ALPM-MTREE data.
        path: PathBuf,
        /// The on-disk path, that does not match that of the ALPM-MTREE data.
        link_path: PathBuf,
    },

    /// The time of a path in the ALPM-MTREE metadata does not match the time of the corresponding
    /// on-disk file.
    #[error(
        "The time of {mtree_path:?} in the ALPM-MTREE data is {mtree_time}, but that of path {path:?} is {path_time}"
    )]
    PathTimeMismatch {
        /// The path in the ALPM-MTREE data that does not have a matching path on disk.
        mtree_path: PathBuf,
        /// The size of the path according to ALPM-MTREE data.
        mtree_time: i64,
        /// The on-disk path, that does not match the size of the ALPM-MTREE data.
        path: PathBuf,
        /// The on-disk path, that does not match the size of the ALPM-MTREE data.
        path_time: i64,
    },

    /// The UID of a path in the ALPM-MTREE metadata does not match that of the corresponding
    /// on-disk file.
    #[error(
        "The UID of {mtree_path:?} in the ALPM-MTREE data is {mtree_uid}, but that of path {path:?} is {path_uid}"
    )]
    PathUidMismatch {
        /// The path in the ALPM-MTREE data that does not have a matching path on disk.
        mtree_path: PathBuf,
        /// The UID recorded in the ALPM-MTREE data.
        mtree_uid: u32,
        /// The on-disk path, that does not match the size of the ALPM-MTREE data.
        path: PathBuf,
        /// The UID used for `path`.
        path_uid: u32,
    },

    /// Unable to read a link.
    #[error("Unable to read link of symlink {path:?}:\n{source}")]
    ReadLink {
        /// The path for which the link can not be read.
        path: PathBuf,
        /// The source error
        source: std::io::Error,
    },

    /// There are file system paths for which no matching ALPM-MTREE paths exist.
    #[error(
        "There are no matching ALPM-MTREE paths for the following file system paths:\n{}",
        paths.iter().fold(String::new(), |mut output, path| {
            output.push_str(&format!("{path:?}\n"));
            output})
    )]
    UnmatchedFileSystemPaths {
        /// The list of file system paths for which no matching ALPM-MTREE paths exist.
        paths: Vec<PathBuf>,
    },

    /// There are ALPM-MTREE paths for which no matching file system paths exist.
    #[error(
        "There are no matching file system paths for the following ALPM-MTREE paths:\n{}",
        paths.iter().fold(String::new(), |mut output, path| {
            output.push_str(&format!("{path:?}\n"));
            output})
    )]
    UnmatchedMtreePaths {
        /// The list of ALPM-MTREE paths for which no matching file system paths exist.
        paths: Vec<PathBuf>,
    },
}
