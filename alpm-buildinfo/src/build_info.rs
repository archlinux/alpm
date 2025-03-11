//! Handling of BuildInfo versions.

pub mod v1;
pub mod v2;
use std::{
    fmt::Display,
    fs::File,
    io::stdin,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{BuildInfoV1, BuildInfoV2, Error, Schema};

/// A representation of the [BUILDINFO] file format.
///
/// Tracks all available variants of the file format.
///
/// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
#[derive(Clone, Debug, serde::Serialize)]
#[serde(untagged)]
pub enum BuildInfo {
    V1(BuildInfoV1),
    V2(BuildInfoV2),
}

impl BuildInfo {
    /// Creates a [`BuildInfo`] from `file`.
    ///
    /// Optionally, `schema` is used to validate the created [`BuildInfo`] variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_buildinfo::BuildInfo;
    /// use tempfile::NamedTempFile;
    ///
    /// # fn main() -> testresult::TestResult {
    /// // Prepare a file with BUILDINFO data
    /// let buildinfo_data = r#"format = 1
    /// pkgname = foo
    /// pkgbase = foo
    /// pkgver = 1:1.0.0-1
    /// pkgarch = any
    /// pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
    /// packager = Foobar McFooface <foobar@mcfooface.org>
    /// builddate = 1
    /// builddir = /build
    /// buildenv = envfoo
    /// options = some_option
    /// installed = bar-1.2.3-1-any
    /// "#;
    /// let buildinfo_file = NamedTempFile::new()?;
    /// let mut output = File::create(&buildinfo_file)?;
    /// write!(output, "{}", buildinfo_data)?;
    ///
    /// let buildinfo = BuildInfo::from_file(buildinfo_file.path(), None)?;
    /// assert_eq!(buildinfo.to_string(), buildinfo_data);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - the `file` cannot be opened for reading,
    /// - or no variant of [`BuildInfo`] can be constructed from the contents of `file`.
    pub fn from_file(file: impl AsRef<Path>, schema: Option<Schema>) -> Result<Self, Error> {
        let file = file.as_ref();
        Self::from_reader(
            File::open(file).map_err(|source| {
                Error::IoPathError(PathBuf::from(file), "opening the file for reading", source)
            })?,
            schema,
        )
    }

    /// Creates a [`BuildInfo`] from stdin.
    ///
    /// Optionally, `schema` is used to validate the created [`BuildInfo`] variant.
    ///
    /// # Errors
    ///
    /// Returns an error if no variant of [`BuildInfo`] can be constructed from the contents of
    /// stdin.
    pub fn from_stdin(schema: Option<Schema>) -> Result<Self, Error> {
        Self::from_reader(stdin(), schema)
    }

    /// Creates a [`BuildInfo`] from a `reader`.
    ///
    /// Optionally, `schema` is used to validate the created [`BuildInfo`] variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_buildinfo::BuildInfo;
    /// use tempfile::NamedTempFile;
    ///
    /// # fn main() -> testresult::TestResult {
    /// // Prepare a file with BUILDINFO data
    /// let buildinfo_data = r#"format = 1
    /// pkgname = foo
    /// pkgbase = foo
    /// pkgver = 1:1.0.0-1
    /// pkgarch = any
    /// pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
    /// packager = Foobar McFooface <foobar@mcfooface.org>
    /// builddate = 1
    /// builddir = /build
    /// buildenv = envfoo
    /// options = some_option
    /// installed = bar-1.2.3-1-any
    /// "#;
    /// let buildinfo_file = NamedTempFile::new()?;
    /// let mut output = File::create(&buildinfo_file)?;
    /// write!(output, "{}", buildinfo_data)?;
    ///
    /// // Prepare a reader with the BUILDINFO data and read it
    /// let file = File::open(&buildinfo_file.path())?;
    /// let buildinfo = BuildInfo::from_reader(file, None)?;
    /// assert_eq!(buildinfo.to_string(), buildinfo_data);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - the `reader` cannot be read to string,
    /// - or no variant of [`BuildInfo`] can be constructed from the contents of the `reader`.
    pub fn from_reader(
        mut reader: impl std::io::Read,
        schema: Option<Schema>,
    ) -> Result<Self, Error> {
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .map_err(|source| Error::IoReadError {
                context: "reading BuildInfo data",
                source,
            })?;

        if let Some(schema) = schema {
            Self::from_str_with_schema(&buf, schema)
        } else {
            Self::from_str(&buf)
        }
    }

    /// Creates a [`BuildInfo`] from string slice.
    ///
    /// Uses `schema` to enforce the creation of a specific [`BuildInfo`] variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_buildinfo::{BuildInfo, Schema};
    /// use tempfile::NamedTempFile;
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
    /// buildenv = envfoo
    /// options = some_option
    /// installed = bar-1.2.3-1-any
    /// "#;
    ///
    /// let buildinfo_v2 =
    ///     BuildInfo::from_str_with_schema(buildinfo_v2_data, Schema::V2("2".parse()?))?;
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
    /// buildenv = envfoo
    /// options = some_option
    /// installed = bar-1.2.3-1-any
    /// "#;
    ///
    /// let buildinfo_v1 =
    ///     BuildInfo::from_str_with_schema(buildinfo_v1_data, Schema::V1("1".parse()?))?;
    /// assert_eq!(buildinfo_v1.to_string(), buildinfo_v1_data);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the specific variant of [`BuildInfo`] cannot be constructed from `s`.
    pub fn from_str_with_schema(s: &str, schema: Schema) -> Result<Self, Error> {
        match schema {
            Schema::V1(_) => Ok(BuildInfo::V1(BuildInfoV1::from_str(s)?)),
            Schema::V2(_) => Ok(BuildInfo::V2(BuildInfoV2::from_str(s)?)),
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
    /// Attempts to automatically detect the used [`Schema`] version from `s`.
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - a [`Schema`] cannot be derived from `s`,
    /// - or the detected variant of [`BuildInfo`] cannot be constructed from `s`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Schema::from_contents(s)? {
            Schema::V1(_) => Ok(BuildInfo::V1(BuildInfoV1::from_str(s)?)),
            Schema::V2(_) => Ok(BuildInfo::V2(BuildInfoV2::from_str(s)?)),
        }
    }
}
