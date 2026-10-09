//! Error handling for [alpm-db] creation, access and writing.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::{fs::TryLockError, path::PathBuf};

use fluent_i18n::t;
use uuid::Uuid;

use crate::db::Database;

/// The error that can occur when accessing or writing an [alpm-db].
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No write lock is provided when attempting to create a new [alpm-db].
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error(
        "{msg}",
        msg = t!("error-database-creation-misses-lock", {
            "path" => .path.display().to_string(),
        })
    )]
    DatabaseCreationMissesLock {
        /// The path of the database, that has no write lock.
        path: PathBuf,
    },

    /// Encountered a non-empty directory when attempting to create a new [alpm-db].
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error(
        "{msg}",
        msg = t!("error-database-creation-not-empty", {
            "path" => .path.display().to_string(),
        })
    )]
    DatabaseCreationNotEmpty {
        /// The path of the database directory, that is not empty.
        path: PathBuf,
    },

    /// An [alpm-db] is locked.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error(
        "{msg}",
        msg = t!("error-database-is-locked", {
            "path" => .path.display().to_string(),
            "lock_file" => .lock_file.display().to_string(),
        })
    )]
    DatabaseIsLocked {
        /// The path of the database, that is locked.
        path: PathBuf,

        /// The path of lock file for the database.
        lock_file: PathBuf,
    },

    /// An [alpm-db] root path is not a directory.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error("{msg}", msg = t!("error-database-root-is-not-a-directory", { "path" => .path.display().to_string() }))]
    DatabaseIsNotADirectory {
        /// The root path of the database, that is not a directory.
        path: PathBuf,
    },

    /// A file lock for the [alpm-db] lock file cannot be created.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error("{msg}", msg = t!("error-database-file-lock-for-lock-file", { "path" => .path, "source" => .source.to_string() }))]
    DatabaseLockLockFile {
        /// The path of the database lock file that cannot be locked.
        path: PathBuf,

        /// The error source.
        source: TryLockError,
    },

    /// The permissions for the root directory of an [alpm-db] differ from the defaults
    /// ([`Database::DEFAULT_DIR_PERMISSIONS`]).
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error(
        "{msg}",
        msg = t!("error-database-wrong-permissions", {
            "path" => .path.display().to_string(),
            "defaults" => format!("{:o}", Database::DEFAULT_DIR_PERMISSIONS),
            "permissions" => format!("{:o}", .permissions),
        })
    )]
    DatabaseWrongPermissions {
        /// The path of the database, that has wrong permissions.
        path: PathBuf,

        /// The permissions, that do not align with the defaults.
        permissions: u32,
    },

    /// A file component in an [alpm-db] entry is not a file.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error("{msg}", msg = t!("error-database-entry-component-is-not-a-file", { "path" => .path }))]
    EntryComponentIsNotAFile {
        /// The path of the entry, that is not a directory.
        path: PathBuf,

        /// The file name of the component.
        file_name: &'static str,
    },

    /// A transaction was attempted to be created without any actions.
    #[error("{msg}", msg = t!("error-database-transaction-no-action", { "uuid" => .uuid.to_string() }))]
    TransactionHasNoAction {
        /// The UUID of the transaction.
        uuid: Uuid,
    },
}
