//! Handling of BuildInfo versions.

mod format;
pub mod v1;
pub mod v2;

use std::{
    fmt::Display,
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_common::{FileFormatSchema, MetadataFile};

use crate::{BuildInfoSchema, BuildInfoV1, BuildInfoV2, Error};

/// A representation of the [BUILDINFO] file format.
///
/// Tracks all available variants of the file format.
///
/// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(untagged)]
pub enum BuildInfo {
    /// The [BUILDINFOv1] file format.
    ///
    /// [BUILDINFOv1]: https://alpm.archlinux.page/specifications/BUILDINFOv1.5.html
    V1(BuildInfoV1),
    /// The [BUILDINFOv2] file format.
    ///
    /// [BUILDINFOv2]: https://alpm.archlinux.page/specifications/BUILDINFOv2.5.html
    V2(BuildInfoV2),
}

impl MetadataFile<BuildInfoSchema> for BuildInfo {
    type Err = Error;

    /// Creates a [`BuildInfo`] from `file`, optionally validated using a [`BuildInfoSchema`].
    ///
    /// Opens the `file` and defers to [`BuildInfo::from_reader_with_schema`].
    ///
    /// # Note
    ///
    /// To automatically derive the [`BuildInfoSchema`], use [`BuildInfo::from_file`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_buildinfo::{BuildInfo, BuildInfoSchema};
    /// use alpm_common::{FileFormatSchema, MetadataFile};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// // Prepare a file with BUILDINFO data
    /// let (file, buildinfo_data) = {
    ///     let buildinfo_data = r#"format = 2
    /// pkgname = foo
    /// pkgbase = foo
    /// pkgver = 1:1.0.0-1
    /// pkgarch = any
    /// pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
    /// packager = Foobar McFooface <foobar@mcfooface.org>
    /// builddate = 1
    /// builddir = /build
    /// startdir = /startdir/
    /// buildtool = devtools
    /// buildtoolver = 1:1.2.1-1-any
    /// buildenv = ccache
    /// options = lto
    /// installed = bar-1.2.3-1-any
    /// "#;
    ///     let file = tempfile::NamedTempFile::new()?;
    ///     let mut output = File::create(&file)?;
    ///     write!(output, "{}", buildinfo_data)?;
    ///     (file, buildinfo_data)
    /// };
    ///
    /// let buildinfo = BuildInfo::from_file_with_schema(
    ///     file.path(),
    ///     Some(BuildInfoSchema::V2(SchemaVersion::new(Version::new(
    ///         2, 0, 0,
    ///     )))),
    /// )?;
    /// assert_eq!(buildinfo.to_string(), buildinfo_data);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - the `file` cannot be opened for reading,
    /// - no variant of [`BuildInfo`] can be constructed from the contents of `file`,
    /// - or `schema` is [`Some`] and the [`BuildInfoSchema`] does not match the contents of `file`.
    fn from_file_with_schema(
        file: impl AsRef<Path>,
        schema: Option<BuildInfoSchema>,
    ) -> Result<Self, Error> {
        let file = file.as_ref();
        Self::from_reader_with_schema(
            File::open(file).map_err(|source| {
                Error::IoPathError(PathBuf::from(file), "opening the file for reading", source)
            })?,
            schema,
        )
    }

    /// Creates a [`BuildInfo`] from a `reader`, optionally validated using a [`BuildInfoSchema`].
    ///
    /// Reads the `reader` to string and defers to [`BuildInfo::from_str_with_schema`].
    ///
    /// # Note
    ///
    /// To automatically derive the [`BuildInfoSchema`], use [`BuildInfo::from_reader`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_buildinfo::{BuildInfo, BuildInfoSchema};
    /// use alpm_common::MetadataFile;
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// // Prepare a reader with BUILDINFO data
    /// let (reader, buildinfo_data) = {
    ///     let buildinfo_data = r#"format = 2
    /// pkgname = foo
    /// pkgbase = foo
    /// pkgver = 1:1.0.0-1
    /// pkgarch = any
    /// pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
    /// packager = Foobar McFooface <foobar@mcfooface.org>
    /// builddate = 1
    /// builddir = /build
    /// startdir = /startdir/
    /// buildtool = devtools
    /// buildtoolver = 1:1.2.1-1-any
    /// buildenv = ccache
    /// options = lto
    /// installed = bar-1.2.3-1-any
    /// "#;
    ///     let buildinfo_file = tempfile::NamedTempFile::new()?;
    ///     let mut output = File::create(&buildinfo_file)?;
    ///     write!(output, "{}", buildinfo_data)?;
    ///     (File::open(&buildinfo_file.path())?, buildinfo_data)
    /// };
    ///
    /// let buildinfo = BuildInfo::from_reader_with_schema(
    ///     reader,
    ///     Some(BuildInfoSchema::V2(SchemaVersion::new(Version::new(
    ///         2, 0, 0,
    ///     )))),
    /// )?;
    /// assert_eq!(buildinfo.to_string(), buildinfo_data);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - the `reader` cannot be read to string,
    /// - no variant of [`BuildInfo`] can be constructed from the contents of the `reader`,
    /// - or `schema` is [`Some`] and the [`BuildInfoSchema`] does not match the contents of the
    ///   `reader`.
    fn from_reader_with_schema(
        mut reader: impl std::io::Read,
        schema: Option<BuildInfoSchema>,
    ) -> Result<Self, Error> {
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .map_err(|source| Error::IoReadError {
                context: "reading BuildInfo data",
                source,
            })?;
        Self::from_str_with_schema(&buf, schema)
    }

    /// Creates a [`BuildInfo`] from string slice, optionally validated using a [`BuildInfoSchema`].
    ///
    /// If `schema` is [`None`] attempts to detect the [`BuildInfoSchema`] from `s`.
    /// Attempts to create a [`BuildInfo`] variant that corresponds to the [`BuildInfoSchema`].
    ///
    /// # Note
    ///
    /// To automatically derive the [`BuildInfoSchema`], use [`BuildInfo::from_str`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_buildinfo::{BuildInfo, BuildInfoSchema};
    /// use alpm_common::MetadataFile;
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// let buildinfo_v2_data = r#"format = 2
    /// pkgname = foo
    /// pkgbase = foo
    /// pkgver = 1:1.0.0-1
    /// pkgarch = any
    /// pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
    /// packager = Foobar McFooface <foobar@mcfooface.org>
    /// builddate = 1
    /// builddir = /build
    /// startdir = /startdir/
    /// buildtool = devtools
    /// buildtoolver = 1:1.2.1-1-any
    /// buildenv = ccache
    /// options = lto
    /// installed = bar-1.2.3-1-any
    /// "#;
    ///
    /// let buildinfo_v2 = BuildInfo::from_str_with_schema(
    ///     buildinfo_v2_data,
    ///     Some(BuildInfoSchema::V2(SchemaVersion::new(Version::new(
    ///         2, 0, 0,
    ///     )))),
    /// )?;
    /// assert_eq!(buildinfo_v2.to_string(), buildinfo_v2_data);
    ///
    /// let buildinfo_v1_data = r#"format = 1
    /// pkgname = foo
    /// pkgbase = foo
    /// pkgver = 1:1.0.0-1
    /// pkgarch = any
    /// pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
    /// packager = Foobar McFooface <foobar@mcfooface.org>
    /// builddate = 1
    /// builddir = /build
    /// buildenv = ccache
    /// options = lto
    /// installed = bar-1.2.3-1-any
    /// "#;
    ///
    /// let buildinfo_v1 = BuildInfo::from_str_with_schema(
    ///     buildinfo_v1_data,
    ///     Some(BuildInfoSchema::V1(SchemaVersion::new(Version::new(
    ///         1, 0, 0,
    ///     )))),
    /// )?;
    /// assert_eq!(buildinfo_v1.to_string(), buildinfo_v1_data);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - `schema` is [`Some`] and the specified variant of [`BuildInfo`] cannot be constructed from
    ///   `s`,
    /// - `schema` is [`None`] and
    ///   - a [`BuildInfoSchema`] cannot be derived from `s`,
    ///   - or the detected variant of [`BuildInfo`] cannot be constructed from `s`.
    fn from_str_with_schema(s: &str, schema: Option<BuildInfoSchema>) -> Result<Self, Error> {
        let schema = match schema {
            Some(schema) => schema,
            None => BuildInfoSchema::derive_from_str(s)?,
        };

        match schema {
            BuildInfoSchema::V1(_) => Ok(BuildInfo::V1(BuildInfoV1::from_str(s)?)),
            BuildInfoSchema::V2(_) => Ok(BuildInfo::V2(BuildInfoV2::from_str(s)?)),
        }
    }
}

impl Display for BuildInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::V1(buildinfo) => buildinfo.to_string(),
                Self::V2(buildinfo) => buildinfo.to_string(),
            },
        )
    }
}

impl FromStr for BuildInfo {
    type Err = Error;

    /// Creates a [`BuildInfo`] from string slice `s`.
    ///
    /// Calls [`BuildInfo::from_str_with_schema`] with `schema` set to [`None`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - a [`BuildInfoSchema`] cannot be derived from `s`,
    /// - or the detected variant of [`BuildInfo`] cannot be constructed from `s`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_with_schema(s, None)
    }
}
