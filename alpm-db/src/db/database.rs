//! Management of the [alpm-db] directory structure.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::{
    fs::{self, create_dir_all, read_dir},
    path::{Path, PathBuf},
};

use crate::{
    Error,
    db::{DatabaseEntry, DatabaseEntryName, DbSchema},
};

/// A representation of an [alpm-db] database on disk.
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
#[derive(Clone, Debug)]
pub struct Database {
    /// The root path of the database.
    pub root_path: PathBuf,
    /// The schema of the database.
    ///
    /// See [`DbSchema`] variants for available schema versions.
    pub schema: DbSchema,
}

impl Database {
    /// Creates a new database in the given `root_path`
    ///
    /// It will also write the schema version file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the directory cannot be prepared,
    /// - or the schema version file cannot be written.
    pub fn create(root_path: impl AsRef<Path>, schema: DbSchema) -> Result<Self, Error> {
        let root_path = root_path.as_ref();
        create_dir_all(root_path).map_err(|source| Error::IoPathError {
            path: root_path.to_path_buf(),
            context: "creating database root",
            source,
        })?;

        schema.write_version_file(root_path)?;
        Ok(Self {
            root_path: root_path.to_path_buf(),
            schema,
        })
    }

    /// Opens an existing database from `root_path`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the directory does not exist,
    /// - the path is not a directory,
    /// - or reading the schema version file fails.
    pub fn open(root_path: impl AsRef<Path>) -> Result<Self, Error> {
        let root_path = root_path.as_ref();

        let metadata = fs::metadata(root_path).map_err(|source| Error::IoPathError {
            path: root_path.to_path_buf(),
            context: "reading metadata for database root",
            source,
        })?;
        if !metadata.is_dir() {
            return Err(alpm_common::Error::NotADirectory {
                path: root_path.to_path_buf(),
            }
            .into());
        }

        let schema = DbSchema::read_from(root_path)?;
        Ok(Self {
            root_path: root_path.to_path_buf(),
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
        for dir_entry in read_dir(&self.root_path).map_err(|source| Error::IoPathError {
            path: self.root_path.clone(),
            context: "reading database entries",
            source,
        })? {
            let dir_entry = dir_entry.map_err(|source| Error::IoPathError {
                path: self.root_path.clone(),
                context: "iterating database entries",
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
        let path = self.root_path.join(name.as_dir_name());
        if !path.exists() {
            return Ok(None);
        }
        if !path.is_dir() {
            return Err(alpm_common::Error::NotADirectory { path }.into());
        }
        Ok(Some(DatabaseEntry::from_dir(path)?))
    }

    /// Adds (or replaces) an entry on disk.
    ///
    /// Delegates to [`DatabaseEntry::write_to`].
    ///
    /// # Errors
    ///
    /// Returns an error if writing the entry to disk fails.
    pub fn add_entry(&self, entry: &DatabaseEntry) -> Result<PathBuf, Error> {
        entry.write_to(&self.root_path)
    }
}

#[cfg(test)]
mod tests {
    use testresult::TestResult;

    use super::*;

    #[test]
    fn create_and_open_database() -> TestResult {
        let tmp = tempfile::tempdir()?;
        let db_path = tmp.path().join("db");
        let schema = DbSchema::latest();
        let db = Database::create(&db_path, schema.clone())?;

        let entry = crate::db::entry::tests::create_sample_entry()?;
        db.add_entry(&entry)?;

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
}
