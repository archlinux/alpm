//! Management of the [alpm-db] directory structure.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::{
    fs::{File, OpenOptions, create_dir_all, remove_file, set_permissions, symlink_metadata},
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt, chown},
    path::{Path, PathBuf},
    sync::Arc,
};

use alpm_common::FileFormatSchema;
use alpm_mtree::Mtree;
use alpm_types::InstalledPackage;
use fluent_i18n::t;
use log::{debug, error, trace};
use nix::unistd::{getegid, geteuid};

use crate::{
    db::{DbSchema, Entry, Error, TransactionWithData},
    desc::DbDescFile,
    files::DbFiles,
};

/// A file-based lock to prevent concurrent access to a database.
#[derive(Debug)]
pub struct DatabaseLock {
    /// The path to the lock file.
    path: PathBuf,

    /// The underlying file handle.
    file: File,
}

impl DatabaseLock {
    /// The default file name of a lock file.
    pub const DEFAULT_LOCK_FILE_NAME: &str = "db.lck";

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
}

impl AsRef<Path> for DatabaseLock {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Drop for DatabaseLock {
    /// Releases the database lock before the [`DatabaseLock`] is deconstructed.
    ///
    /// This first releases the file lock on the lock file on disk, then removes the file from disk.
    ///
    /// # Note
    ///
    /// As [`Drop`] must not panic or return an error, any error that may occur during release of
    /// the file lock or removal of the lock file is logged as an error message.
    fn drop(&mut self) {
        if let Err(error) = self.file.unlock() {
            error!(
                "Error when releasing the alpm-db lock file {:?}: {error}",
                self.path
            );
        }

        if let Err(error) = remove_file(&self.path) {
            error!(
                "Error when removing the alpm-db lock file {:?}: {error}",
                self.path
            );
        }
    }
}

/// The representation of an [alpm-db].
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
#[derive(Clone, Debug)]
pub struct Database {
    /// The directory in which the data of the database resides.
    pub path: PathBuf,

    /// The schema of the database.
    ///
    /// See [`DbSchema`] variants for available schema versions.
    pub schema: DbSchema,

    /// The database lock to prevent concurrent access.
    lock: Option<Arc<DatabaseLock>>,
}

impl Database {
    /// The default directory permissions of a database root directory.
    pub(crate) const DEFAULT_DIR_PERMISSIONS: u32 = 0o040755;

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
            self.path
        );

        let mut paths = Vec::new();
        for entry in self
            .path
            .read_dir()
            .map_err(|source| crate::Error::IoPath {
                path: self.path.clone(),
                context: t!("error-io-database-read-dir"),
                source,
            })?
        {
            let dir_entry = entry.map_err(|source| crate::Error::IoPath {
                path: self.path.clone(),
                context: t!("error-io-database-read-dir-entry"),
                source,
            })?;

            // Ignore non-directory paths.
            let metadata = dir_entry
                .metadata()
                .map_err(|source| crate::Error::IoPath {
                    path: self.path.clone(),
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
    pub fn entries(&self) -> Result<Vec<Entry>, crate::Error> {
        let paths = self.paths()?;

        let mut entries = Vec::new();
        for path in paths {
            entries.push(Entry::new(path)?)
        }

        Ok(entries)
    }

    /// Returns the list of all [`DbDescFile`]s found in the database.
    ///
    /// # Note
    ///
    /// This is a convenience method, relying on [`Self::entries`].
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
    pub fn db_descs(&self) -> Result<Vec<DbDescFile>, crate::Error> {
        let entries = self.entries()?;
        let mut output = Vec::new();
        for mut entry in entries {
            let data = entry.db_desc()?;
            output.push(data);
        }

        Ok(output)
    }

    /// Returns the list of all [`DbFiles`] found in the database.
    ///
    /// # Note
    ///
    /// This is a convenience method, relying on [`Self::entries`].
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
    pub fn db_files(&self) -> Result<Vec<DbFiles>, crate::Error> {
        let entries = self.entries()?;
        let mut output = Vec::new();
        for mut entry in entries {
            let data = entry.db_files()?;
            output.push(data);
        }

        Ok(output)
    }

    /// Returns the list of all [`Mtree`] found in the database.
    ///
    /// # Note
    ///
    /// This is a convenience method, relying on [`Self::entries`].
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
    pub fn mtrees(&self) -> Result<Vec<Mtree>, crate::Error> {
        let entries = self.entries()?;
        let mut output = Vec::new();
        for mut entry in entries {
            let data = entry.mtree()?;
            output.push(data);
        }

        Ok(output)
    }

    /// Returns the list of all installed packages.
    ///
    /// # Note
    ///
    /// This is a convenience method, relying on [`Self::db_descs`].
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
    pub fn installed_packages(&self) -> Result<Vec<InstalledPackage>, crate::Error> {
        let entries = self.entries()?;
        let mut output = Vec::new();
        for mut entry in entries {
            let data = entry.db_desc()?;
            output.push(data.installed_package());
        }

        Ok(output)
    }

    /// Runs a transaction against this database.
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - the database is not locked
    /// - an action of the transaction fails
    pub fn execute_transaction(
        &self,
        transaction: &TransactionWithData,
    ) -> Result<(), crate::Error> {
        debug!("Executing transaction {}", transaction.uuid());
        if self.lock.is_none() {
            return Err(Error::DatabaseTransactionMissesLock {
                path: self.path.clone(),
                uuid: *transaction.uuid(),
            }
            .into());
        }

        transaction.execute(&self.path)?;

        Ok(())
    }
}

/// A builder for opening a [`Database`].
#[derive(Debug)]
pub struct DatabaseBuilder {
    path: PathBuf,
    schema: Option<DbSchema>,
    lock: Option<Arc<DatabaseLock>>,
}

impl DatabaseBuilder {
    /// Creates a new [`DatabaseBuilder`] for a `path`.
    ///
    /// # Note
    ///
    /// The `path` denotes the [`Database`] root directory.
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        debug!("Prepare to access alpm-db: {:?}", path);
        Self {
            path: path.to_path_buf(),
            schema: None,
            lock: None,
        }
    }

    /// Adds a database schema for creating a new [`Database`].
    ///
    /// # Note
    ///
    /// When using this method, also a [`DatabaseLock`] has to be added using [`Self::write_lock`].
    /// If the targeted [`Database`] root directory (see [`Self::new`]) exists already, it must be
    /// empty!
    pub fn create(self, schema: DbSchema) -> Self {
        debug!("Create new alpm-db with schema {}: {:?}", schema, self.path);

        Self {
            path: self.path,
            schema: Some(schema),
            lock: self.lock,
        }
    }

    /// Adds a write lock for a [`DatabaseBuilder`].
    ///
    /// The `lock_file` targets the [`Database`] which is about to be created or opened for writing.
    ///
    /// # Note
    ///
    /// A lock file is required, when creating a new [alpm-db] and when writing to it.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    pub fn write_lock(self, lock_file: DatabaseLock) -> Self {
        let lock = Arc::new(lock_file);
        debug!(
            "Add write lock for alpm-db {:?}: {:?}",
            self.path,
            lock.as_ref()
        );

        Self {
            path: self.path,
            schema: self.schema,
            lock: Some(lock),
        }
    }

    /// Creates a [`Database`].
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - canonicalizing the path of the targeted database root directory fails
    /// - a database is supposed to be created, but no write lock is provided
    /// - a database is supposed to be created, but the targeted path exists and is not a directory
    /// - a database is supposed to be created, but the targeted directory exists and is not empty
    /// - a database is supposed to be created, but the targeted directory exists and permissions
    ///   for it cannot be adjusted
    /// - a database is supposed to be created, but the targeted directory or any of its parents
    ///   cannot be created
    /// - a database is supposed to be opened, but the targeted path is not a directory
    /// - a database is supposed to be opened read-only (i.e. no lock file is provided using
    ///   [`Self::write_lock`]), but a lock file exists in a default location
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    pub fn finish(self) -> Result<Database, crate::Error> {
        debug!(
            "Open the alpm-db at {:?}{}",
            self.path,
            if let Some(lock) = self.lock.as_ref() {
                format!(", guarded by lock file {:?}", lock.as_ref())
            } else {
                "".to_string()
            }
        );

        // Return the canonicalized path and schema of the created or opened database.
        let (path, schema) = if let Some(schema) = self.schema {
            // Fail if we want to create a database, but didn't provide a write lock for it.
            if self.lock.is_none() {
                return Err(Error::DatabaseCreationMissesLock {
                    path: self.path.clone(),
                }
                .into());
            }

            let path =
                if self.path.as_path().exists() {
                    // If the path exists already, canonicalize it and ensure, that it is valid.
                    // The path must be an empty directory with default permissions, owned by the
                    // UID/GID of the calling process.

                    let path = self.path.as_path().canonicalize().map_err(|source| {
                        crate::Error::IoPath {
                            path: self.path.clone(),
                            context: t!("error-io-path-db-base-canonicalize"),
                            source,
                        }
                    })?;

                    // Fail, if the targeted path is not a directory.
                    if !&path.is_dir() {
                        return Err(Error::DatabaseIsNotADirectory {
                            path: self.path.clone(),
                        }
                        .into());
                    }

                    // Fail, if the targeted directory exists and is not empty.
                    if self
                        .path
                        .as_path()
                        .read_dir()
                        .map_err(|source| crate::Error::IoPath {
                            path: path.clone(),
                            context: t!("error-io-database-read-dir"),
                            source,
                        })?
                        .count()
                        != 0
                    {
                        return Err(Error::DatabaseCreationNotEmpty { path: path.clone() }.into());
                    }

                    let euid = geteuid().as_raw();
                    let egid = getegid().as_raw();
                    let metadata =
                        path.symlink_metadata()
                            .map_err(|source| crate::Error::IoPath {
                                path: self.path.clone(),
                                context: t!("error-io-path-metadata"),
                                source,
                            })?;

                    // Change ownership if it doesn't match the calling UID/GID.
                    if !(metadata.uid() == euid && metadata.gid() == egid) {
                        chown(&path, Some(euid), Some(egid)).map_err(|source| {
                            crate::Error::IoPath {
                                path: path.clone(),
                                context: t!("error-io-path-chown"),
                                source,
                            }
                        })?;
                    }

                    // Change permissions if they don't match the defaults.
                    if metadata.permissions().mode() != Database::DEFAULT_DIR_PERMISSIONS {
                        set_permissions(
                            &path,
                            PermissionsExt::from_mode(Database::DEFAULT_DIR_PERMISSIONS),
                        )
                        .map_err(|source| crate::Error::IoPath {
                            path: path.clone(),
                            context: t!("error-io-path-db-base-create"),
                            source,
                        })?;
                    }

                    path
                } else {
                    // If the path does not yet exist, create it, canonicalize it and set default
                    // permissions.

                    create_dir_all(self.path.as_path()).map_err(|source| crate::Error::IoPath {
                        path: self.path.clone(),
                        context: t!("error-io-path-db-base-create"),
                        source,
                    })?;
                    let path = self.path.as_path().canonicalize().map_err(|source| {
                        crate::Error::IoPath {
                            path: self.path.clone(),
                            context: t!("error-io-path-db-base-canonicalize"),
                            source,
                        }
                    })?;

                    set_permissions(
                        &path,
                        PermissionsExt::from_mode(Database::DEFAULT_DIR_PERMISSIONS),
                    )
                    .map_err(|source| crate::Error::IoPath {
                        path: path.clone(),
                        context: t!("error-io-path-db-base-create"),
                        source,
                    })?;

                    path
                };

            schema.write_version_file(&path)?;

            (path, schema)
        } else {
            let path =
                self.path
                    .as_path()
                    .canonicalize()
                    .map_err(|source| crate::Error::IoPath {
                        path: self.path.clone(),
                        context: t!("error-io-path-db-base-canonicalize"),
                        source,
                    })?;

            // Fail, if the targeted path is not a directory.
            let metadata = symlink_metadata(&path).map_err(|source| crate::Error::IoPath {
                path: path.clone(),
                context: t!("error-io-path-metadata"),
                source,
            })?;
            if !metadata.is_dir() {
                return Err(Error::DatabaseIsNotADirectory { path: path.clone() }.into());
            }
            if metadata.permissions().mode() != Database::DEFAULT_DIR_PERMISSIONS {
                return Err(Error::DatabaseWrongPermissions {
                    path: path.clone(),
                    permissions: metadata.permissions().mode(),
                }
                .into());
            }

            // Fail, if no lock file is specified, but a lock file with the default
            // name exists right next to the database root directory.
            if let Some(parent) = &path.parent()
                && self.lock.is_none()
                && parent.join(DatabaseLock::DEFAULT_LOCK_FILE_NAME).is_file()
            {
                return Err(Error::DatabaseIsLocked {
                    path: path.clone(),
                    lock_file: parent.join(DatabaseLock::DEFAULT_LOCK_FILE_NAME),
                }
                .into());
            }

            let schema = DbSchema::derive_from_file(path.join(DbSchema::VERSION_FILE_NAME))?;

            (path, schema)
        };

        Ok(Database {
            path,
            schema,
            lock: self.lock,
        })
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
        let lock_file_path = temp_dir.path().join(DatabaseLock::DEFAULT_LOCK_FILE_NAME);

        let lock = DatabaseLock::new(&lock_file_path)?;
        let metadata = lock_file_path.metadata()?;
        assert_eq!(
            metadata.permissions().mode(),
            DatabaseLock::DEFAULT_FILE_MODE
        );

        drop(lock);

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

    /// Ensures, that [`DatabaseBuilder`] succeeds to create a new database in a new directory.
    #[test]
    fn database_open_builder_create_succeeds_with_new_dir() -> TestResult {
        init_logger();
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        let database_dir = temp_path.join("local");

        let lock_file = DatabaseLock::new(temp_path.join(DatabaseLock::DEFAULT_LOCK_FILE_NAME))?;
        let database = DatabaseBuilder::new(&database_dir)
            .create(DbSchema::default())
            .write_lock(lock_file)
            .finish()?;

        let entries = database.entries()?;
        assert!(entries.is_empty());
        let database_metadata = database_dir.symlink_metadata()?;
        assert_eq!(
            database_metadata.permissions().mode(),
            Database::DEFAULT_DIR_PERMISSIONS
        );

        Ok(())
    }

    /// Ensures, that [`DatabaseBuilder`] succeeds to create a new database in an existing,
    /// empty directory.
    #[test]
    fn database_open_builder_create_succeeds_with_empty_dir() -> TestResult {
        init_logger();
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        let lock_file = DatabaseLock::new(temp_path.join(DatabaseLock::DEFAULT_LOCK_FILE_NAME))?;
        let database_dir = temp_path.join("local");
        create_dir_all(&database_dir)?;
        set_permissions(&database_dir, PermissionsExt::from_mode(0o777))?;

        let database = DatabaseBuilder::new(&database_dir)
            .create(DbSchema::default())
            .write_lock(lock_file)
            .finish()?;

        let entries = database.entries()?;
        assert!(entries.is_empty());
        let database_metadata = database_dir.symlink_metadata()?;
        assert_eq!(
            database_metadata.permissions().mode(),
            Database::DEFAULT_DIR_PERMISSIONS
        );

        Ok(())
    }

    /// Ensures, that [`DatabaseBuilder`] succeeds to open an existing, empty database.
    #[test]
    fn database_open_builder_open_succeeds_with_empty_database() -> TestResult {
        init_logger();
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        let database_dir = temp_path.join("local");
        create_dir_all(&database_dir)?;
        let schema = DbSchema::default();
        schema.write_version_file(&database_dir)?;

        let database = DatabaseBuilder::new(&database_dir).finish()?;

        let entries = database.entries()?;
        assert!(entries.is_empty());
        let database_metadata = database_dir.symlink_metadata()?;
        assert_eq!(
            database_metadata.permissions().mode(),
            Database::DEFAULT_DIR_PERMISSIONS
        );

        Ok(())
    }

    /// Ensures, that [`DatabaseBuilder`] fails to create a new database in an existing,
    /// non-empty directory.
    #[test]
    fn database_open_builder_create_fails_on_file_as_base_dir() -> TestResult {
        init_logger();
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        let lock_file = DatabaseLock::new(temp_path.join(DatabaseLock::DEFAULT_LOCK_FILE_NAME))?;
        let database_dir = temp_path.join("local");
        File::create(&database_dir)?;

        let Err(error) = DatabaseBuilder::new(&database_dir)
            .create(DbSchema::default())
            .write_lock(lock_file)
            .finish()
        else {
            panic!("Expected to fail with Error::DatabaseCreationNotEmpty");
        };

        debug!("{error}");
        assert!(matches!(
            error,
            crate::Error::Database(Error::DatabaseIsNotADirectory { .. })
        ));

        Ok(())
    }

    /// Ensures, that [`DatabaseBuilder`] fails to create a new database in an existing,
    /// non-empty directory.
    #[test]
    fn database_open_builder_create_fails_with_non_empty_dir() -> TestResult {
        init_logger();
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        let lock_file = DatabaseLock::new(temp_path.join(DatabaseLock::DEFAULT_LOCK_FILE_NAME))?;
        let database_dir = temp_path.join("local");
        create_dir_all(database_dir.join("example-1.0.0-1"))?;

        let Err(error) = DatabaseBuilder::new(&database_dir)
            .create(DbSchema::default())
            .write_lock(lock_file)
            .finish()
        else {
            panic!("Expected to fail with Error::DatabaseCreationNotEmpty");
        };

        debug!("{error}");
        assert!(matches!(
            error,
            crate::Error::Database(Error::DatabaseCreationNotEmpty { .. })
        ));

        Ok(())
    }

    /// Ensures, that [`DatabaseBuilder`] fails to create a new database in an existing,
    /// non-empty directory.
    #[test]
    fn database_open_builder_create_fails_without_write_lock() -> TestResult {
        init_logger();
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        let database_dir = temp_path.join("local");

        let Err(error) = DatabaseBuilder::new(&database_dir)
            .create(DbSchema::default())
            .finish()
        else {
            panic!("Expected to fail with Error::DatabaseCreationMissesLock");
        };

        debug!("{error}");
        assert!(matches!(
            error,
            crate::Error::Database(Error::DatabaseCreationMissesLock { .. })
        ));

        Ok(())
    }

    /// Ensures, that [`DatabaseBuilder`] fails to open an existing database, if the targeted
    /// database root is not a directory.
    #[test]
    fn database_open_builder_open_fails_on_not_a_directory() -> TestResult {
        init_logger();
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        let database_dir = temp_path.join("local");
        File::create_new(&database_dir)?;

        let Err(error) = DatabaseBuilder::new(&database_dir).finish() else {
            panic!("Expected to fail with Error::DatabaseIsNotADirectory");
        };

        debug!("{error}");
        assert!(matches!(
            error,
            crate::Error::Database(Error::DatabaseIsNotADirectory { .. })
        ));

        Ok(())
    }

    /// Ensures, that [`DatabaseBuilder`] fails to open an existing database, if a write lock exists
    /// in a default location.
    #[test]
    fn database_open_builder_open_fails_on_default_write_lock() -> TestResult {
        init_logger();
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        let database_dir = temp_path.join("local");
        create_dir_all(&database_dir)?;
        File::create_new(temp_path.join(DatabaseLock::DEFAULT_LOCK_FILE_NAME))?;

        let Err(error) = DatabaseBuilder::new(&database_dir).finish() else {
            panic!("Expected to fail with Error::DatabaseIsLocked");
        };

        debug!("{error}");
        assert!(matches!(
            error,
            crate::Error::Database(Error::DatabaseIsLocked { .. })
        ));

        Ok(())
    }

    /// Ensures, that [`DatabaseBuilder`] fails to open an existing (empty) database, if the root
    /// directory of the alpm-db has wrong permissions.
    #[test]
    fn database_open_builder_open_fails_on_wrong_permissions() -> TestResult {
        init_logger();
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        let database_dir = temp_path.join("local");
        create_dir_all(&database_dir)?;
        set_permissions(&database_dir, PermissionsExt::from_mode(0o777))?;

        let Err(error) = DatabaseBuilder::new(&database_dir).finish() else {
            panic!("Expected to fail with Error::DatabaseWrongPermissions");
        };

        debug!("{error}");
        assert!(matches!(
            error,
            crate::Error::Database(Error::DatabaseWrongPermissions { .. })
        ));

        Ok(())
    }

    /// Tests with transactions.
    mod transactions {
        use std::str::FromStr;

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
        use rstest::{fixture, rstest};

        use super::*;
        use crate::{
            db::{ActionWithData, WritableEntry},
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

            Ok(MtreeTransfer::from_str_with_schema(&data, None)?)
        }

        /// A single example transaction data.
        #[derive(Debug)]
        struct ExampleTransactionData {
            entry_name: EntryName,
        }

        impl WritableEntry for ExampleTransactionData {
            fn entry_name(&self) -> &alpm_types::EntryName {
                &self.entry_name
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

        /// An example [`TransactionWithData`], adding a single entry.
        #[fixture]
        fn example_transaction() -> Result<TransactionWithData, crate::Error> {
            let action = ActionWithData::Add {
                entry: "example-1.0.0-1".parse()?,
                data: Box::new(ExampleTransactionData {
                    entry_name: "example-1.0.0-1".parse()?,
                }),
            };
            TransactionWithData::new(vec![action])
        }

        /// Ensures, that an example entry can be written to a newly created database.
        #[rstest]
        fn database_execute_transaction_succeeds_to_add_entry(
            example_transaction: Result<TransactionWithData, crate::Error>,
        ) -> TestResult {
            init_logger();
            let temp_dir = TempDir::new()?;
            let temp_path = temp_dir.path();
            let database_dir = temp_path.join("local");

            // Creates a new, empty database.
            let lock_file =
                DatabaseLock::new(temp_path.join(DatabaseLock::DEFAULT_LOCK_FILE_NAME))?;
            let database = DatabaseBuilder::new(&database_dir)
                .create(DbSchema::default())
                .write_lock(lock_file)
                .finish()?;

            let database_metadata = database_dir.symlink_metadata()?;
            assert_eq!(
                database_metadata.permissions().mode(),
                Database::DEFAULT_DIR_PERMISSIONS
            );
            let entries = database.entries()?;
            assert!(entries.is_empty());

            // Executes example transaction to add a single entry.
            let transaction = example_transaction?;
            database.execute_transaction(&transaction)?;

            let entries = database.entries()?;
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

        /// Ensures, that an example entry can be written to a newly created database.
        #[rstest]
        fn database_execute_transaction_fails_on_missing_lock_file(
            example_transaction: Result<TransactionWithData, crate::Error>,
        ) -> TestResult {
            init_logger();
            let temp_dir = TempDir::new()?;
            let temp_path = temp_dir.path();
            let database_dir = temp_path.join("local");

            // Create and then open a database read-only.
            {
                let lock_file =
                    DatabaseLock::new(temp_path.join(DatabaseLock::DEFAULT_LOCK_FILE_NAME))?;
                DatabaseBuilder::new(&database_dir)
                    .create(DbSchema::default())
                    .write_lock(lock_file)
                    .finish()?;
            }
            let database = DatabaseBuilder::new(&database_dir).finish()?;

            let transaction = example_transaction?;

            let Err(error) = database.execute_transaction(&transaction) else {
                panic!("Expected to fail with Error::DatabaseTransactionMissesLock");
            };
            debug!("{error}");
            assert!(matches!(
                error,
                crate::Error::Database(Error::DatabaseTransactionMissesLock { .. })
            ));

            Ok(())
        }
    }
}
