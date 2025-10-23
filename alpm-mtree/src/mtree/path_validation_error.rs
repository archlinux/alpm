//! Error handling related to path validation based on [`Mtree`] data.

use std::{fmt::Display, path::PathBuf};

use alpm_types::Sha256Checksum;
use fluent_i18n::t;

#[cfg(doc)]
use crate::Mtree;

/// A list of errors that may occur when comparing [`Mtree`] data with paths inside a `base_dir`.
///
/// Tracks a `base_dir` whose files are compared to [`Mtree`] data.
/// Also tracks a list of zero or more [`PathValidationError`]s that occurred when validating paths
/// inside `base_dir` by comparing it with [`Mtree`] data.
///
/// After initialization, [`append`][`PathValidationErrors::append`] can be used to add any
/// errors to this struct that occurred during validation.
/// After validation (which is considered an infallible action), calling
/// [`check`][`PathValidationErrors::check`] returns an error if any errors have been collected
/// during validation.
#[derive(Debug, Default, thiserror::Error)]
pub struct PathValidationErrors {
    base_dir: PathBuf,
    errors: Vec<PathValidationError>,
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

    /// Checks if errors have been appended and consumes `self`.
    ///
    /// # Errors
    ///
    /// Returns an error if one or more errors have been appended.
    pub fn check(self) -> Result<(), crate::Error> {
        if !self.errors.is_empty() {
            return Err(crate::Error::PathValidation(self));
        }

        Ok(())
    }
}

impl Display for PathValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            t!("error-path-validation-errors", {
                "base_dir" => self.base_dir.display().to_string(),
                "errors" => self
                    .errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        )
    }
}

/// The error that can occur when comparing [`Mtree`] paths with paths on a file system.
#[derive(Debug, thiserror::Error)]
pub enum PathValidationError {
    /// Alpm-common error.
    #[error("{msg}", msg = t!("error-alpm-common", { "source" => .0.to_string() }))]
    AlpmCommon(#[from] alpm_common::Error),

    /// Unable to create hash digest for a path.
    #[error("{msg}", msg = t!("error-create-hash-digest", {
        "path" => path.display().to_string(),
        "source" => source.to_string()
    }))]
    CreateHashDigest {
        /// The path for which a hash digest can not be created.
        path: PathBuf,
        /// The source error.
        source: std::io::Error,
    },

    /// The hash digest of a path in the ALPM-MTREE data does not match that of the corresponding
    /// on-disk file.
    #[error("{msg}", msg = t!("error-path-digest-mismatch", {
        "mtree_path" => mtree_path.display().to_string(),
        "mtree_digest" => mtree_digest.to_string(),
        "path" => path.display().to_string(),
        "path_digest" => path_digest.to_string()
    }))]
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
    #[error("{msg}", msg = t!("error-path-gid-mismatch", {
        "mtree_path" => mtree_path.display().to_string(),
        "mtree_gid" => mtree_gid.to_string(),
        "path" => path.display().to_string(),
        "path_gid" => path_gid.to_string()
    }))]
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
    #[error("{msg}", msg = t!("error-path-metadata", {
        "path" => path.display().to_string(),
        "source" => source.to_string()
    }))]
    PathMetadata {
        /// The on-disk path for which metadata can not be retrieved.
        path: PathBuf,
        /// The source Error.
        source: std::io::Error,
    },

    /// A path does not match what it is supposed to be.
    #[error("{msg}", msg = t!("error-path-mismatch", {
        "mtree_path" => mtree_path.display().to_string(),
        "path" => path.display().to_string()
    }))]
    PathMismatch {
        /// The path in the ALPM-MTREE data that does not have a matching path on disk.
        mtree_path: PathBuf,
        /// The on-disk path, that does not match that of the ALPM-MTREE data.
        path: PathBuf,
    },

    /// An on-disk path does not exist.
    #[error("{msg}", msg = t!("error-path-missing", {
        "mtree_path" => mtree_path.display().to_string(),
        "path" => path.display().to_string()
    }))]
    PathMissing {
        /// The path in the ALPM-MTREE data that does not have a matching path on disk.
        mtree_path: PathBuf,
        /// The absolute on-disk path, that does not exist.
        path: PathBuf,
    },

    /// The mode of a path in the ALPM-MTREE metadata does not match that of the corresponding
    /// on-disk file.
    #[error("{msg}", msg = t!("error-path-mode-mismatch", {
        "mtree_path" => mtree_path.display().to_string(),
        "mtree_mode" => mtree_mode,
        "path" => path.display().to_string(),
        "path_mode" => path_mode
    }))]
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
    #[error("{msg}", msg = t!("error-path-not-a-dir", {
        "mtree_path" => mtree_path.display().to_string(),
        "path" => path.display().to_string()
    }))]
    PathNotADir {
        /// The path in the ALPM-MTREE data requiring `path` to be a directory.
        mtree_path: PathBuf,
        /// The absolute on-disk path, that is not a directory.
        path: PathBuf,
    },

    /// An on-disk path is not a file.
    #[error("{msg}", msg = t!("error-path-not-a-file", {
        "mtree_path" => mtree_path.display().to_string(),
        "path" => path.display().to_string()
    }))]
    PathNotAFile {
        /// The path in the ALPM-MTREE data requiring `path` to be a file.
        mtree_path: PathBuf,
        /// The absolute on-disk path, that is not a file.
        path: PathBuf,
    },

    /// The size of a path in the ALPM-MTREE metadata does not match the size of the corresponding
    /// on-disk file.
    #[error("{msg}", msg = t!("error-path-size-mismatch", {
        "mtree_path" => mtree_path.display().to_string(),
        "mtree_size" => mtree_size.to_string(),
        "path" => path.display().to_string(),
        "path_size" => path_size.to_string()
    }))]
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
    #[error("{msg}", msg = t!("error-path-symlink-mismatch", {
        "mtree_path" => mtree_path.display().to_string(),
        "mtree_link_path" => mtree_link_path.display().to_string(),
        "path" => path.display().to_string(),
        "link_path" => link_path.display().to_string()
    }))]
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
    #[error("{msg}", msg = t!("error-path-time-mismatch", {
        "mtree_path" => mtree_path.display().to_string(),
        "mtree_time" => mtree_time.to_string(),
        "path" => path.display().to_string(),
        "path_time" => path_time.to_string()
    }))]
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
    #[error("{msg}", msg = t!("error-path-uid-mismatch", {
        "mtree_path" => mtree_path.display().to_string(),
        "mtree_uid" => mtree_uid.to_string(),
        "path" => path.display().to_string(),
        "path_uid" => path_uid.to_string()
    }))]
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
    #[error("{msg}", msg = t!("error-read-link", {
        "path" => path.display().to_string(),
        "mtree_path" => mtree_path.display().to_string(),
        "source" => source.to_string()
    }))]
    ReadLink {
        /// The path for a symlink on the file system which can not be read.
        path: PathBuf,
        /// The path in the ALPM-MTREE data that does not have a matching path on disk.
        mtree_path: PathBuf,
        /// The source error
        source: std::io::Error,
    },

    /// There are file system paths for which no matching ALPM-MTREE paths exist.
    #[error("{msg}\n", msg = t!("error-unmatched-fs-paths", {
        "paths" => paths.iter().map(|p| format!("{p:?}")).collect::<Vec<_>>().join("\n")
    }))]
    UnmatchedFileSystemPaths {
        /// The list of file system paths for which no matching ALPM-MTREE paths exist.
        paths: Vec<PathBuf>,
    },

    /// There are ALPM-MTREE paths for which no matching file system paths exist.
    #[error("{msg}", msg = t!("error-unmatched-mtree-paths", {
        "paths" => paths.iter().map(|p| format!("{p:?}")).collect::<Vec<_>>().join("\n")
    }))]
    UnmatchedMtreePaths {
        /// The list of ALPM-MTREE paths for which no matching file system paths exist.
        paths: Vec<PathBuf>,
    },
}
