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

use crate::Error;

/// An enum describing all valid BUILDINFO schemas
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BuildInfoSchema {
    /// Schema for the [BUILDINFOv1] file format.
    ///
    /// [BUILDINFOv1]: https://alpm.archlinux.page/specifications/BUILDINFOv1.5.html
    V1(SchemaVersion),
    /// Schema for the [BUILDINFOv2] file format.
    ///
    /// [BUILDINFOv2]: https://alpm.archlinux.page/specifications/BUILDINFOv2.5.html
    V2(SchemaVersion),
}

impl FileFormatSchema for BuildInfoSchema {
    type Err = Error;

    /// Returns the schema version
    fn inner(&self) -> &SchemaVersion {
        match self {
            BuildInfoSchema::V1(v) => v,
            BuildInfoSchema::V2(v) => v,
        }
    }

    /// Derives a [`BuildInfoSchema`] from a BUILDINFO file.
    ///
    /// Opens the `file` and defers to [`BuildInfoSchema::derive_from_reader`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - opening `file` for reading fails
    /// - or deriving a [`BuildInfoSchema`] from the contents of `file` fails.
    fn derive_from_file(file: impl AsRef<Path>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let file = file.as_ref();
        Self::derive_from_reader(File::open(file).map_err(|source| {
            Error::IoPathError(
                PathBuf::from(file),
                "deriving schema version from BUILDINFO file",
                source,
            )
        })?)
    }

    /// Derives a [`BuildInfoSchema`] from BUILDINFO data in a `reader`.
    ///
    /// Reads the `reader` to string and defers to [`BuildInfoSchema::derive_from_str`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - reading a [`String`] from `reader` fails
    /// - or deriving a [`BuildInfoSchema`] from the contents of `reader` fails.
    fn derive_from_reader(reader: impl std::io::Read) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut buf = String::new();
        let mut reader = reader;
        reader
            .read_to_string(&mut buf)
            .map_err(|source| Error::IoReadError {
                context: "deriving schema version from BUILDINFO data",
                source,
            })?;
        Self::derive_from_str(&buf)
    }

    /// Derives a [`BuildInfoSchema`] from a string slice containing BUILDINFO data.
    ///
    /// Relies on the `format` keyword and its assigned value in the BUILDINFO data to derive a
    /// corresponding [`BuildInfoSchema`].
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_buildinfo::BuildInfoSchema;
    /// use alpm_common::FileFormatSchema;
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> Result<(), alpm_buildinfo::Error> {
    /// let buildinfo_v2 = r#"format = 2
    /// builddate = 1
    /// builddir = /build
    /// startdir = /startdir
    /// buildtool = devtools
    /// buildtoolver = 1:1.2.1-1-any
    /// buildenv = ccache
    /// installed = bar-1.2.3-1-any
    /// options = lto
    /// packager = Foobar McFooface <foobar@mcfooface.org>
    /// pkgarch = any
    /// pkgbase = foo
    /// pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
    /// pkgname = foo
    /// pkgver = 1:1.0.0-1"#;
    ///
    /// assert_eq!(
    ///     BuildInfoSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))),
    ///     BuildInfoSchema::derive_from_str(buildinfo_v2)?
    /// );
    ///
    /// let buildinfo_v1 = r#"format = 1
    /// builddate = 1
    /// builddir = /build
    /// startdir = /startdir
    /// buildenv = ccache
    /// installed = bar-1.2.3-1-any
    /// options = lto
    /// packager = Foobar McFooface <foobar@mcfooface.org>
    /// pkgarch = any
    /// pkgbase = foo
    /// pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
    /// pkgname = foo
    /// pkgver = 1:1.0.0-1"#;
    ///
    /// assert_eq!(
    ///     BuildInfoSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))),
    ///     BuildInfoSchema::derive_from_str(buildinfo_v1)?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - the `format` field is missing from `s`
    /// - or deriving a [`BuildInfoSchema`] from `format` field fails.
    fn derive_from_str(s: &str) -> Result<BuildInfoSchema, Error> {
        // Deserialize the file into a simple map, so we can take a look at the `format` string
        // that determines the buildinfo version.
        let raw_buildinfo: HashMap<String, Item> = alpm_parsers::custom_ini::from_str(s)?;
        if let Some(Item::Value(version)) = raw_buildinfo.get("format") {
            Self::from_str(version)
        } else {
            Err(Error::MissingFormatField)
        }
    }
}

impl Default for BuildInfoSchema {
    /// Returns the default [`BuildInfoSchema`] variant ([`BuildInfoSchema::V2`])
    fn default() -> Self {
        Self::V2(SchemaVersion::new(Version::new(2, 0, 0)))
    }
}

impl FromStr for BuildInfoSchema {
    type Err = Error;

    /// Creates a [`BuildInfoSchema`] from string slice `s`.
    ///
    /// Relies on [`SchemaVersion::from_str`] to create a corresponding [`BuildInfoSchema`] from
    /// `s`.
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - no [`SchemaVersion`] can be created from `s`,
    /// - or the conversion from [`SchemaVersion`] to [`BuildInfoSchema`] fails.
    fn from_str(s: &str) -> Result<BuildInfoSchema, Self::Err> {
        match SchemaVersion::from_str(s) {
            Ok(version) => Self::try_from(version),
            Err(_) => Err(Error::UnsupportedSchemaVersion(s.to_string())),
        }
    }
}

impl TryFrom<SchemaVersion> for BuildInfoSchema {
    type Error = Error;

    /// Converts a [`SchemaVersion`] to a [`BuildInfoSchema`].
    ///
    /// # Errors
    ///
    /// Returns an error if the [`SchemaVersion`]'s inner [`Version`] does not provide a major
    /// version that corresponds to a [`BuildInfoSchema`] variant.
    fn try_from(value: SchemaVersion) -> Result<Self, Self::Error> {
        match value.inner().major {
            1 => Ok(BuildInfoSchema::V1(value)),
            2 => Ok(BuildInfoSchema::V2(value)),
            _ => Err(Error::UnsupportedSchemaVersion(value.to_string())),
        }
    }
}

impl Display for BuildInfoSchema {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                BuildInfoSchema::V1(version) | BuildInfoSchema::V2(version) =>
                    version.inner().major,
            }
        )
    }
}
