//! Schema handling for [alpm-db] database files.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::{
    fmt::{Display, Formatter},
    fs,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_common::FileFormatSchema;
use alpm_types::{SchemaVersion, semver_version::Version};
use fluent_i18n::t;

use crate::Error;

/// The name of the schema version file in an [alpm-db] database.
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
pub const ALPM_DB_VERSION_FILE: &str = "ALPM_DB_VERSION";

/// The current major version supported by this library.
const CURRENT_MAJOR_VERSION: u64 = 9;

/// Describes supported schema versions of the ALPM databases.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DbSchema {
    /// Schema version 9 as introduced with pacman 4.2.0 on 2014-12-19.
    V9(SchemaVersion),
}

impl Default for DbSchema {
    /// Returns the latest supported schema version.
    fn default() -> Self {
        Self::latest()
    }
}

impl Display for DbSchema {
    /// Formats the schema version as a string.
    ///
    /// Uses only the major version number of the underlying [`SchemaVersion`].
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner().inner().major)
    }
}

impl DbSchema {
    /// Returns the latest supported schema version.
    pub fn latest() -> Self {
        Self::V9(SchemaVersion::new(Version::new(
            CURRENT_MAJOR_VERSION,
            0,
            0,
        )))
    }

    /// Writes the schema version file into the given base path.
    ///
    /// It writes the version followed by a newline.
    ///
    /// # Errors
    ///
    /// Returns an error if the version file cannot be written.
    pub fn write_version_file(&self, base_path: impl AsRef<Path>) -> Result<(), Error> {
        let base_path = base_path.as_ref();
        let base_path = base_path.join(ALPM_DB_VERSION_FILE);
        fs::write(&base_path, format!("{}\n", self)).map_err(|source| Error::IoPathError {
            path: base_path,
            context: t!("error-io-path-write-db-version"),
            source,
        })
    }

    /// Reads the schema version file from the given base path.
    ///
    /// It trims any trailing newlines from the file contents before parsing.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or does not describe a supported schema.
    pub fn read_from(base_path: impl AsRef<Path>) -> Result<Self, Error> {
        let base_path = base_path.as_ref();
        let base_path = base_path.join(ALPM_DB_VERSION_FILE);
        let contents = fs::read_to_string(&base_path).map_err(|source| Error::IoPathError {
            path: base_path.clone(),
            context: t!("error-io-path-read-db-version"),
            source,
        })?;
        Self::derive_from_str(contents.trim_end_matches('\n'))
    }
}

impl FileFormatSchema for DbSchema {
    type Err = Error;

    /// Returns a reference to the inner [`SchemaVersion`].
    fn inner(&self) -> &SchemaVersion {
        match self {
            DbSchema::V9(version) => version,
        }
    }

    /// Derives a [`DbSchema`] from an [`ALPM_DB_VERSION_FILE`]  file on disk.
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
        let mut handle = fs::File::open(file).map_err(|source| Error::IoPathError {
            path: PathBuf::from(file),
            context: t!("error-io-path-open-db-version"),
            source,
        })?;
        Self::derive_from_reader(&mut handle)
    }

    /// Derives a [`DbSchema`] from a reader containing an [`ALPM_DB_VERSION_FILE`]  file.
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
            .map_err(|source| Error::IoReadError {
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
        if s.is_empty() {
            return Err(Error::UnsupportedSchemaVersion(String::new()));
        }
        let version = SchemaVersion::from_str(s)?;
        Self::try_from(version)
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
        match value.inner().major {
            CURRENT_MAJOR_VERSION => Ok(Self::V9(value)),
            _ => Err(Error::UnsupportedSchemaVersion(value.to_string())),
        }
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
        let version = SchemaVersion::from_str(s)?;
        Self::try_from(version)
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Cursor, str::FromStr};

    use rstest::rstest;
    use tempfile::TempDir;
    use testresult::TestResult;

    use super::*;

    #[rstest]
    #[case("9")]
    #[case("9.0.0")]
    fn derive_from_valid_str(#[case] input: &str) -> TestResult {
        assert_eq!(DbSchema::latest(), DbSchema::derive_from_str(input)?);
        assert_eq!(DbSchema::latest(), DbSchema::from_str(input)?);
        Ok(())
    }

    #[test]
    fn derive_from_file() -> TestResult {
        let test_dir = TempDir::new()?;
        let version_file_path = test_dir.path().join(ALPM_DB_VERSION_FILE);
        fs::write(&version_file_path, "9\n").unwrap();
        let schema = DbSchema::derive_from_file(&version_file_path).unwrap();
        assert_eq!(DbSchema::latest(), schema);
        Ok(())
    }

    #[test]
    fn derive_from_reader_trims_newlines() -> TestResult {
        let schema = DbSchema::derive_from_reader(Cursor::new("9\n"))?;
        assert_eq!(DbSchema::latest(), schema);
        Ok(())
    }

    #[test]
    fn empty_string() {
        let err = DbSchema::derive_from_str("").unwrap_err();
        assert!(matches!(
            err,
            Error::UnsupportedSchemaVersion(version) if version.is_empty()
        ));
    }

    #[test]
    fn unsupported_major_version() {
        let err = DbSchema::derive_from_str("8").unwrap_err();
        assert!(matches!(
            err,
            Error::UnsupportedSchemaVersion(version) if version == "8.0.0"
        ));
    }
}
