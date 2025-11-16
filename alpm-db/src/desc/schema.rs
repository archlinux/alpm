//! Schema definition for the [alpm-db-desc] file format.
//!
//! [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html

use std::{
    fmt::{Display, Formatter},
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_common::FileFormatSchema;
use alpm_types::{SchemaVersion, semver_version::Version};
use fluent_i18n::t;

use crate::Error;

/// An enum describing all valid [alpm-db-desc] schemas.
///
/// Each variant corresponds to a specific revision of the
/// specification.
///
/// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DbDescSchema {
    /// Schema for the [alpm-db-descv1] file format.
    ///
    /// [alpm-db-descv1]: https://alpm.archlinux.page/specifications/alpm-db-descv1.5.html
    V1(SchemaVersion),
    /// Schema for the [alpm-db-descv2] file format.
    ///
    /// [alpm-db-descv2]: https://alpm.archlinux.page/specifications/alpm-db-descv2.5.html
    V2(SchemaVersion),
}

impl FileFormatSchema for DbDescSchema {
    type Err = Error;

    /// Returns a reference to the inner [`SchemaVersion`].
    fn inner(&self) -> &SchemaVersion {
        match self {
            DbDescSchema::V1(v) => v,
            DbDescSchema::V2(v) => v,
        }
    }

    /// Derives a [`DbDescSchema`] from an [alpm-db-desc] file on disk.
    ///
    /// Opens the `file` and defers to [`DbDescSchema::derive_from_reader`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the file cannot be opened for reading,
    /// - or deriving a [`DbDescSchema`] from its contents fails.
    ///
    /// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    fn derive_from_file(file: impl AsRef<Path>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let file = file.as_ref();
        Self::derive_from_reader(File::open(file).map_err(|source| Error::IoPathError {
            path: PathBuf::from(file),
            context: t!("error-io-path-schema-file"),
            source,
        })?)
    }

    /// Derives a [`DbDescSchema`] from [alpm-db-desc] data in a reader.
    ///
    /// Reads the `reader` to a string and defers to [`DbDescSchema::derive_from_str`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - reading from `reader` fails,
    /// - or deriving a [`DbDescSchema`] from its contents fails.
    ///
    /// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    fn derive_from_reader(reader: impl std::io::Read) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut buf = String::new();
        let mut reader = reader;
        reader
            .read_to_string(&mut buf)
            .map_err(|source| Error::IoReadError {
                context: t!("error-io-read-schema-data"),
                source,
            })?;
        Self::derive_from_str(&buf)
    }

    /// Derives a [`DbDescSchema`] from a string slice containing [alpm-db-desc] data.
    ///
    /// The parser uses a simple heuristic:
    ///
    /// - v1 → no `%XDATA%` section present
    /// - v2 → `%XDATA%` section present
    ///
    /// This approach avoids relying on explicit version metadata, as the DB desc
    /// format itself is not self-describing.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_common::FileFormatSchema;
    /// use alpm_db::desc::DbDescSchema;
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> Result<(), alpm_db::Error> {
    /// let v1_data = r#"%NAME%
    /// foo
    ///
    /// %VERSION%
    /// 1.0.0-1
    /// "#;
    ///
    /// assert_eq!(
    ///     DbDescSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))),
    ///     DbDescSchema::derive_from_str(v1_data)?
    /// );
    ///
    /// let v2_data = r#"%NAME%
    /// foo
    ///
    /// %VERSION%
    /// 1.0.0-1
    ///
    /// %XDATA%
    /// pkgtype=pkg
    /// "#;
    ///
    /// assert_eq!(
    ///     DbDescSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))),
    ///     DbDescSchema::derive_from_str(v2_data)?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error only if internal conversion or string handling fails.
    ///
    /// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    fn derive_from_str(s: &str) -> Result<DbDescSchema, Error> {
        // Instead of an explicit "format" key, we use a heuristic:
        // presence of `%XDATA%` implies version 2.
        if s.contains("%XDATA%") {
            Ok(DbDescSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))))
        } else {
            Ok(DbDescSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))))
        }
    }
}

impl Default for DbDescSchema {
    /// Returns the default schema variant ([`DbDescSchema::V2`]).
    fn default() -> Self {
        Self::V2(SchemaVersion::new(Version::new(2, 0, 0)))
    }
}

impl FromStr for DbDescSchema {
    type Err = Error;

    /// Parses a [`DbDescSchema`] from a version string.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the input string is not a valid version,
    /// - or the version does not correspond to a known schema variant.
    fn from_str(s: &str) -> Result<DbDescSchema, Self::Err> {
        match SchemaVersion::from_str(s) {
            Ok(version) => Self::try_from(version),
            Err(_) => Err(Error::UnsupportedSchemaVersion(s.to_string())),
        }
    }
}

impl TryFrom<SchemaVersion> for DbDescSchema {
    type Error = Error;

    /// Converts a [`SchemaVersion`] into a corresponding [`DbDescSchema`].
    ///
    /// # Errors
    ///
    /// Returns an error if the major version of `SchemaVersion` does not
    /// correspond to a known [`DbDescSchema`] variant.
    fn try_from(value: SchemaVersion) -> Result<Self, Self::Error> {
        match value.inner().major {
            1 => Ok(DbDescSchema::V1(value)),
            2 => Ok(DbDescSchema::V2(value)),
            _ => Err(Error::UnsupportedSchemaVersion(value.to_string())),
        }
    }
}

impl Display for DbDescSchema {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                DbDescSchema::V1(version) | DbDescSchema::V2(version) => version.inner().major,
            }
        )
    }
}
