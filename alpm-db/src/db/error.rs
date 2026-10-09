//! Error handling for [alpm-db] creation, access and writing.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::{fs::TryLockError, path::PathBuf};

use alpm_types::EntryName;
use fluent_i18n::t;
use uuid::Uuid;

use crate::db::DIR_MODE;

/// The error that can occur when accessing or writing an [alpm-db].
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
#[derive(Debug, thiserror::Error)]
pub enum Error {
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

    /// An [alpm-db] has no transaction.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error(
        "{msg}",
        msg = t!("error-database-has-no-transaction", {
            "path" => .path.display().to_string(),
        })
    )]
    DatabaseHasNoTransaction {
        /// The path of the database.
        path: PathBuf,
    },

    /// An [alpm-db] has a transaction and a write lock already.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error(
        "{msg}",
        msg = t!("error-database-has-locked-transaction-already", {
            "path" => .path.display().to_string(),
            "lock_file" => .lock_file.display().to_string(),
            "uuid" => .uuid.to_string(),
        })
    )]
    DatabaseHasLockedTransactionAlready {
        /// The path of the database.
        path: PathBuf,

        /// The path of the lock file for the database.
        lock_file: PathBuf,

        /// The UUID of the transaction.
        uuid: Uuid,
    },

    /// An [alpm-db] has a transaction already, but no write lock.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error(
        "{msg}",
        msg = t!("error-database-has-transaction-but-no-lock", {
            "path" => .path.display().to_string(),
            "uuid" => .uuid.to_string(),
        })
    )]
    DatabaseHasTransactionButNoLock {
        /// The path of the database.
        path: PathBuf,

        /// The UUID of the transaction.
        uuid: Uuid,
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

    /// An [alpm-db] is locked.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error(
        "{msg}",
        msg = t!("error-database-is-locked-but-no-transaction", {
            "path" => .path.display().to_string(),
            "lock_file" => .lock_file.display().to_string(),
        })
    )]
    DatabaseIsLockedButNoTransaction {
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

    /// The permissions for the root directory of an [alpm-db] differ from the defaults.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error(
        "{msg}",
        msg = t!("error-database-wrong-permissions", {
            "path" => .path.display().to_string(),
            "defaults" => format!("{:o}", DIR_MODE & 0o777),
            "permissions" => format!("{:o}", .permissions & 0o777),
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

    /// An [alpm-db] transaction data directory path is not a directory.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error("{msg}", msg = t!("error-transaction-data-dir-is-not-a-directory", { "path" => .path.display().to_string() }))]
    TransactionDataDirIsNotADirectory {
        /// The root path of the database, that is not a directory.
        path: PathBuf,
    },

    /// One or more errors occurred while preparing the temporary data for an [alpm-db] transaction.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error(
        "{msg}",
        msg = t!("error-transaction-temporary-data-errors",{
            "path" => .path.display().to_string(),
            "uuid" => .uuid.to_string(),
            "errors" => .errors.join("\n\n"),
        })
    )]
    TransactionTemporaryDataErrors {
        /// The root path of the database.
        path: PathBuf,

        /// The transaction UUID.
        uuid: Uuid,

        /// The string representation of the errors, that occurred.
        errors: Vec<String>,
    },
}
