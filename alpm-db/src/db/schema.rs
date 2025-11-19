//! Schema handling for [alpm-db] database files.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::{
    fmt::{Display, Formatter},
    fs,
    io::Read,
    path::{MAIN_SEPARATOR, Path, PathBuf},
    str::FromStr,
};

use alpm_common::FileFormatSchema;
use alpm_types::{SchemaVersion, semver_version::Version};

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

    /// Writes the schema version file into the given `root_path`.
    ///
    /// It writes the version followed by a newline.
    ///
    /// # Errors
    ///
    /// Returns an error if the version file cannot be written.
    pub fn write_version_file(&self, root_path: impl AsRef<Path>) -> Result<(), Error> {
        let root_path = root_path.as_ref();
        let root_path = root_path.join(ALPM_DB_VERSION_FILE);
        fs::write(&root_path, format!("{}{MAIN_SEPARATOR}", self)).map_err(|source| {
            Error::IoPathError {
                path: root_path,
                context: "writing ALPM_DB_VERSION file",
                source,
            }
        })
    }

    /// Reads the schema version file from the given `root_path`.
    ///
    /// It trims any trailing main path separators from the file contents before parsing.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or does not describe a supported schema.
    pub fn read_from(root_path: impl AsRef<Path>) -> Result<Self, Error> {
        let root_path = root_path.as_ref();
        let root_path = root_path.join(ALPM_DB_VERSION_FILE);
        let contents = fs::read_to_string(&root_path).map_err(|source| Error::IoPathError {
            path: root_path.clone(),
            context: "reading ALPM_DB_VERSION file",
            source,
        })?;
        Self::derive_from_str(contents.trim_end_matches(MAIN_SEPARATOR))
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
            context: "opening ALPM_DB_VERSION",
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
                context: "reading ALPM_DB_VERSION",
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
