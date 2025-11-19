//! Management of the [alpm-db] directory structure.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::{
    collections::BTreeMap,
    fs::{self, DirEntry, OpenOptions, create_dir_all, read_dir},
    path::{Path, PathBuf},
    sync::Arc,
};

use alpm_types::Name;
use fluent_i18n::t;

use crate::{
    Error,
    db::{DatabaseEntry, DatabaseEntryName, DbSchema},
};

/// The name of the lock file used to prevent concurrent database access.
const DB_LOCK_FILE_NAME: &str = "db.lck";

/// A file-based lock to prevent concurrent access to a database.
#[derive(Debug)]
struct DatabaseLock {
    /// The path to the lock file.
    path: PathBuf,
    /// The underlying file handle.
    _file: std::fs::File,
}

impl DatabaseLock {
    /// Acquires a new database lock for the database at given `base_path`.
    ///
    /// # Errors
    ///
    /// Returns an error if creating the lock file fails.
    fn acquire(base_path: &Path) -> Result<Self, Error> {
        let lock_dir = base_path.parent().unwrap_or(base_path);
        let path = lock_dir.join(DB_LOCK_FILE_NAME);
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|source| Error::IoPathError {
                path: path.clone(),
                context: t!("error-io-path-db-lock-create"),
                source,
            })?;
        Ok(Self { path, _file: file })
    }
}

impl Drop for DatabaseLock {
    /// Releases the database lock by removing the lock file.
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// A representation of an [alpm-db] database on disk.
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
#[derive(Clone, Debug)]
pub struct Database {
    /// The base path of the database.
    pub base_path: PathBuf,
    /// The schema of the database.
    ///
    /// See [`DbSchema`] variants for available schema versions.
    pub schema: DbSchema,

    /// The database lock to prevent concurrent access.
    _lock: Arc<DatabaseLock>,
}

/// A report of issues found while checking a database on disk.
#[derive(Debug, Default)]
pub struct DatabaseCheckReport {
    /// The number of entries that were checked.
    pub entries_checked: usize,
    /// The list of errors that were found.
    pub errors: Vec<Error>,
}

impl Database {
    /// Creates a new [`Database`] in the `base_path` using a `schema`.
    ///
    /// Creates the directory `base_path` if it does not exist yet.
    /// Writes a schema version file using the provided `schema`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the directory cannot be created,
    /// - or the schema version file cannot be written.
    pub fn create(base_path: impl AsRef<Path>, schema: DbSchema) -> Result<Self, Error> {
        let base_path = base_path.as_ref();
        create_dir_all(base_path).map_err(|source| Error::IoPathError {
            path: base_path.to_path_buf(),
            context: t!("error-io-path-db-base-create"),
            source,
        })?;

        let lock = Arc::new(DatabaseLock::acquire(base_path)?);
        schema.write_version_file(base_path)?;
        Ok(Self {
            base_path: base_path.to_path_buf(),
            schema,
            _lock: lock,
        })
    }

    /// Creates a new [`Database`] by opening an existing structure at `base_path`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the directory does not exist,
    /// - the path is not a directory,
    /// - reading the schema version file fails,
    /// - or duplicate entries exist (same package name, different versions).
    pub fn open(base_path: impl AsRef<Path>) -> Result<Self, Error> {
        let base_path = base_path.as_ref();

        let metadata = fs::metadata(base_path).map_err(|source| Error::IoPathError {
            path: base_path.to_path_buf(),
            context: t!("error-io-path-db-base-metadata"),
            source,
        })?;
        if !metadata.is_dir() {
            return Err(alpm_common::Error::NotADirectory {
                path: base_path.to_path_buf(),
            }
            .into());
        }

        let lock = Arc::new(DatabaseLock::acquire(base_path)?);
        let schema = DbSchema::read_from(base_path)?;
        let db = Self {
            base_path: base_path.to_path_buf(),
            schema,
            _lock: lock,
        };
        let report = db.check()?;
        if let Some(error) = report
            .errors
            .into_iter()
            .find(|error| matches!(error, Error::DatabaseEntryDuplicateName(_)))
        {
            return Err(error);
        }
        Ok(db)
    }

    /// Returns the list of all entries in the database.
    ///
    /// # Notes
    ///
    /// - Skips any non-directory entries.
    /// - Sorts the entries by name.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - reading the database directory fails,
    /// - or parsing any of the entries fails.
    pub fn entries(&self) -> Result<Vec<DatabaseEntry>, Error> {
        let mut entries = Vec::new();
        for dir_entry in self.entries_iter()? {
            let dir_entry = dir_entry?;
            let file_type = dir_entry.file_type().map_err(|source| Error::IoPathError {
                path: self.base_path.clone(),
                context: t!("error-io-path-db-entries-iterate"),
                source,
            })?;
            if !file_type.is_dir() || file_type.is_symlink() {
                continue;
            }
            let path = dir_entry.path();
            entries.push(DatabaseEntry::try_from(path)?);
        }
        entries.sort();
        Ok(entries)
    }

    /// Returns an iterator over all directory entries in the database.
    ///
    /// # Notes
    ///
    /// - The iterator yields [`DirEntry`] items without filtering.
    ///
    /// # Errors
    ///
    /// Returns an error if reading the database directory fails.
    pub fn entries_iter(&self) -> Result<impl Iterator<Item = Result<DirEntry, Error>>, Error> {
        let base_path = self.base_path.clone();
        let iter = read_dir(&self.base_path)
            .map_err(|source| Error::IoPathError {
                path: self.base_path.clone(),
                context: t!("error-io-path-db-entries-read"),
                source,
            })?
            .map(move |dir_entry| {
                dir_entry.map_err(|source| Error::IoPathError {
                    path: base_path.clone(),
                    context: t!("error-io-path-db-entries-iterate"),
                    source,
                })
            });
        Ok(iter)
    }

    /// Checks the database for potential issues.
    ///
    /// It currently checks for:
    ///
    /// - invalid entries,
    /// - duplicate entries (same package name, different versions).
    ///
    /// # Errors
    ///
    /// Returns an error if reading the database directory fails.
    pub fn check(&self) -> Result<DatabaseCheckReport, Error> {
        let mut report = DatabaseCheckReport::default();
        let mut entries_by_name: BTreeMap<Name, Vec<DatabaseEntryName>> = BTreeMap::new();

        let dir_entries = read_dir(&self.base_path).map_err(|source| Error::IoPathError {
            path: self.base_path.clone(),
            context: t!("error-io-path-db-entries-read"),
            source,
        })?;

        for dir_entry in dir_entries {
            let dir_entry = match dir_entry {
                Ok(entry) => entry,
                Err(source) => {
                    report.errors.push(Error::IoPathError {
                        path: self.base_path.clone(),
                        context: t!("error-io-path-db-entries-iterate"),
                        source,
                    });
                    continue;
                }
            };
            let file_type = match dir_entry.file_type() {
                Ok(file_type) => file_type,
                Err(source) => {
                    report.errors.push(Error::IoPathError {
                        path: self.base_path.clone(),
                        context: t!("error-io-path-db-entries-iterate"),
                        source,
                    });
                    continue;
                }
            };
            if !file_type.is_dir() || file_type.is_symlink() {
                continue;
            }

            let path = dir_entry.path();
            report.entries_checked += 1;

            let entry_name = match DatabaseEntryName::try_from(path.as_path()) {
                Ok(name) => name,
                Err(error) => {
                    report.errors.push(error);
                    continue;
                }
            };

            entries_by_name
                .entry(entry_name.name.clone())
                .or_default()
                .push(entry_name);

            if let Err(error) = DatabaseEntry::try_from(path.as_path()) {
                report.errors.push(error);
            }
        }

        for (name, mut entries) in entries_by_name {
            if entries.len() > 1 {
                entries.sort();
                report
                    .errors
                    .push(Error::DatabaseEntryDuplicateName(Box::new(
                        crate::error::DatabaseEntryDuplicateName { name, entries },
                    )));
            }
        }

        Ok(report)
    }

    /// Retrieves an entry by its name.
    ///
    /// Returns `Ok(None)` if the entry does not exist.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the entry path exists but is not a directory,
    /// - or parsing the entry fails.
    pub fn entry(&self, name: &DatabaseEntryName) -> Result<Option<DatabaseEntry>, Error> {
        let path = self.base_path.join(name.as_path_buf());
        if !path.exists() {
            return Ok(None);
        }
        if !path.is_dir() {
            return Err(alpm_common::Error::NotADirectory { path }.into());
        }
        Ok(Some(DatabaseEntry::try_from(path)?))
    }

    /// Retrieves an entry by package name.
    ///
    /// Returns [`None`] if no entry is found.
    ///
    /// # Errors
    ///
    /// Returns an error if an entry is found but cannot be parsed.
    pub fn entry_by_name(&self, name: &Name) -> Result<Option<DatabaseEntry>, Error> {
        let mut matches = Vec::new();
        for dir_entry in self.entries_iter()? {
            let dir_entry = dir_entry?;
            let file_type = dir_entry.file_type().map_err(|source| Error::IoPathError {
                path: self.base_path.clone(),
                context: t!("error-io-path-db-entries-iterate"),
                source,
            })?;
            if !file_type.is_dir() || file_type.is_symlink() {
                continue;
            }
            let path = dir_entry.path();
            let entry_name = DatabaseEntryName::try_from(path.as_path())?;
            if entry_name.name.as_ref() != name.as_ref() {
                continue;
            }
            matches.push(DatabaseEntry::try_from(path)?);
        }
        matches.sort();
        Ok(matches.pop())
    }

    /// Creates a new entry on disk.
    ///
    /// Ensures that no other entry of the same name exists.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - another entry of the same name (potentially differing version) exists already,
    /// - writing the entry to disk fails.
    pub fn create_entry(&self, entry: &DatabaseEntry) -> Result<PathBuf, Error> {
        if self.entry(&entry.name)?.is_some() {
            return Err(Error::DatabaseEntryAlreadyExists {
                name: entry.name.clone(),
            });
        }
        entry.write_to_db(&self.base_path)
    }

    /// Deletes all entries for the given package name.
    ///
    /// Removes every directory whose entry name matches `name`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - reading the database directory fails,
    /// - parsing an existing entry name fails,
    /// - or removing an entry directory fails.
    pub fn delete_entry(&self, name: &Name) -> Result<(), Error> {
        for dir_entry in read_dir(&self.base_path).map_err(|source| Error::IoPathError {
            path: self.base_path.clone(),
            context: t!("error-io-path-db-entries-read"),
            source,
        })? {
            let dir_entry = dir_entry.map_err(|source| Error::IoPathError {
                path: self.base_path.clone(),
                context: t!("error-io-path-db-entries-iterate"),
                source,
            })?;
            let path = dir_entry.path();
            if !path.is_dir() {
                continue;
            }

            let entry_name = DatabaseEntryName::try_from(path.as_path())?;
            if entry_name.name.as_ref() != name.as_ref() {
                continue;
            }

            fs::remove_dir_all(&path).map_err(|source| Error::IoPathError {
                path: path.clone(),
                context: t!("error-io-path-db-entry-remove"),
                source,
            })?;
        }

        Ok(())
    }

    /// Updates (or adds) an entry on disk.
    ///
    /// Removes any existing entries with the same package name and then writes the new entry.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - reading the database directory fails,
    /// - removing an existing entry directory fails,
    /// - or writing the new entry fails.
    pub fn update_entry(&self, entry: &DatabaseEntry) -> Result<PathBuf, Error> {
        self.delete_entry(&entry.name.name)?;
        self.create_entry(entry)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alpm_types::FullVersion;
    use testresult::TestResult;

    use super::*;
    use crate::{db::entry::tests::create_sample_entry, desc::DbDescFile};

    #[test]
    fn create_entry() -> TestResult {
        let tmp = tempfile::tempdir()?;
        let db_path = tmp.path().join("db");
        let schema = DbSchema::latest();
        let db = Database::create(&db_path, schema.clone())?;

        let entry = create_sample_entry()?;
        db.create_entry(&entry)?;

        drop(db);
        let reopened = Database::open(&db_path)?;
        assert_eq!(reopened.schema, schema);
        let entries = reopened.entries()?;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, entry.name);
        assert_eq!(entries[0].desc, entry.desc);
        assert_eq!(entries[0].mtree.raw, entry.mtree.raw);

        let from_lookup = reopened.entry(&entry.name)?.expect("entry missing");
        assert_eq!(from_lookup.name, entry.name);
        Ok(())
    }

    #[test]
    fn update_entry() -> TestResult {
        let tmp = tempfile::tempdir()?;
        let db_path = tmp.path().join("db");
        let db = Database::create(&db_path, DbSchema::latest())?;

        let original = create_sample_entry()?;
        db.create_entry(&original)?;

        let mut updated = original.clone();
        updated.name = DatabaseEntryName::from_str("foo-1.0.0-2")?;
        match &mut updated.desc {
            DbDescFile::V1(desc) => desc.version = FullVersion::from_str("1.0.0-2")?,
            DbDescFile::V2(desc) => desc.version = FullVersion::from_str("1.0.0-2")?,
        }

        let new_path = db.update_entry(&updated)?;
        assert_eq!(new_path, db_path.join(updated.name.as_path_buf()));
        assert!(!db_path.join(original.name.as_path_buf()).exists());

        let entries = db.entries()?;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, updated.name);
        assert_eq!(entries[0].desc, updated.desc);
        Ok(())
    }

    #[test]
    fn delete_entry() -> TestResult {
        let tmp = tempfile::tempdir()?;
        let db_path = tmp.path().join("db");
        let db = Database::create(&db_path, DbSchema::latest())?;

        let entry = crate::db::entry::tests::create_sample_entry()?;
        db.create_entry(&entry)?;
        db.delete_entry(&entry.name.name)?;

        assert!(!db_path.join(entry.name.as_path_buf()).exists());
        let entries = db.entries()?;
        assert!(entries.is_empty());
        Ok(())
    }

    #[test]
    fn entry_by_name() -> TestResult {
        let tmp = tempfile::tempdir()?;
        let db_path = tmp.path().join("db");
        let db = Database::create(&db_path, DbSchema::latest())?;

        let first = create_sample_entry()?;
        db.create_entry(&first)?;

        let mut second = first.clone();
        second.name = DatabaseEntryName::from_str("foo-2.0.0-1")?;
        match &mut second.desc {
            DbDescFile::V1(desc) => desc.version = FullVersion::from_str("2.0.0-1")?,
            DbDescFile::V2(desc) => desc.version = FullVersion::from_str("2.0.0-1")?,
        }
        db.create_entry(&second)?;

        // Should retrieve the newest version
        let newest = db.entry_by_name(&first.name.name)?.expect("missing entry");
        assert_eq!(newest.name, second.name);
        Ok(())
    }

    #[test]
    fn open_fails_on_duplicate_entries() -> TestResult {
        let tmp = tempfile::tempdir()?;
        let db_path = tmp.path().join("db");
        let db = Database::create(&db_path, DbSchema::latest())?;

        let first = create_sample_entry()?;
        db.create_entry(&first)?;

        let mut second = first.clone();
        second.name = DatabaseEntryName::from_str("foo-2.0.0-1")?;
        match &mut second.desc {
            DbDescFile::V1(desc) => desc.version = FullVersion::from_str("2.0.0-1")?,
            DbDescFile::V2(desc) => desc.version = FullVersion::from_str("2.0.0-1")?,
        }
        db.create_entry(&second)?;

        drop(db);
        let err = Database::open(&db_path).unwrap_err();
        assert!(matches!(err, Error::DatabaseEntryDuplicateName(_)));
        Ok(())
    }

    #[test]
    fn lock_file() -> TestResult {
        let tmp = tempfile::tempdir()?;
        let db_path = tmp.path().join("db");
        let lock_path = tmp.path().join(DB_LOCK_FILE_NAME);

        let db = Database::create(&db_path, DbSchema::latest())?;
        assert!(lock_path.exists());

        drop(db);
        assert!(!lock_path.exists());
        Ok(())
    }

    #[test]
    fn duplicate_entry_creation_fails() -> TestResult {
        let tmp = tempfile::tempdir()?;
        let db_path = tmp.path().join("db");
        let db = Database::create(&db_path, DbSchema::latest())?;

        let entry = create_sample_entry()?;
        db.create_entry(&entry)?;

        let err = db.create_entry(&entry).unwrap_err();
        assert!(matches!(
            err,
            Error::DatabaseEntryAlreadyExists { name } if name == entry.name
        ));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn entries_skip_symlinked_dirs() -> TestResult {
        let tmp = tempfile::tempdir()?;
        let db_path = tmp.path().join("db");
        let db = Database::create(&db_path, DbSchema::latest())?;

        let entry = create_sample_entry()?;
        db.create_entry(&entry)?;

        let target = db_path.join(entry.name.as_path_buf());
        let link_path = db_path.join("symlinked-entry");
        std::os::unix::fs::symlink(&target, &link_path)?;

        let entries = db.entries()?;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, entry.name);
        Ok(())
    }
}
