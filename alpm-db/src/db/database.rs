//! Management of the [alpm-db] directory structure.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::{
    fs::{File, OpenOptions, create_dir_all, remove_file, set_permissions, symlink_metadata},
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt, chown},
    path::{Path, PathBuf},
};

use alpm_common::FileFormatSchema;
use alpm_mtree::Mtree;
use alpm_types::InstalledPackage;
use fluent_i18n::t;
use log::{debug, trace, warn};
use nix::unistd::{getegid, geteuid};

use crate::{
    db::{
        Action,
        DIR_MODE,
        DbSchema,
        Entry,
        Error,
        Transaction,
        TransactionPlan,
        permissions_world_writable,
    },
    desc::DbDescFile,
    files::DbFiles,
};

/// A file-based lock to prevent concurrent access to a database.
#[derive(Debug)]
struct DatabaseLock {
    /// The path to the lock file.
    path: PathBuf,

    /// The underlying file handle.
    file: File,
}

impl DatabaseLock {
    /// The default file mode of the lock file.
    const DEFAULT_FILE_MODE: u32 = 0o100000;

    /// Creates a new [`DatabaseLock`] at a specific `path`.
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - the lock file cannot be created at `path` (e.g. because it already exists)
    /// - exclusive access to the file cannot be established
    pub fn new(path: impl AsRef<Path>) -> Result<Self, crate::Error> {
        let path = path.as_ref();
        debug!("Create an alpm-db lock file at {path:?}");

        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(Self::DEFAULT_FILE_MODE)
            .open(path)
            .map_err(|source| crate::Error::IoPath {
                path: path.to_path_buf(),
                context: t!("error-io-path-db-lock-create"),
                source,
            })?;
        file.try_lock()
            .map_err(|source| Error::DatabaseFailedExclusiveLockFile {
                path: path.to_path_buf(),
                source,
            })?;

        Ok(Self {
            path: path.to_path_buf(),
            file,
        })
    }

    /// Releases and removes the lock file and consumes [`Self`].
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - the lock file cannot be released
    /// - the lock file cannot be deleted
    pub fn remove(self) -> Result<(), crate::Error> {
        self.file.unlock().map_err(|source| crate::Error::IoPath {
            path: self.path.clone(),
            context: t!("error-io-path-db-lock-release"),
            source,
        })?;

        remove_file(&self.path).map_err(|source| crate::Error::IoPath {
            path: self.path.clone(),
            context: t!("error-io-path-db-lock-delete"),
            source,
        })?;

        Ok(())
    }
}

impl AsRef<Path> for DatabaseLock {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

/// Fails if a `path` exists.
///
/// # Errors
///
/// Returns an error, if a (lock) file exists at `path`.
fn fail_if_lock_exists(database_config: &DatabaseConfig) -> Result<(), crate::Error> {
    if database_config.lock_file.exists() {
        return Err(Error::DatabaseIsLocked {
            path: database_config.lock_file.clone(),
            lock_file: database_config.lock_file.clone(),
        }
        .into());
    }

    Ok(())
}

/// A configuration for transactions and the various locations that they rely on.
#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    /// The path to the alpm-db root directory.
    database_dir: PathBuf,

    /// The path of the lock file for the transaction/database.
    lock_file: PathBuf,

    /// The data directory for current and past transactions.
    ///
    /// Below this directory the following data is kept:
    ///
    /// - current transaction plan
    /// - past transaction plans
    /// - temporary data for current transaction plan
    transaction_data_dir: PathBuf,
}

impl DatabaseConfig {
    /// The default pacman database location.
    pub const PACMAN_DATABASE_DIR: &str = "/var/lib/pacman/local";

    /// The default pacman database lock file location.
    pub const PACMAN_DATABASE_LOCK_FILE: &str = "/var/lib/pacman/db.lck";

    /// The default location for transaction plans and temporary data for transactions.
    pub const TRANSACTION_DATA_DIR: &str = "/var/lib/alpm/alpm-db";

    /// Creates a new [`DatabaseConfig`].
    pub fn new(
        database_dir: PathBuf,
        lock_file: PathBuf,
        transaction_data_dir: PathBuf,
    ) -> Result<Self, crate::Error> {
        Ok(Self {
            database_dir,
            lock_file,
            transaction_data_dir,
        })
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_dir: PathBuf::from(Self::PACMAN_DATABASE_DIR),
            lock_file: PathBuf::from(Self::PACMAN_DATABASE_LOCK_FILE),
            transaction_data_dir: PathBuf::from(Self::TRANSACTION_DATA_DIR),
        }
    }
}

/// The representation of an [alpm-db].
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
#[derive(Debug)]
pub struct Database {
    /// The configuration for the interaction with this database.
    config: DatabaseConfig,

    /// The schema of the database.
    ///
    /// See [`DbSchema`] variants for available schema versions.
    schema: DbSchema,

    /// All available entries, serving as the the view onto the database's state.
    entries: Vec<Entry>,

    /// A single, optional transaction for write operations.
    transaction: Option<Transaction>,

    /// The write lock for a transaction.
    write_lock: Option<DatabaseLock>,
}

impl Database {
    /// Creates a new [`Database`] for an existing [alpm-db] at `path`.
    ///
    /// # Note
    ///
    /// The `path` denotes the [`Database`] root directory.
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - `path` cannot be canonicalized
    /// - the metadata for `path` cannot be retrieved
    /// - `path` is not a directory
    /// - `path` does not use the default file permissions
    /// - the database schema cannot be read
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    pub fn new(mut config: DatabaseConfig) -> Result<Self, crate::Error> {
        debug!("Open alpm-db: {:?}", config.database_dir);

        let config = {
            let path =
                config
                    .database_dir
                    .canonicalize()
                    .map_err(|source| crate::Error::IoPath {
                        path: config.database_dir.clone(),
                        context: t!("error-io-path-db-base-canonicalize"),
                        source,
                    })?;
            config.database_dir = path;
            config
        };

        fail_if_lock_exists(&config)?;

        // Fail, if the targeted path is not a directory.
        let metadata =
            symlink_metadata(&config.database_dir).map_err(|source| crate::Error::IoPath {
                path: config.database_dir.clone(),
                context: t!("error-io-path-metadata"),
                source,
            })?;
        if !metadata.is_dir() {
            return Err(Error::DatabaseIsNotADirectory {
                path: config.database_dir.clone(),
            }
            .into());
        }

        // Emit a warning if the directory is world-writable.
        if permissions_world_writable(metadata.permissions().mode()) {
            warn!(
                "{}",
                t!("warn-database-root-world-writable", { "path" => config.database_dir.display().to_string() })
            );
        }

        let schema =
            DbSchema::derive_from_file(config.database_dir.join(DbSchema::VERSION_FILE_NAME))?;

        let database = {
            let mut database = Self {
                config,
                schema,
                entries: Vec::new(),
                transaction: None,
                write_lock: None,
            };

            for path in database.paths()? {
                database.entries.push(Entry::new(path)?)
            }

            database
        };

        Ok(database)
    }

    /// Creates a new [`Database`] by creating a new, empty [alpm-db] structure at `path`.
    ///
    /// # Note
    ///
    /// If the targeted [`Database`] root directory (see [`Self::new`]) exists already, it must be
    /// empty!
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - the targeted path exists, but it cannot be canonicalized
    /// - the targeted path exists and is not a directory
    /// - the targeted directory exists and is not empty
    /// - the targeted directory exists and permissions for it cannot be adjusted
    /// - the targeted path does not exist yet and the directory or any of its parents cannot be
    ///   created
    /// - setting the default file permissions for the newly created directory fails
    /// - writing the database version file fails
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    pub fn create(mut config: DatabaseConfig, schema: DbSchema) -> Result<Self, crate::Error> {
        debug!(
            "Create new alpm-db with schema {}: {:?}",
            schema, config.database_dir
        );

        let path = if config.database_dir.exists() {
            // If the path exists already, canonicalize it and ensure, that it is valid.
            // The path must be an empty directory with default permissions, owned by the
            // UID/GID of the calling process.

            let path =
                config
                    .database_dir
                    .canonicalize()
                    .map_err(|source| crate::Error::IoPath {
                        path: config.database_dir,
                        context: t!("error-io-path-db-base-canonicalize"),
                        source,
                    })?;

            // Fail, if the targeted path is not a directory.
            if !&path.is_dir() {
                return Err(Error::DatabaseIsNotADirectory { path }.into());
            }

            // Fail, if the targeted directory exists and is not empty.
            if path
                .read_dir()
                .map_err(|source| crate::Error::IoPath {
                    path: path.clone(),
                    context: t!("error-io-database-read-dir"),
                    source,
                })?
                .count()
                != 0
            {
                return Err(Error::DatabaseCreationNotEmpty { path }.into());
            }

            let euid = geteuid().as_raw();
            let egid = getegid().as_raw();
            let metadata = path
                .symlink_metadata()
                .map_err(|source| crate::Error::IoPath {
                    path: path.clone(),
                    context: t!("error-io-path-metadata"),
                    source,
                })?;

            // Change ownership if it doesn't match the calling UID/GID.
            if !(metadata.uid() == euid && metadata.gid() == egid) {
                chown(&path, Some(euid), Some(egid)).map_err(|source| crate::Error::IoPath {
                    path: path.clone(),
                    context: t!("error-io-path-chown"),
                    source,
                })?;
            }

            // Change permissions if they don't match the defaults.
            if metadata.permissions().mode() != DIR_MODE {
                set_permissions(&path, PermissionsExt::from_mode(DIR_MODE)).map_err(|source| {
                    crate::Error::IoPath {
                        path: path.clone(),
                        context: t!("error-io-path-db-base-create"),
                        source,
                    }
                })?;
            }

            path
        } else {
            // If the path does not yet exist, create it, canonicalize it and set default
            // permissions.

            create_dir_all(&config.database_dir).map_err(|source| crate::Error::IoPath {
                path: config.database_dir.clone(),
                context: t!("error-io-path-db-base-create"),
                source,
            })?;
            let path =
                config
                    .database_dir
                    .canonicalize()
                    .map_err(|source| crate::Error::IoPath {
                        path: config.database_dir,
                        context: t!("error-io-path-db-base-canonicalize"),
                        source,
                    })?;

            set_permissions(&path, PermissionsExt::from_mode(DIR_MODE)).map_err(|source| {
                crate::Error::IoPath {
                    path: path.clone(),
                    context: t!("error-io-path-db-base-create"),
                    source,
                }
            })?;

            path
        };

        config.database_dir = path;

        schema.write_version_file_to_db_root(&config.database_dir)?;

        Ok(Self {
            config,
            schema,
            entries: Vec::new(),
            transaction: None,
            write_lock: None,
        })
    }

    /// Returns a reference to the path of the database.
    pub fn path(&self) -> &Path {
        &self.config.database_dir
    }

    /// Returns a reference to the [`DbSchema`] of the database.
    pub fn schema(&self) -> &DbSchema {
        &self.schema
    }

    /// Initializes a transaction for this database.
    ///
    /// First, a new lock file is created in the location defined by the
    /// [`DatabaseConfig`] passed to the [`Database`] upon creation.
    /// Then, the transaction is created, using the data directory defined by the
    /// [`DatabaseConfig`] passed to the [`Database`] upon creation.
    ///
    /// # Note
    ///
    /// A transactions is required for all add and removal actions.
    /// After initializing a transaction, actions can be added to it with [`Self::add_action`].
    /// Finally, the transaction can be committed using [`Self::commit_transaction`].
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - a lock (file) exists in the location configured by the [`DatabaseConfig`]
    /// - the database already has a transaction
    /// - the write lock file cannot be created
    pub fn init_transaction(&mut self) -> Result<(), crate::Error> {
        match (self.transaction.as_ref(), self.write_lock.as_ref()) {
            (Some(transaction), Some(lock)) => {
                return Err(Error::DatabaseHasLockedTransactionAlready {
                    path: self.config.database_dir.clone(),
                    lock_file: lock.path.to_path_buf(),
                    uuid: *transaction.uuid(),
                }
                .into());
            }
            (None, Some(lock)) => {
                return Err(Error::DatabaseIsLocked {
                    path: self.config.database_dir.clone(),
                    lock_file: lock.path.to_path_buf(),
                }
                .into());
            }
            (Some(_), None) | (None, None) => {}
        }

        self.write_lock = Some(DatabaseLock::new(&self.config.lock_file)?);
        let transaction =
            Transaction::new(&self.config.database_dir, &self.config.transaction_data_dir)?;
        // The transaction creation canonicalizes the data dir path.
        self.config.transaction_data_dir = transaction.data_dir().to_path_buf();
        self.transaction = Some(transaction);

        Ok(())
    }

    /// Adds an [`Action`] to the current transaction of the [`Database`].
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - the database is locked but has no transaction
    /// - the database has a transaction but no lock
    /// - the database has no transaction
    pub fn add_action(&mut self, action: Action) -> Result<(), crate::Error> {
        debug!("Add action: {action}");

        let mut transaction = match (self.transaction.take(), self.write_lock.as_ref()) {
            (Some(transaction), Some(_)) => transaction,
            (None, Some(lock)) => {
                return Err(Error::DatabaseIsLockedButNoTransaction {
                    path: self.config.database_dir.clone(),
                    lock_file: lock.path.to_path_buf(),
                }
                .into());
            }
            (Some(transaction), None) => {
                return Err(Error::DatabaseHasTransactionButNoLock {
                    path: self.config.database_dir.to_path_buf(),
                    uuid: *transaction.uuid(),
                }
                .into());
            }
            (None, None) => {
                return Err(Error::DatabaseHasNoTransaction {
                    path: self.config.database_dir.to_path_buf(),
                }
                .into());
            }
        };

        transaction.add_action(action);
        self.transaction = Some(transaction);

        Ok(())
    }

    /// Commits a transaction against the data of this database and consumes `Self`.
    ///
    /// After successful commit of the transaction, the write lock file is removed.
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - the database has no transaction
    /// - committing one of the action items of the transaction fails
    pub fn commit_transaction(mut self) -> Result<(), crate::Error> {
        let (transaction, lock) = match (self.transaction.take(), self.write_lock.take()) {
            (Some(transaction), Some(lock)) => (transaction, lock),
            (None, Some(lock)) => {
                return Err(Error::DatabaseIsLockedButNoTransaction {
                    path: self.config.database_dir.clone(),
                    lock_file: lock.path.to_path_buf(),
                }
                .into());
            }
            (Some(transaction), None) => {
                return Err(Error::DatabaseHasTransactionButNoLock {
                    path: self.config.database_dir.to_path_buf(),
                    uuid: *transaction.uuid(),
                }
                .into());
            }
            (None, None) => {
                return Err(Error::DatabaseHasNoTransaction {
                    path: self.config.database_dir.to_path_buf(),
                }
                .into());
            }
        };

        // Write the transaction plan to the directory for current transactions.
        let transaction_plan = TransactionPlan::from(&transaction);
        transaction_plan.write_to_current_dir()?;

        // Commit the data changes.
        transaction.commit()?;

        // Move the transaction plan from current to previous location in the data directory.
        transaction_plan.move_to_previous_dir()?;

        lock.remove()?;

        Ok(())
    }

    /// Returns the sorted list of all directory paths in the [`Database`].
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - an entry of the alpm-db cannot be read
    /// - the metadata of an entry in the alpm-db cannot be retrieved
    fn paths(&self) -> Result<Vec<PathBuf>, crate::Error> {
        debug!(
            "Collect the list of directory paths in the alpm-db: {:?}",
            self.config.database_dir
        );

        let mut paths = Vec::new();
        for entry in self
            .config
            .database_dir
            .read_dir()
            .map_err(|source| crate::Error::IoPath {
                path: self.config.database_dir.clone(),
                context: t!("error-io-database-read-dir"),
                source,
            })?
        {
            let dir_entry = entry.map_err(|source| crate::Error::IoPath {
                path: self.config.database_dir.clone(),
                context: t!("error-io-database-read-dir-entry"),
                source,
            })?;

            // Ignore non-directory paths.
            let metadata = dir_entry
                .metadata()
                .map_err(|source| crate::Error::IoPath {
                    path: self.config.database_dir.clone(),
                    context: t!("error-io-path-metadata"),
                    source,
                })?;
            if !metadata.is_dir() {
                trace!("Skip non-directory path in alpm-db: {:?}", dir_entry.path());
                continue;
            }

            paths.push(dir_entry.path());
        }
        paths.sort();

        Ok(paths)
    }

    /// Returns the list of all [`Entry`]s in the database.
    ///
    /// # Notes
    ///
    /// - Skips any non-directory entries.
    /// - Sorts the entries by name.
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - collecting all entries in the [alpm-db] fails
    /// - reading an [`Entry`] item fails
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    pub fn entries(&self) -> Vec<&Entry> {
        self.entries.iter().collect()
    }

    /// Returns the list of all [`DbDescFile`]s found in the database.
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - collecting all entries in the [alpm-db] fails
    /// - reading the [alpm-db-desc] component of an entry fails
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    /// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    pub fn db_descs(&mut self) -> Result<Vec<DbDescFile>, crate::Error> {
        let mut output = Vec::new();
        for entry in self.entries.iter_mut() {
            let data = entry.db_desc()?;
            output.push(data);
        }

        Ok(output)
    }

    /// Returns the list of all [`DbFiles`] found in the database.
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - collecting all entries in the [alpm-db] fails
    /// - reading the [alpm-db-files] component of an entry fails
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    /// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
    pub fn db_files(&mut self) -> Result<Vec<DbFiles>, crate::Error> {
        let mut output = Vec::new();
        for entry in self.entries.iter_mut() {
            let data = entry.db_files()?;
            output.push(data);
        }

        Ok(output)
    }

    /// Returns the list of all [`Mtree`] found in the database.
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - collecting all entries in the [alpm-db] fails
    /// - reading the [ALPM-MTREE] component of an entry fails
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    pub fn mtrees(&mut self) -> Result<Vec<Mtree>, crate::Error> {
        let mut output = Vec::new();
        for entry in self.entries.iter_mut() {
            let data = entry.mtree()?;
            output.push(data);
        }

        Ok(output)
    }

    /// Returns the list of all installed packages.
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - collecting all entries in the [alpm-db] fails
    /// - reading the [alpm-db-desc] component of an entry fails
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    /// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    pub fn installed_packages(&mut self) -> Result<Vec<InstalledPackage>, crate::Error> {
        let mut output = Vec::new();
        for entry in self.entries.iter_mut() {
            let data = entry.db_desc()?;
            output.push(data.installed_package());
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use log::LevelFilter;
    use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
    use tempfile::{NamedTempFile, TempDir};
    use testresult::TestResult;

    use super::*;

    /// Initialize a logger that shows trace messages on stderr.
    fn init_logger() {
        if TermLogger::init(
            LevelFilter::Trace,
            Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        )
        .is_err()
        {
            debug!("Not initializing another logger, as one is initialized already.");
        }
    }

    /// Ensures, that [`DatabaseLock::new`] succeeds on creating a lock file.
    ///
    /// Also checks, that the default file mode is applied and that the file is cleaned up on drop.
    #[test]
    fn database_lock_new_succeeds() -> TestResult {
        init_logger();
        let temp_dir = TempDir::new()?;
        let lock_file_path = temp_dir.path().join(PathBuf::from("db.lck"));

        let lock = DatabaseLock::new(&lock_file_path)?;
        let metadata = lock_file_path.metadata()?;
        assert_eq!(
            metadata.permissions().mode(),
            DatabaseLock::DEFAULT_FILE_MODE
        );

        lock.remove()?;

        assert!(!lock_file_path.exists());

        Ok(())
    }

    /// Ensures, that [`DatabaseLock::new`] fails on creating a lock file, if the target file exists
    /// already.
    #[test]
    fn database_lock_new_fails_on_existing_file() -> TestResult {
        init_logger();
        let temp_file = NamedTempFile::new()?;

        let Err(error) = DatabaseLock::new(temp_file.path()) else {
            panic!("Expected to fail with crate::Error::IoPath");
        };

        debug!("{error}");
        assert!(matches!(error, crate::Error::IoPath { .. }));

        Ok(())
    }

    /// Ensures, that [`Database::create`] succeeds on a new directory.
    #[test]
    fn database_create_succeeds_with_new_dir() -> TestResult {
        init_logger();
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        let database_dir = temp_path.join("db/local");
        let lock_file = temp_path.join("db/db.lck");
        let transaction_data_dir = temp_path.join("alpm-db");
        let database_config =
            DatabaseConfig::new(database_dir.clone(), lock_file, transaction_data_dir)?;

        let database = Database::create(database_config, DbSchema::default())?;

        let entries = database.entries();
        assert!(entries.is_empty());
        let database_metadata = database_dir.symlink_metadata()?;
        assert_eq!(database_metadata.permissions().mode(), DIR_MODE);

        Ok(())
    }

    /// Ensures, that [`Database::create`] succeeds on an existing, empty directory.
    #[test]
    fn database_create_succeeds_with_empty_dir() -> TestResult {
        init_logger();
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        let database_dir = temp_path.join("db/local");
        let lock_file = temp_path.join("db/db.lck");
        let transaction_data_dir = temp_path.join("alpm-db");
        let database_config =
            DatabaseConfig::new(database_dir.clone(), lock_file, transaction_data_dir)?;

        create_dir_all(&database_dir)?;
        set_permissions(&database_dir, PermissionsExt::from_mode(0o777))?;

        let database = Database::create(database_config, DbSchema::default())?;

        let entries = database.entries();
        assert!(entries.is_empty());
        let database_metadata = database_dir.symlink_metadata()?;
        assert_eq!(database_metadata.permissions().mode(), DIR_MODE);

        Ok(())
    }

    /// Ensures, that [`Database::new`] succeeds to open an existing, empty database.
    #[test]
    fn database_new_succeeds_with_empty_database() -> TestResult {
        init_logger();
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        let database_dir = temp_path.join("db/local");
        let lock_file = temp_path.join("db/db.lck");
        let transaction_data_dir = temp_path.join("alpm-db");
        let database_config =
            DatabaseConfig::new(database_dir.clone(), lock_file, transaction_data_dir)?;

        create_dir_all(&database_dir)?;
        let schema = DbSchema::default();
        schema.write_version_file_to_db_root(&database_dir)?;

        let database = Database::new(database_config)?;

        let entries = database.entries();
        assert!(entries.is_empty());
        let database_metadata = database_dir.symlink_metadata()?;
        assert_eq!(database_metadata.permissions().mode(), DIR_MODE);

        Ok(())
    }

    /// Ensures, that [`Database::create`] fails on target dir being a file.
    #[test]
    fn database_create_fails_on_file_as_base_dir() -> TestResult {
        init_logger();
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        let database_dir = temp_path.join("db/local");
        let lock_file = temp_path.join("db/db.lck");
        let transaction_data_dir = temp_path.join("alpm-db");
        let database_config =
            DatabaseConfig::new(database_dir.clone(), lock_file, transaction_data_dir)?;

        create_dir_all(temp_path.join("db"))?;
        File::create(&database_dir)?;

        let Err(error) = Database::create(database_config, DbSchema::default()) else {
            panic!("Expected to fail with Error::DatabaseCreationNotEmpty");
        };

        debug!("{error}");
        assert!(matches!(
            error,
            crate::Error::Database(Error::DatabaseIsNotADirectory { .. })
        ));

        Ok(())
    }

    /// Ensures, that [`Database::create`] fails on an existing, non-empty directory.
    #[test]
    fn database_create_fails_with_non_empty_dir() -> TestResult {
        init_logger();
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        let database_dir = temp_path.join("db/local");
        let lock_file = temp_path.join("db/db.lck");
        let transaction_data_dir = temp_path.join("alpm-db");
        let database_config =
            DatabaseConfig::new(database_dir.clone(), lock_file, transaction_data_dir)?;

        create_dir_all(database_dir.join("example-1.0.0-1"))?;

        let Err(error) = Database::create(database_config, DbSchema::default()) else {
            panic!("Expected to fail with Error::DatabaseCreationNotEmpty");
        };

        debug!("{error}");
        assert!(matches!(
            error,
            crate::Error::Database(Error::DatabaseCreationNotEmpty { .. })
        ));

        Ok(())
    }

    /// Ensures, that [`Database::new`] fails on non-directory path.
    #[test]
    fn database_new_fails_on_not_a_directory() -> TestResult {
        init_logger();
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        let database_dir = temp_path.join("db/local");
        let lock_file = temp_path.join("db/db.lck");
        let transaction_data_dir = temp_path.join("alpm-db");
        let database_config =
            DatabaseConfig::new(database_dir.clone(), lock_file, transaction_data_dir)?;

        create_dir_all(temp_path.join("db"))?;
        File::create_new(&database_dir)?;

        let Err(error) = Database::new(database_config) else {
            panic!("Expected to fail with Error::DatabaseIsNotADirectory");
        };

        debug!("{error}");
        assert!(matches!(
            error,
            crate::Error::Database(Error::DatabaseIsNotADirectory { .. })
        ));

        Ok(())
    }

    /// Ensures, that [`Database::new`] fails, if a write lock exists in a default location.
    #[test]
    fn database_new_fails_on_default_write_lock() -> TestResult {
        init_logger();
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        let database_dir = temp_path.join("db/local");
        let lock_file = temp_path.join("db/db.lck");
        let transaction_data_dir = temp_path.join("alpm-db");
        let database_config = DatabaseConfig::new(
            database_dir.clone(),
            lock_file.clone(),
            transaction_data_dir,
        )?;

        create_dir_all(&database_dir)?;
        let schema = DbSchema::default();
        schema.write_version_file_to_db_root(&database_dir)?;
        File::create_new(&lock_file)?;

        let Err(error) = Database::new(database_config) else {
            panic!("Expected to fail with Error::DatabaseIsLocked");
        };

        debug!("{error}");
        assert!(matches!(
            error,
            crate::Error::Database(Error::DatabaseIsLocked { .. })
        ));

        Ok(())
    }

    /// Tests with transactions.
    mod transactions {

        use std::{io::Write, str::FromStr};

        use alpm_common::MetadataFile;
        use alpm_mtree::mtree::MtreeTransfer;
        use alpm_types::{
            Architecture,
            EntryName,
            ExtraData,
            ExtraDataEntry,
            PackageInstallReason,
            PackageValidation,
            RelativeFilePath,
            SystemArchitecture,
        };
        use flate2::{Compression, GzBuilder};

        use super::*;
        use crate::{
            db::{Action, EntrySource},
            desc::DbDescFileV2,
            files::{BackupEntry, DbFilesV1},
        };

        /// Returns an example [`DbDescFile`].
        fn db_desc_file() -> Result<DbDescFile, crate::Error> {
            Ok(DbDescFile::V2(DbDescFileV2 {
                name: "example".parse()?,
                version: "1.0.0-1".parse()?,
                base: "example".parse()?,
                description: "An example package".into(),
                url: Some("https://example.org".parse()?),
                arch: Architecture::Some(SystemArchitecture::X86_64),
                builddate: 1,
                installdate: 2,
                packager: "Foobar McFooface <foobar@mcfooface.org>".parse()?,
                size: 10,
                groups: Vec::new(),
                reason: PackageInstallReason::Explicit,
                license: vec!["0BSD".parse()?],
                validation: vec![PackageValidation::None],
                replaces: Vec::new(),
                depends: Vec::new(),
                optdepends: Vec::new(),
                conflicts: Vec::new(),
                provides: Vec::new(),
                xdata: ExtraData::try_from(ExtraDataEntry::new(
                    "pkgtype".to_string(),
                    "pkg".to_string(),
                ))?,
            }))
        }

        /// Returns an example [`DbFiles`].
        fn db_files_file() -> Result<DbFiles, crate::Error> {
            Ok(DbFiles::V1(DbFilesV1::try_from((
                vec![
                    PathBuf::from("etc/"),
                    PathBuf::from("etc/example"),
                    PathBuf::from("etc/example/conf"),
                    PathBuf::from("usr"),
                    PathBuf::from("usr/bin"),
                    PathBuf::from("usr/bin/foo"),
                ],
                vec![BackupEntry {
                    path: RelativeFilePath::new(PathBuf::from("etc/example/conf"))?,
                    md5: "d3b07384d113edec49eaa6238ad5ff00".parse()?,
                }],
            ))?))
        }

        /// Returns an example [`MtreeTransfer`].
        fn mtree_transfer() -> Result<MtreeTransfer, crate::Error> {
            let data = {
                let data = r#"#mtree
/set mode=644 uid=0 gid=0 type=file
./etc mode=755 type=dir time=1700000000.0
./etc/example mode=755 type=dir time=1700000000.0
./etc/example/conf time=1700000000.0 size=1337 sha256digest=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
./some_dir type=dir time=1700000000.0
./usr mode=755 type=dir time=1700000000.0
./usr/bin mode=755 type=dir time=1700000000.0
./usr/bin/foo time=1700000000.0 size=1337 sha256digest=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
"#.to_string();

                let mut gzip = GzBuilder::new()
                    .filename("mtree")
                    .write(Vec::new(), Compression::default());
                gzip.write_all(&data.into_bytes())
                    .map_err(|source| crate::Error::Io {
                        context: "writing data to a gzip encoder".to_string(),
                        source,
                    })?;
                gzip.finish().map_err(|source| crate::Error::Io {
                    context: "retrieving gzip compressed data".to_string(),
                    source,
                })?
            };

            Ok(MtreeTransfer::from_reader_with_schema(
                data.as_slice(),
                None,
            )?)
        }

        /// A single example transaction data.
        #[derive(Debug)]
        struct ExampleTransactionData {
            entry_name: EntryName,
        }

        impl EntrySource for ExampleTransactionData {
            fn entry_name(&self) -> EntryName {
                self.entry_name.clone()
            }

            fn alpm_db_desc(&self) -> Result<DbDescFile, crate::Error> {
                db_desc_file()
            }

            fn alpm_db_files(&self) -> Result<DbFiles, crate::Error> {
                db_files_file()
            }

            fn alpm_mtree(&self) -> Result<MtreeTransfer, crate::Error> {
                mtree_transfer()
            }

            fn alpm_install_scriptlet(&self) -> Result<Option<String>, crate::Error> {
                let script = r#"!/usr/bin/env bash

post_install() {
    printf 'YOLO\n'
}
"#;
                Ok(Some(script.to_string()))
            }
        }

        /// Ensures, that [`Database::commit_transaction`] succeeds to write an example entry to a
        /// newly created database.
        #[test]
        fn database_commit_transaction_succeeds() -> TestResult {
            init_logger();
            let temp_dir = TempDir::new()?;
            let temp_path = temp_dir.path();
            let database_dir = temp_path.join("db/local");
            let lock_file = temp_path.join("db/db.lck");
            let transaction_data_dir = temp_path.join("alpm-db");
            let database_config = DatabaseConfig::new(
                database_dir.clone(),
                lock_file.clone(),
                transaction_data_dir.clone(),
            )?;

            // Creates a new, empty database.
            let mut database = Database::create(database_config, DbSchema::default())?;
            let entries = database.entries();
            assert!(entries.is_empty());

            // Executes example transaction to add a single entry.
            database.init_transaction()?;
            assert!(lock_file.exists());
            database.add_action(Action::Add(Box::new(ExampleTransactionData {
                entry_name: "example-1.0.0-1".parse()?,
            })))?;
            database.commit_transaction()?;
            assert!(!lock_file.exists());

            let database_config = DatabaseConfig::new(
                database_dir.clone(),
                lock_file.clone(),
                transaction_data_dir.clone(),
            )?;
            let mut database = Database::new(database_config)?;
            let entries = database.entries();
            assert_eq!(entries.len(), 1);
            let entries = database.db_descs()?;
            assert_eq!(entries.len(), 1);
            let entries = database.db_files()?;
            assert_eq!(entries.len(), 1);
            let entries = database.installed_packages()?;
            assert_eq!(
                entries.first(),
                Some(&InstalledPackage::from_str("example-1.0.0-1-x86_64")?)
            );
            let entries = database.mtrees()?;
            assert_eq!(entries.len(), 1);

            Ok(())
        }
    }
}
