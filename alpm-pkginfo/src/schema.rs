//! Schemas for PKGINFO data.

use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_common::FileFormatSchema;
use alpm_parsers::custom_ini::parser::Item;
use alpm_types::{SchemaVersion, semver_version::Version};
use fluent_i18n::t;

use crate::Error;

/// An enum tracking all available [PKGINFO] schemas.
///
/// The schema of a PKGINFO refers to its available fields in a specific version.
///
/// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PackageInfoSchema {
    /// Schema for the [PKGINFOv1] file format.
    ///
    /// [PKGINFOv1]: https://alpm.archlinux.page/specifications/PKGINFOv1.5.html
    V1(SchemaVersion),
    /// Schema for the [PKGINFOv2] file format.
    ///
    /// [PKGINFOv2]: https://alpm.archlinux.page/specifications/PKGINFOv2.5.html
    V2(SchemaVersion),
}

impl FileFormatSchema for PackageInfoSchema {
    type Err = Error;

    /// Returns a reference to the inner [`SchemaVersion`].
    fn inner(&self) -> &SchemaVersion {
        match self {
            PackageInfoSchema::V1(v) | PackageInfoSchema::V2(v) => v,
        }
    }

    /// Derives a [`PackageInfoSchema`] from a PKGINFO file.
    ///
    /// Opens the `file` and defers to [`PackageInfoSchema::derive_from_reader`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - opening `file` for reading fails
    /// - or deriving a [`PackageInfoSchema`] from the contents of `file` fails.
    fn derive_from_file(file: impl AsRef<Path>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let file = file.as_ref();
        Self::derive_from_reader(File::open(file).map_err(|source| Error::IoPathError {
            path: PathBuf::from(file),
            context: t!("error-io-derive-schema-from-pkginfo"),
            source,
        })?)
    }

    /// Derives a [`PackageInfoSchema`] from PKGINFO data in a `reader`.
    ///
    /// Reads the `reader` to string and defers to [`PackageInfoSchema::derive_from_str`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - reading a [`String`] from `reader` fails
    /// - or deriving a [`PackageInfoSchema`] from the contents of `reader` fails.
    fn derive_from_reader(reader: impl std::io::Read) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut buf = String::new();
        let mut reader = reader;
        reader
            .read_to_string(&mut buf)
            .map_err(|source| Error::IoReadError {
                context: t!("error-io-derive-schema-from-pkginfo"),
                source,
            })?;
        Self::derive_from_str(&buf)
    }

    /// Derives a [`PackageInfoSchema`] from a string slice containing PKGINFO data.
    ///
    /// Since the PKGINFO format does not carry any version information, this function looks for the
    /// first `xdata` field (if any) to determine whether the input may be [PKGINFOv2].
    /// If no `xdata` field is found, [PKGINFOv1] is assumed.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_common::FileFormatSchema;
    /// use alpm_pkginfo::PackageInfoSchema;
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> Result<(), alpm_pkginfo::Error> {
    /// let pkginfo_v2 = r#"
    /// pkgname = example
    /// pkgbase = example
    /// pkgver = 1:1.0.0-1
    /// pkgdesc = A project that does something
    /// url = https://example.org/
    /// builddate = 1729181726
    /// packager = John Doe <john@example.org>
    /// size = 181849963
    /// arch = any
    /// xdata = pkgtype=pkg
    /// "#;
    /// assert_eq!(
    ///     PackageInfoSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))),
    ///     PackageInfoSchema::derive_from_str(pkginfo_v2)?
    /// );
    ///
    /// let pkginfo_v1 = r#"
    /// pkgname = example
    /// pkgbase = example
    /// pkgver = 1:1.0.0-1
    /// pkgdesc = A project that does something
    /// url = https://example.org/
    /// builddate = 1729181726
    /// packager = John Doe <john@example.org>
    /// size = 181849963
    /// arch = any
    /// "#;
    /// assert_eq!(
    ///     PackageInfoSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))),
    ///     PackageInfoSchema::derive_from_str(pkginfo_v1)?
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
    /// [PKGINFOv1]: https://alpm.archlinux.page/specifications/PKGINFOv1.5.html
    /// [PKGINFOv2]: https://alpm.archlinux.page/specifications/PKGINFOv2.5.html
    fn derive_from_str(s: &str) -> Result<PackageInfoSchema, Error> {
        // Deserialize the file into a simple map, so we can take a look at whether there is a
        // `xdata` string that indicates PKGINFOv2.
        let raw: HashMap<String, Item> = alpm_parsers::custom_ini::from_str(s)?;
        let value = match raw.get("xdata") {
            Some(Item::Value(value)) => Some(value),
            Some(Item::List(values)) => {
                if !values.is_empty() {
                    values.iter().next()
                } else {
                    return Err(Error::ExtraDataEmpty);
                }
            }
            None => return Ok(Self::V1(SchemaVersion::new(Version::new(1, 0, 0)))),
        };

        if let Some(value) = value {
            if value.starts_with("pkgtype") {
                Ok(Self::V2(SchemaVersion::new(Version::new(2, 0, 0))))
            } else {
                Err(Error::FirstExtraDataNotPkgType)
            }
        } else {
            Err(Error::ExtraDataEmpty)
        }
    }
}

impl Default for PackageInfoSchema {
    /// Returns the default [`PackageInfoSchema`] variant ([`PackageInfoSchema::V2`]).
    fn default() -> Self {
        Self::V2(SchemaVersion::new(Version::new(2, 0, 0)))
    }
}

impl FromStr for PackageInfoSchema {
    type Err = Error;

    /// Creates a [`PackageInfoSchema`] from string slice `s`.
    ///
    /// Relies on [`SchemaVersion::from_str`] to create a corresponding [`PackageInfoSchema`] from
    /// `s`.
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - no [`SchemaVersion`] can be created from `s`,
    /// - or the conversion from [`SchemaVersion`] to [`PackageInfoSchema`] fails.
    fn from_str(s: &str) -> Result<PackageInfoSchema, Self::Err> {
        match SchemaVersion::from_str(s) {
            Ok(version) => Self::try_from(version),
            Err(_) => Err(Error::UnsupportedSchemaVersion(s.to_string())),
        }
    }
}

impl TryFrom<SchemaVersion> for PackageInfoSchema {
    type Error = Error;

    /// Converts a [`SchemaVersion`] to a [`PackageInfoSchema`].
    ///
    /// # Errors
    ///
    /// Returns an error if the [`SchemaVersion`]'s inner [`Version`] does not provide a major
    /// version that corresponds to a [`PackageInfoSchema`] variant.
    fn try_from(value: SchemaVersion) -> Result<Self, Self::Error> {
        match value.inner().major {
            1 => Ok(PackageInfoSchema::V1(value)),
            2 => Ok(PackageInfoSchema::V2(value)),
            _ => Err(Error::UnsupportedSchemaVersion(value.to_string())),
        }
    }
}

impl Display for PackageInfoSchema {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                PackageInfoSchema::V1(version) | PackageInfoSchema::V2(version) =>
                    version.inner().major,
            }
        )
    }
}
