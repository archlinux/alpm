//! High-level PKGINFO handling.

pub mod v1;
pub mod v2;
use std::{
    fmt::Display,
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_common::{FileFormatSchema, MetadataFile};

use crate::{Error, PackageInfoSchema, PackageInfoV1, PackageInfoV2};

/// A representation of the [PKGINFO] file format.
///
/// Tracks all available versions of the file format.
///
/// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
#[derive(Clone, Debug, serde::Serialize)]
#[serde(untagged)]
pub enum PackageInfo {
    V1(PackageInfoV1),
    V2(PackageInfoV2),
}

impl MetadataFile<PackageInfoSchema> for PackageInfo {
    type Err = Error;

    /// Creates a [`PackageInfo`] from `file`, optionally validated using a [`PackageInfoSchema`].
    ///
    /// Opens the `file` and defers to [`PackageInfo::from_reader_with_schema`].
    ///
    /// # Note
    ///
    /// To automatically derive the [`PackageInfoSchema`], use [`PackageInfo::from_file`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_common::{FileFormatSchema, MetadataFile};
    /// use alpm_pkginfo::{PackageInfo, PackageInfoSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// // Prepare a file with PKGINFO data
    /// let (file, pkginfo_data) = {
    ///     let pkginfo_data = r#"pkgname = example
    /// pkgbase = example
    /// xdata = pkgtype=pkg
    /// pkgver = 1:1.0.0-1
    /// pkgdesc = A project that does something
    /// url = https://example.org/
    /// builddate = 1729181726
    /// packager = John Doe <john@example.org>
    /// size = 181849963
    /// arch = any
    /// "#;
    ///     let pkginfo_file = tempfile::NamedTempFile::new()?;
    ///     let mut output = File::create(&pkginfo_file)?;
    ///     write!(output, "{}", pkginfo_data)?;
    ///     (pkginfo_file, pkginfo_data)
    /// };
    ///
    /// let pkginfo = PackageInfo::from_file_with_schema(
    ///     file.path().to_path_buf(),
    ///     Some(PackageInfoSchema::V2(SchemaVersion::new(Version::new(
    ///         2, 0, 0,
    ///     )))),
    /// )?;
    /// assert_eq!(pkginfo.to_string(), pkginfo_data);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - the `file` cannot be opened for reading,
    /// - no variant of [`PackageInfo`] can be constructed from the contents of `file`,
    /// - or `schema` is [`Some`] and the [`PackageInfoSchema`] does not match the contents of
    ///   `file`.
    fn from_file_with_schema(
        file: impl AsRef<Path>,
        schema: Option<PackageInfoSchema>,
    ) -> Result<Self, Error> {
        let file = file.as_ref();
        Self::from_reader_with_schema(
            File::open(file).map_err(|source| {
                Error::IoPathError(PathBuf::from(file), "opening the file for reading", source)
            })?,
            schema,
        )
    }

    /// Creates a [`PackageInfo`] from a `reader`, optionally validated using a
    /// [`PackageInfoSchema`].
    ///
    /// Reads the `reader` to string and defers to [`PackageInfo::from_str_with_schema`].
    ///
    /// # Note
    ///
    /// To automatically derive the [`PackageInfoSchema`], use [`PackageInfo::from_reader`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_common::MetadataFile;
    /// use alpm_pkginfo::{PackageInfo, PackageInfoSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// // Prepare a reader with PKGINFO data
    /// let (reader, pkginfo_data) = {
    ///     let pkginfo_data = r#"pkgname = example
    /// pkgbase = example
    /// xdata = pkgtype=pkg
    /// pkgver = 1:1.0.0-1
    /// pkgdesc = A project that does something
    /// url = https://example.org/
    /// builddate = 1729181726
    /// packager = John Doe <john@example.org>
    /// size = 181849963
    /// arch = any
    /// "#;
    ///     let pkginfo_file = tempfile::NamedTempFile::new()?;
    ///     let mut output = File::create(&pkginfo_file)?;
    ///     write!(output, "{}", pkginfo_data)?;
    ///     (File::open(&pkginfo_file.path())?, pkginfo_data)
    /// };
    ///
    /// let pkginfo = PackageInfo::from_reader_with_schema(
    ///     reader,
    ///     Some(PackageInfoSchema::V2(SchemaVersion::new(Version::new(
    ///         2, 0, 0,
    ///     )))),
    /// )?;
    /// assert_eq!(pkginfo.to_string(), pkginfo_data);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - the `reader` cannot be read to string,
    /// - no variant of [`PackageInfo`] can be constructed from the contents of the `reader`,
    /// - or `schema` is [`Some`] and the [`PackageInfoSchema`] does not match the contents of the
    ///   `reader`.
    fn from_reader_with_schema(
        mut reader: impl std::io::Read,
        schema: Option<PackageInfoSchema>,
    ) -> Result<Self, Error> {
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .map_err(|source| Error::IoReadError {
                context: "reading PackageInfo data",
                source,
            })?;
        Self::from_str_with_schema(&buf, schema)
    }

    /// Creates a [`PackageInfo`] from string slice, optionally validated using a
    /// [`PackageInfoSchema`].
    ///
    /// If `schema` is [`None`] attempts to detect the [`PackageInfoSchema`] from `s`.
    /// Attempts to create a [`PackageInfo`] variant that corresponds to the [`PackageInfoSchema`].
    ///
    /// # Note
    ///
    /// To automatically derive the [`PackageInfoSchema`], use [`PackageInfo::from_str`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_common::MetadataFile;
    /// use alpm_pkginfo::{PackageInfo, PackageInfoSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// let pkginfo_v2_data = r#"pkgname = example
    /// pkgbase = example
    /// xdata = pkgtype=pkg
    /// pkgver = 1:1.0.0-1
    /// pkgdesc = A project that does something
    /// url = https://example.org/
    /// builddate = 1729181726
    /// packager = John Doe <john@example.org>
    /// size = 181849963
    /// arch = any
    /// "#;
    ///
    /// let pkginfo_v2 = PackageInfo::from_str_with_schema(
    ///     pkginfo_v2_data,
    ///     Some(PackageInfoSchema::V2(SchemaVersion::new(Version::new(
    ///         2, 0, 0,
    ///     )))),
    /// )?;
    /// assert_eq!(pkginfo_v2.to_string(), pkginfo_v2_data);
    ///
    /// let pkginfo_v1_data = r#"pkgname = example
    /// pkgbase = example
    /// pkgver = 1:1.0.0-1
    /// pkgdesc = A project that does something
    /// url = https://example.org/
    /// builddate = 1729181726
    /// packager = John Doe <john@example.org>
    /// size = 181849963
    /// arch = any
    /// "#;
    ///
    /// let pkginfo_v1 = PackageInfo::from_str_with_schema(
    ///     pkginfo_v1_data,
    ///     Some(PackageInfoSchema::V1(SchemaVersion::new(Version::new(
    ///         1, 0, 0,
    ///     )))),
    /// )?;
    /// assert_eq!(pkginfo_v1.to_string(), pkginfo_v1_data);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - `schema` is [`Some`] and the specified variant of [`PackageInfo`] cannot be constructed
    ///   from `s`,
    /// - `schema` is [`None`] and
    ///   - a [`PackageInfoSchema`] cannot be derived from `s`,
    ///   - or the detected variant of [`PackageInfo`] cannot be constructed from `s`.
    fn from_str_with_schema(s: &str, schema: Option<PackageInfoSchema>) -> Result<Self, Error> {
        let schema = match schema {
            Some(schema) => schema,
            None => PackageInfoSchema::derive_from_str(s)?,
        };

        match schema {
            PackageInfoSchema::V1(_) => Ok(PackageInfo::V1(PackageInfoV1::from_str(s)?)),
            PackageInfoSchema::V2(_) => Ok(PackageInfo::V2(PackageInfoV2::from_str(s)?)),
        }
    }
}

impl Display for PackageInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::V1(pkginfo) => pkginfo.to_string(),
                Self::V2(pkginfo) => pkginfo.to_string(),
            },
        )
    }
}

impl FromStr for PackageInfo {
    type Err = Error;

    /// Creates a [`PackageInfo`] from string slice `s`.
    ///
    /// Calls [`PackageInfo::from_str_with_schema`] with `schema` set to [`None`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - a [`PackageInfoSchema`] cannot be derived from `s`,
    /// - or the detected variant of [`PackageInfo`] cannot be constructed from `s`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_with_schema(s, None)
    }
}
