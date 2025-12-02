//! Schemas for SRCINFO data.

use std::{
    fmt::{Display, Formatter},
    fs::File,
    path::Path,
    str::FromStr,
};

use alpm_common::FileFormatSchema;
use alpm_types::{SchemaVersion, semver_version::Version};
use fluent_i18n::t;
use winnow::Parser;

use crate::{Error, source_info::parser::SourceInfoContent};

/// An enum tracking all available [SRCINFO] schemas.
///
/// The schema of a SRCINFO refers to the minimum required sections and keywords, as well as the
/// complete set of available keywords in a specific version.
///
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceInfoSchema {
    /// Schema for the [SRCINFO] file format.
    ///
    /// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
    V1(SchemaVersion),
}

impl FileFormatSchema for SourceInfoSchema {
    type Err = Error;

    /// Returns a reference to the inner [`SchemaVersion`].
    fn inner(&self) -> &SchemaVersion {
        match self {
            SourceInfoSchema::V1(v) => v,
        }
    }

    /// Derives a [`SourceInfoSchema`] from a SRCINFO file.
    ///
    /// Opens the `file` and defers to [`SourceInfoSchema::derive_from_reader`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - opening `file` for reading fails
    /// - or deriving a [`SourceInfoSchema`] from the contents of `file` fails.
    fn derive_from_file(file: impl AsRef<Path>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let file = file.as_ref();
        Self::derive_from_reader(File::open(file).map_err(|source| Error::IoPath {
            path: file.to_path_buf(),
            context: t!("error-io-deriving-schema-from-srcinfo-file"),
            source,
        })?)
    }

    /// Derives a [`SourceInfoSchema`] from SRCINFO data in a `reader`.
    ///
    /// Reads the `reader` to string and defers to [`SourceInfoSchema::derive_from_str`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - reading a [`String`] from `reader` fails
    /// - or deriving a [`SourceInfoSchema`] from the contents of `reader` fails.
    fn derive_from_reader(reader: impl std::io::Read) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut buf = String::new();
        let mut reader = reader;
        reader
            .read_to_string(&mut buf)
            .map_err(|source| Error::Io {
                context: t!("error-io-deriving-schema-from-srcinfo-data"),
                source,
            })?;
        Self::derive_from_str(&buf)
    }

    /// Derives a [`SourceInfoSchema`] from a string slice containing SRCINFO data.
    ///
    /// Since the SRCINFO format is only covered by a single version and it not carrying any
    /// version information, this function checks whether `s` contains at least the sections
    /// `pkgbase` and `pkgname` and the keywords `pkgver` and `pkgrel`.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_common::FileFormatSchema;
    /// use alpm_srcinfo::SourceInfoSchema;
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> Result<(), alpm_srcinfo::Error> {
    /// let srcinfo_data = r#"
    /// pkgbase = example
    ///     pkgdesc = An example
    ///     pkgver = 0.1.0
    ///     pkgrel = 1
    ///
    /// pkgname = example
    /// "#;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if `s` cannot be parsed.
    fn derive_from_str(s: &str) -> Result<SourceInfoSchema, Error> {
        let _parsed = SourceInfoContent::parser
            // A temporary fix for <https://github.com/winnow-rs/winnow/issues/847>
            .parse(s.replace('\t', " ").as_str())
            .map_err(|err| Error::ParseError(format!("{err}")))?;

        Ok(SourceInfoSchema::V1(SchemaVersion::new(Version::new(
            1, 0, 0,
        ))))
    }
}

impl Default for SourceInfoSchema {
    /// Returns the default [`SourceInfoSchema`] variant ([`SourceInfoSchema::V1`]).
    fn default() -> Self {
        Self::V1(SchemaVersion::new(Version::new(1, 0, 0)))
    }
}

impl FromStr for SourceInfoSchema {
    type Err = Error;

    /// Creates a [`SourceInfoSchema`] from string slice `s`.
    ///
    /// Relies on [`SchemaVersion::from_str`] to create a corresponding [`SourceInfoSchema`] from
    /// `s`.
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - no [`SchemaVersion`] can be created from `s`,
    /// - or the conversion from [`SchemaVersion`] to [`SourceInfoSchema`] fails.
    fn from_str(s: &str) -> Result<SourceInfoSchema, Self::Err> {
        match SchemaVersion::from_str(s) {
            Ok(version) => Self::try_from(version),
            Err(_) => Err(Error::UnsupportedSchemaVersion(s.to_string())),
        }
    }
}

impl TryFrom<SchemaVersion> for SourceInfoSchema {
    type Error = Error;

    /// Converts a [`SchemaVersion`] to a [`SourceInfoSchema`].
    ///
    /// # Errors
    ///
    /// Returns an error if the [`SchemaVersion`]'s inner [`Version`] does not provide a major
    /// version that corresponds to a [`SourceInfoSchema`] variant.
    fn try_from(value: SchemaVersion) -> Result<Self, Self::Error> {
        match value.inner().major {
            1 => Ok(SourceInfoSchema::V1(value)),
            _ => Err(Error::UnsupportedSchemaVersion(value.to_string())),
        }
    }
}

impl Display for SourceInfoSchema {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                SourceInfoSchema::V1(version) => version.inner().major,
            }
        )
    }
}
