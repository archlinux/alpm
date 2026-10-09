//! Error handling for [alpm-db] creation, access and writing.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::{fs::TryLockError, path::PathBuf};

use alpm_types::EntryName;
use fluent_i18n::t;
use uuid::Uuid;

use crate::db::{Database, Entry, entry::ALPM_DB_ENTRY_COMPONENT_FILE_MODE};

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
    #[error("{msg}", msg = t!("error-database-exclusive-lock-file-failed", { "path" => .path, "source" => .source.to_string() }))]
    DatabaseFailedExclusiveLockFile {
        /// The path of the database lock file that cannot be locked.
        path: PathBuf,

        /// The error source.
        source: TryLockError,
    },

    /// No write lock is provided when attempting to run or continue a transaction.
    #[error(
        "{msg}",
        msg = t!("error-database-transaction-misses-lock", {
            "path" => .path.display().to_string(),
            "uuid" => .uuid.to_string()
        })
    )]
    DatabaseTransactionMissesLock {
        /// The path of the database, that has no write lock.
        path: PathBuf,

        /// The UUID of the transaction.
        uuid: Uuid,
    },

    /// The permissions for the root directory of an [alpm-db] differ from the defaults.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error(
        "{msg}",
        msg = t!("error-database-wrong-permissions", {
            "path" => .path.display().to_string(),
            "defaults" => format!("{:o}", Database::DEFAULT_DIR_PERMISSIONS | 0o777),
            "permissions" => format!("{:o}", .permissions | 0o777),
        })
    )]
    DatabaseWrongPermissions {
        /// The path of the database, that has wrong permissions.
        path: PathBuf,

        /// The permissions, that do not align with the defaults.
        permissions: u32,
    },

    /// A component (`desc`, `files`, `install` or `mtree`) in an [alpm-db] entry is not a file.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error("{msg}", msg = t!("error-database-entry-component-is-not-a-file", { "path" => .path }))]
    EntryComponentIsNotAFile {
        /// The path of the entry component, that is not a file.
        path: PathBuf,
    },

    /// The [`EntryName`] of an [alpm-db] entry and its [alpm-db-desc] component do not match.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    #[error(
        "{msg}",
        msg = t!("error-database-entry-component-name-mismatch", {
            "entry_name" => .entry_name.to_string(),
            "component_entry_name" => .component_entry_name.to_string(),
        })
    )]
    EntryComponentNameMismatch {
        /// The entry name of the entry.
        entry_name: String,

        /// The entry name of the entry component.
        component_entry_name: String,
    },

    /// The file name for an entry component in the [alpm-db] is wrong.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error(
        "{msg}",
        msg = t!("error-database-entry-component-wrong-file-name", {
            "path" => .path.display().to_string(),
            "expected" => .expected,
        })
    )]
    EntryComponentWrongFileName {
        /// The path of the entry component, that has a wrong file name.
        path: PathBuf,

        /// The expected file name.
        expected: &'static str,
    },

    /// The permissions for an entry component in the [alpm-db] differ from the defaults.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error(
        "{msg}",
        msg = t!("error-database-entry-component-wrong-permissions", {
            "path" => .path.display().to_string(),
            "defaults" => format!("{:o}", ALPM_DB_ENTRY_COMPONENT_FILE_MODE),
            "permissions" => format!("{:o}", .permissions),
        })
    )]
    EntryComponentWrongPermissions {
        /// The path of the entry component, that has wrong permissions.
        path: PathBuf,

        /// The permissions, that do not align with the defaults.
        permissions: u32,
    },

    /// The permissions for an entry directory in the [alpm-db] differ from the defaults.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error(
        "{msg}",
        msg = t!("error-database-entry-dir-wrong-permissions", {
            "path" => .path.display().to_string(),
            "defaults" => format!("{:o}", Entry::ENTRY_DIR_MODE),
            "permissions" => format!("{:o}", .permissions),
        })
    )]
    EntryDirWrongPermissions {
        /// The path of the entry directory, that has wrong permissions.
        path: PathBuf,

        /// The permissions, that do not align with the defaults.
        permissions: u32,
    },

    /// An entry of an [alpm-db] exists already.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error("{msg}", msg = t!("error-database-entry-already-exists", { "name" => .name.to_string() }))]
    EntryExists {
        /// The name of the entry, that exists already.
        name: EntryName,
    },

    /// An entry of an [alpm-db] does not exist.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error("{msg}", msg = t!("error-database-entry-does-not-exist", { "name" => .name.to_string() }))]
    EntryDoesNotExist {
        /// The name of the entry, that does not exist.
        name: EntryName,
    },

    /// An [alpm-db] entry path is not a directory.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error("{msg}", msg = t!("error-database-entry-is-not-a-directory", { "path" => .path.display().to_string() }))]
    EntryIsNotADirectory {
        /// The path of the database entry, that is not a directory.
        path: PathBuf,
    },

    /// A transaction was attempted to be created without any actions.
    #[error("{msg}", msg = t!("error-database-transaction-no-action", { "uuid" => .uuid.to_string() }))]
    TransactionHasNoAction {
        /// The UUID of the transaction.
        uuid: Uuid,
    },
}
