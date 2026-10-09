//! Schema handling for [alpm-db] database files.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::{
    fmt::{Display, Formatter},
    fs::{File, write},
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_common::FileFormatSchema;
use alpm_types::{SchemaVersion, semver_version::Version};
use fluent_i18n::t;

use crate::Error;

/// The current major version supported by this library.
const VERSION_9: u64 = 9;

/// Describes supported schema versions of the ALPM databases.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DbSchema {
    /// Schema version 9 as introduced with pacman 4.2.0 on 2014-12-19.
    V9(SchemaVersion),
}

impl DbSchema {
    /// The name of the schema version file in an [alpm-db] database.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    pub const VERSION_FILE: &str = "ALPM_DB_VERSION";

    /// Writes the schema version file into the given base path.
    ///
    /// It writes the version followed by a newline.
    ///
    /// # Errors
    ///
    /// Returns an error if the version file cannot be written.
    pub fn write_version_file(&self, base_path: impl AsRef<Path>) -> Result<(), Error> {
        let base_path = base_path.as_ref();
        let file = base_path.join(Self::VERSION_FILE);
        write(&file, format!("{}\n", self)).map_err(|source| Error::IoPath {
            path: file,
            context: t!("error-io-path-write-db-version"),
            source,
        })
    }

    /// Reads an [alpm-db] version file located in `base_path`.
    ///
    /// It trims any trailing newlines from the file contents before parsing.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or does not describe a supported schema.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    pub fn read_from_version_file_in_dir(base_path: impl AsRef<Path>) -> Result<Self, Error> {
        let base_path = base_path.as_ref();
        Self::derive_from_file(base_path.join(Self::VERSION_FILE))
    }
}

impl AsRef<SchemaVersion> for DbSchema {
    fn as_ref(&self) -> &SchemaVersion {
        match self {
            Self::V9(v) => v,
        }
    }
}

impl Default for DbSchema {
    /// Returns the latest supported schema version.
    fn default() -> Self {
        Self::V9(SchemaVersion::new(Version::new(VERSION_9, 0, 0)))
    }
}

impl Display for DbSchema {
    /// Formats the schema version as a string.
    ///
    /// Uses only the major version number of the underlying [`SchemaVersion`].
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl FileFormatSchema for DbSchema {
    type Err = Error;

    /// Derives a [`DbSchema`] from a version file.
    ///
    /// Opens the `file` and defers to [`DbSchema::derive_from_reader`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the file cannot be opened for reading,
    /// - or deriving a [`DbSchema`] from its contents fails.
    fn derive_from_file(file: impl AsRef<Path>) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        let file = file.as_ref();
        let mut handle = File::open(file).map_err(|source| Error::IoPath {
            path: PathBuf::from(file),
            context: t!("error-io-path-open-db-version"),
            source,
        })?;
        Self::derive_from_reader(&mut handle)
    }

    /// Derives a [`DbSchema`] from a reader.
    ///
    /// Reads the `reader` to a string and defers to [`DbSchema::derive_from_str`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - reading from `reader` fails,
    /// - or deriving a [`DbSchema`] from its contents fails.
    fn derive_from_reader(mut reader: impl Read) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .map_err(|source| Error::IoRead {
                context: t!("error-io-read-db-version"),
                source,
            })?;
        Self::derive_from_str(buf.trim())
    }

    /// Derives a [`DbSchema`] from a string containing the schema version.
    ///
    /// # Errors
    ///
    /// Returns an error if deriving a [`DbSchema`] from the string fails.
    fn derive_from_str(s: &str) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        Self::from_str(s)
    }
}

impl FromStr for DbSchema {
    type Err = Error;

    /// Parses a [`DbSchema`] from a string containing the schema version.
    ///
    /// # Errors
    ///
    /// Returns an error if deriving a [`DbSchema`] from the string fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() || s != VERSION_9.to_string() {
            return Err(Error::UnsupportedSchemaVersion(s.to_string()));
        }

        Ok(Self::V9(SchemaVersion::new(Version::new(VERSION_9, 0, 0))))
    }
}

impl TryFrom<SchemaVersion> for DbSchema {
    type Error = Error;

    /// Tries to convert a [`SchemaVersion`] into a [`DbSchema`].
    ///
    /// # Errors
    ///
    /// Returns an error if the given schema version is not supported.
    fn try_from(value: SchemaVersion) -> Result<Self, Self::Error> {
        match value.as_ref().major {
            VERSION_9 => Ok(Self::V9(value)),
            _ => Err(Error::UnsupportedSchemaVersion(value.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::write, io::Cursor, str::FromStr};

    use tempfile::TempDir;
    use testresult::TestResult;

    use super::*;

    /// Ensures, that [`DbSchema::from_str`] succeeds on a valid version string.
    #[test]
    fn db_schema_from_str_succeeds() -> TestResult {
        assert_eq!(DbSchema::default(), DbSchema::from_str("9")?);
        Ok(())
    }

    /// Ensures, that [`DbSchema::from_str`] fails on empty string.
    #[test]
    fn db_schema_from_str_fails_on_empty_string() {
        let Err(Error::UnsupportedSchemaVersion(_)) = DbSchema::from_str("") else {
            panic!("Expected to fail but succeeded instead");
        };
    }

    /// Ensures, that [`DbSchema::from_str`] fails on an unsupported version string.
    #[test]
    fn db_schema_from_str_fails_on_unsupported_version() {
        let Err(Error::UnsupportedSchemaVersion(_)) = DbSchema::from_str("8") else {
            panic!("Expected to fail but succeeded instead");
        };
    }

    /// Ensures, that [`DbSchema::derive_from_file`] succeeds on a valid version file.
    #[test]
    fn db_schema_derive_from_file() -> TestResult {
        let test_dir = TempDir::new()?;
        let version_file_path = test_dir.path().join(DbSchema::VERSION_FILE);
        write(&version_file_path, "9\n")?;
        let schema = DbSchema::derive_from_file(&version_file_path)?;
        assert_eq!(DbSchema::default(), schema);
        Ok(())
    }

    /// Ensures, that [`DbSchema::derive_from_reader`] succeeds on a [`Read`] impl with valid data.
    #[test]
    fn db_schema_derive_from_file_reader_succeeds() -> TestResult {
        let schema = DbSchema::derive_from_reader(Cursor::new("9\n"))?;
        assert_eq!(DbSchema::default(), schema);
        Ok(())
    }
}
