//! Management of the [alpm-db] directory structure.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::{
    fs::{self, create_dir_all, read_dir},
    path::{Path, PathBuf},
};

use alpm_types::Name;
use fluent_i18n::t;

use crate::{
    Error,
    db::{DatabaseEntry, DatabaseEntryName, DbSchema},
};

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
}

impl Database {
    /// Creates a new database in the given `base_path`.
    ///
    /// It will also write the schema version file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the directory cannot be prepared,
    /// - or the schema version file cannot be written.
    pub fn create(base_path: impl AsRef<Path>, schema: DbSchema) -> Result<Self, Error> {
        let base_path = base_path.as_ref();
        create_dir_all(base_path).map_err(|source| Error::IoPathError {
            path: base_path.to_path_buf(),
            context: t!("error-io-path-db-base-create"),
            source,
        })?;

        schema.write_version_file(base_path)?;
        Ok(Self {
            base_path: base_path.to_path_buf(),
            schema,
        })
    }

    /// Opens an existing database from `base_path`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the directory does not exist,
    /// - the path is not a directory,
    /// - or reading the schema version file fails.
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

        let schema = DbSchema::read_from(base_path)?;
        Ok(Self {
            base_path: base_path.to_path_buf(),
            schema,
        })
    }

    /// Lists all entries contained in the database.
    ///
    /// # Notes
    ///
    /// - It skips any non-directory entries.
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
            entries.push(DatabaseEntry::from_dir(path)?);
        }
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(entries)
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
        Ok(Some(DatabaseEntry::from_dir(path)?))
    }

    /// Retrieves an entry by package name (any version).
    ///
    /// Returns the newest entry (by [`DatabaseEntryName`] ordering) if multiple versions exist.
    ///
    /// # Errors
    ///
    /// Returns an error if reading or parsing entries fails.
    pub fn entry_by_name(&self, name: &Name) -> Result<Option<DatabaseEntry>, Error> {
        let mut matches: Vec<_> = self
            .entries()?
            .into_iter()
            .filter(|entry| entry.name.name.as_ref() == name.as_ref())
            .collect();
        matches.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(matches.pop())
    }

    /// Adds (or replaces) an entry on disk.
    ///
    /// Delegates to [`DatabaseEntry::write_to`].
    ///
    /// # Errors
    ///
    /// Returns an error if writing the entry to disk fails.
    pub fn create_entry(&self, entry: &DatabaseEntry) -> Result<PathBuf, Error> {
        entry.write_to(&self.base_path)
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

    use alpm_types::Version;
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
            DbDescFile::V1(desc) => desc.version = Version::from_str("1.0.0-2")?,
            DbDescFile::V2(desc) => desc.version = Version::from_str("1.0.0-2")?,
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
            DbDescFile::V1(desc) => desc.version = Version::from_str("2.0.0-1")?,
            DbDescFile::V2(desc) => desc.version = Version::from_str("2.0.0-1")?,
        }
        db.create_entry(&second)?;

        // Should retrieve the newest version
        let newest = db.entry_by_name(&first.name.name)?.expect("missing entry");
        assert_eq!(newest.name, second.name);
        Ok(())
    }
}
