//! Schemas for ALPM-MTREE data.

use std::{
    fmt::{Display, Formatter},
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_common::FileFormatSchema;
use alpm_types::{SchemaVersion, semver_version::Version};
use fluent_i18n::t;

use crate::{Error, mtree_buffer_to_string};

/// An enum tracking all available [ALPM-MTREE] schemas.
///
/// The schema of a ALPM-MTREE refers to its available fields in a specific version.
///
/// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MtreeSchema {
    /// The [ALPM-MTREEv1] file format.
    ///
    /// [ALPM-MTREEv1]: https://alpm.archlinux.page/specifications/ALPM-MTREEv1.5.html
    V1(SchemaVersion),
    /// The [ALPM-MTREEv2] file format.
    ///
    /// [ALPM-MTREEv2]: https://alpm.archlinux.page/specifications/ALPM-MTREEv2.5.html
    V2(SchemaVersion),
}

impl FileFormatSchema for MtreeSchema {
    type Err = Error;

    /// Returns a reference to the inner [`SchemaVersion`].
    fn inner(&self) -> &SchemaVersion {
        match self {
            MtreeSchema::V1(v) | MtreeSchema::V2(v) => v,
        }
    }

    /// Derives an [`MtreeSchema`] from an ALPM-MTREE file.
    ///
    /// Opens the `file` and defers to [`MtreeSchema::derive_from_reader`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - opening `file` for reading fails
    /// - or deriving a [`MtreeSchema`] from the contents of `file` fails.
    fn derive_from_file(file: impl AsRef<Path>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let file = file.as_ref();
        Self::derive_from_reader(File::open(file).map_err(|source| Error::IoPath {
            path: PathBuf::from(file),
            context: t!("error-io-derive-schema"),
            source,
        })?)
    }

    /// Derives an [`MtreeSchema`] from ALPM-MTREE data in a `reader`.
    ///
    /// Reads the `reader` to string and defers to [`MtreeSchema::derive_from_str`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - reading a [`String`] from `reader` fails
    /// - or deriving a [`MtreeSchema`] from the contents of `reader` fails.
    fn derive_from_reader(reader: impl std::io::Read) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut buffer = Vec::new();
        let mut buf_reader = BufReader::new(reader);
        buf_reader
            .read_to_end(&mut buffer)
            .map_err(|source| Error::Io {
                context: t!("error-io-read-mtree-data"),
                source,
            })?;
        Self::derive_from_str(&mtree_buffer_to_string(buffer)?)
    }

    /// Derives an [`MtreeSchema`] from a string slice containing ALPM-MTREE data.
    ///
    /// Since the ALPM-MTREE format does not carry any version information, this function checks
    /// whether `s` contains `md5=` or `md5digest=`.
    /// If it does, the input is considered to be [ALPM-MTREEv2].
    /// If the strings are not found, [ALPM-MTREEv1] is assumed.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_common::FileFormatSchema;
    /// use alpm_mtree::MtreeSchema;
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> Result<(), alpm_mtree::Error> {
    /// let mtree_v2 = r#"
    /// #mtree
    /// /set mode=644 uid=0 gid=0 type=file
    /// ./some_file time=1700000000.0 size=1337 sha256digest=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
    /// ./some_link type=link link=some_file time=1700000000.0
    /// ./some_dir type=dir time=1700000000.0
    /// "#;
    /// assert_eq!(
    ///     MtreeSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))),
    ///     MtreeSchema::derive_from_str(mtree_v2)?
    /// );
    ///
    /// let mtree_v1 = r#"
    /// #mtree
    /// /set mode=644 uid=0 gid=0 type=file
    /// ./some_file time=1700000000.0 size=1337 sha256digest=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef md5digest=d3b07384d113edec49eaa6238ad5ff00
    /// ./some_link type=link link=some_file time=1700000000.0
    /// ./some_dir type=dir time=1700000000.0
    /// "#;
    /// assert_eq!(
    ///     MtreeSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))),
    ///     MtreeSchema::derive_from_str(mtree_v1)?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - the first `xdata` keyword is assigned an empty string,
    /// - or the first `xdata` keyword does not assign "pkgtype".
    ///
    /// [ALPM-MTREEv1]: https://alpm.archlinux.page/specifications/ALPM-MTREEv1.5.html
    /// [ALPM-MTREEv2]: https://alpm.archlinux.page/specifications/ALPM-MTREEv2.5.html
    fn derive_from_str(s: &str) -> Result<MtreeSchema, Error> {
        Ok(if s.contains("md5digest=") || s.contains("md5=") {
            MtreeSchema::V1(SchemaVersion::new(Version::new(1, 0, 0)))
        } else {
            MtreeSchema::V2(SchemaVersion::new(Version::new(2, 0, 0)))
        })
    }
}

impl Default for MtreeSchema {
    /// Returns the default [`MtreeSchema`] variant ([`MtreeSchema::V2`]).
    fn default() -> Self {
        Self::V2(SchemaVersion::new(Version::new(2, 0, 0)))
    }
}

impl FromStr for MtreeSchema {
    type Err = Error;

    /// Creates an [`MtreeSchema`] from string slice `s`.
    ///
    /// Relies on [`SchemaVersion::from_str`] to create a corresponding [`MtreeSchema`] from
    /// `s`.
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - no [`SchemaVersion`] can be created from `s`,
    /// - or the conversion from [`SchemaVersion`] to [`MtreeSchema`] fails.
    fn from_str(s: &str) -> Result<MtreeSchema, Self::Err> {
        match SchemaVersion::from_str(s) {
            Ok(version) => Self::try_from(version),
            Err(_) => Err(Error::UnsupportedSchemaVersion(s.to_string())),
        }
    }
}

impl TryFrom<SchemaVersion> for MtreeSchema {
    type Error = Error;

    /// Converts a [`SchemaVersion`] to an [`MtreeSchema`].
    ///
    /// # Errors
    ///
    /// Returns an error if the [`SchemaVersion`]'s inner [`Version`] does not provide a major
    /// version that corresponds to an [`MtreeSchema`] variant.
    fn try_from(value: SchemaVersion) -> Result<Self, Self::Error> {
        match value.inner().major {
            1 => Ok(MtreeSchema::V1(value)),
            2 => Ok(MtreeSchema::V2(value)),
            _ => Err(Error::UnsupportedSchemaVersion(value.to_string())),
        }
    }
}

impl Display for MtreeSchema {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                MtreeSchema::V1(version) | MtreeSchema::V2(version) => version.inner().major,
            }
        )
    }
}
