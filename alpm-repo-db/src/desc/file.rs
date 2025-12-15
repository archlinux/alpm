//! File handling for [alpm-repo-desc] files.
//!
//! [alpm-repo-desc]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html

use std::{
    fmt::Display,
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_common::{FileFormatSchema, MetadataFile, Named, RuntimeRelations, Versioned};
use alpm_types::{FullVersion, Name, OptionalDependency, PackageRelation, RelationOrSoname};
use fluent_i18n::t;

use crate::{
    Error,
    desc::{RepoDescFileV1, RepoDescFileV2, RepoDescSchema},
};

/// A representation of the [alpm-repo-desc] file format.
///
/// Tracks all supported schema versions (`v1` and `v2`) of the package repository description file.
/// Each variant corresponds to a distinct layout of the format.
///
/// [alpm-repo-desc]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(untagged)]
pub enum RepoDescFile {
    /// The [alpm-repo-descv1] file format.
    ///
    /// [alpm-repo-descv1]: https://alpm.archlinux.page/specifications/alpm-repo-descv1.5.html
    V1(RepoDescFileV1),
    /// The [alpm-repo-descv2] file format.
    ///
    /// This revision of the file format, removes %MD5SUM% and makes the %PGPSIG% section optional.
    ///
    /// [alpm-repo-descv2]: https://alpm.archlinux.page/specifications/alpm-repo-descv2.5.html
    V2(RepoDescFileV2),
}

impl MetadataFile<RepoDescSchema> for RepoDescFile {
    type Err = Error;

    /// Creates a [`RepoDescFile`] from a file on disk, optionally validated using a
    /// [`RepoDescSchema`].
    ///
    /// Opens the file and defers to [`RepoDescFile::from_reader_with_schema`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_common::{FileFormatSchema, MetadataFile};
    /// use alpm_repo_db::desc::{RepoDescFile, RepoDescSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// // Prepare a file with package repository desc data (v1)
    /// let (file, desc_data) = {
    ///     let desc_data = r#"%FILENAME%
    /// example-meta-1.0.0-1-any.pkg.tar.zst
    ///
    /// %NAME%
    /// example-meta
    ///
    /// %BASE%
    /// example-meta
    ///
    /// %VERSION%
    /// 1.0.0-1
    ///
    /// %DESC%
    /// An example meta package
    ///
    /// %CSIZE%
    /// 4634
    ///
    /// %ISIZE%
    /// 0
    ///
    /// %MD5SUM%
    /// d3b07384d113edec49eaa6238ad5ff00
    ///
    /// %SHA256SUM%
    /// b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
    ///
    /// %PGPSIG%
    /// iHUEABYKAB0WIQRizHP4hOUpV7L92IObeih9mi7GCAUCaBZuVAAKCRCbeih9mi7GCIlMAP9ws/jU4f580ZRQlTQKvUiLbAZOdcB7mQQj83hD1Nc/GwD/WIHhO1/OQkpMERejUrLo3AgVmY3b4/uGhx9XufWEbgE=
    ///
    /// %URL%
    /// https://example.org/
    ///
    /// %LICENSE%
    /// GPL-3.0-or-later
    ///
    /// %ARCH%
    /// any
    ///
    /// %BUILDDATE%
    /// 1729181726
    ///
    /// %PACKAGER%
    /// Foobar McFooface <foobar@mcfooface.org>
    ///
    /// "#;
    ///     let file = tempfile::NamedTempFile::new()?;
    ///     let mut output = File::create(&file)?;
    ///     write!(output, "{}", desc_data)?;
    ///     (file, desc_data)
    /// };
    ///
    /// let repo_desc = RepoDescFile::from_file_with_schema(
    ///     file.path(),
    ///     Some(RepoDescSchema::V1(SchemaVersion::new(Version::new(
    ///         1, 0, 0,
    ///     )))),
    /// )?;
    /// assert_eq!(repo_desc.to_string(), desc_data);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the file cannot be opened for reading,
    /// - the contents cannot be parsed into any known [`RepoDescFile`] variant,
    /// - or the provided [`RepoDescSchema`] does not match the contents of the file.
    fn from_file_with_schema(
        file: impl AsRef<Path>,
        schema: Option<RepoDescSchema>,
    ) -> Result<Self, Error> {
        let file = file.as_ref();
        Self::from_reader_with_schema(
            File::open(file).map_err(|source| Error::IoPathError {
                path: PathBuf::from(file),
                context: t!("error-io-path-open-file"),
                source,
            })?,
            schema,
        )
    }

    /// Creates a [`RepoDescFile`] from any readable stream, optionally validated using a
    /// [`RepoDescSchema`].
    ///
    /// Reads the `reader` to a string buffer and defers to [`RepoDescFile::from_str_with_schema`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_common::MetadataFile;
    /// use alpm_repo_db::desc::{RepoDescFile, RepoDescSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// // Prepare a reader with package repository desc data (v2)
    /// let (reader, desc_data) = {
    ///     let desc_data = r#"%FILENAME%
    /// example-meta-1.0.0-1-any.pkg.tar.zst
    ///
    /// %NAME%
    /// example-meta
    ///
    /// %BASE%
    /// example-meta
    ///
    /// %VERSION%
    /// 1.0.0-1
    ///
    /// %DESC%
    /// An example meta package
    ///
    /// %CSIZE%
    /// 4634
    ///
    /// %ISIZE%
    /// 0
    ///
    /// %SHA256SUM%
    /// b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
    ///
    /// %URL%
    /// https://example.org/
    ///
    /// %LICENSE%
    /// GPL-3.0-or-later
    ///
    /// %ARCH%
    /// any
    ///
    /// %BUILDDATE%
    /// 1729181726
    ///
    /// %PACKAGER%
    /// Foobar McFooface <foobar@mcfooface.org>
    ///
    /// "#;
    ///     let file = tempfile::NamedTempFile::new()?;
    ///     let mut output = File::create(&file)?;
    ///     write!(output, "{}", desc_data)?;
    ///     (File::open(&file.path())?, desc_data)
    /// };
    ///
    /// let repo_desc = RepoDescFile::from_reader_with_schema(
    ///     reader,
    ///     Some(RepoDescSchema::V2(SchemaVersion::new(Version::new(
    ///         2, 0, 0,
    ///     )))),
    /// )?;
    /// assert_eq!(repo_desc.to_string(), desc_data);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the `reader` cannot be read to string,
    /// - the data cannot be parsed into a known [`RepoDescFile`] variant,
    /// - or the provided [`RepoDescSchema`] does not match the parsed content.
    fn from_reader_with_schema(
        mut reader: impl std::io::Read,
        schema: Option<RepoDescSchema>,
    ) -> Result<Self, Error> {
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .map_err(|source| Error::IoReadError {
                context: t!("error-io-read-repo-desc"),
                source,
            })?;
        Self::from_str_with_schema(&buf, schema)
    }

    /// Creates a [`RepoDescFile`] from a string slice, optionally validated using a
    /// [`RepoDescSchema`].
    ///
    /// If `schema` is [`None`], automatically infers the schema version by inspecting the input
    /// (`v1` if `%MD5SUM%` is present, `v2` otherwise).
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_common::MetadataFile;
    /// use alpm_repo_db::desc::{RepoDescFile, RepoDescSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// let v1_data = r#"%FILENAME%
    /// example-meta-1.0.0-1-any.pkg.tar.zst
    ///
    /// %NAME%
    /// example-meta
    ///
    /// %BASE%
    /// example-meta
    ///
    /// %VERSION%
    /// 1.0.0-1
    ///
    /// %DESC%
    /// An example meta package
    ///
    /// %CSIZE%
    /// 4634
    ///
    /// %ISIZE%
    /// 0
    ///
    /// %MD5SUM%
    /// d3b07384d113edec49eaa6238ad5ff00
    ///
    /// %SHA256SUM%
    /// b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
    ///
    /// %PGPSIG%
    /// iHUEABYKAB0WIQRizHP4hOUpV7L92IObeih9mi7GCAUCaBZuVAAKCRCbeih9mi7GCIlMAP9ws/jU4f580ZRQlTQKvUiLbAZOdcB7mQQj83hD1Nc/GwD/WIHhO1/OQkpMERejUrLo3AgVmY3b4/uGhx9XufWEbgE=
    ///
    /// %URL%
    /// https://example.org/
    ///
    /// %LICENSE%
    /// GPL-3.0-or-later
    ///
    /// %ARCH%
    /// any
    ///
    /// %BUILDDATE%
    /// 1729181726
    ///
    /// %PACKAGER%
    /// Foobar McFooface <foobar@mcfooface.org>
    ///
    /// "#;
    ///
    /// let repo_desc_v1 = RepoDescFile::from_str_with_schema(
    ///     v1_data,
    ///     Some(RepoDescSchema::V1(SchemaVersion::new(Version::new(
    ///         1, 0, 0,
    ///     )))),
    /// )?;
    /// assert_eq!(repo_desc_v1.to_string(), v1_data);
    ///
    /// let v2_data = r#"%FILENAME%
    /// example-meta-1.0.0-1-any.pkg.tar.zst
    ///
    /// %NAME%
    /// example-meta
    ///
    /// %BASE%
    /// example-meta
    ///
    /// %VERSION%
    /// 1.0.0-1
    ///
    /// %DESC%
    /// An example meta package
    ///
    /// %CSIZE%
    /// 4634
    ///
    /// %ISIZE%
    /// 0
    ///
    /// %SHA256SUM%
    /// b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
    ///
    /// %URL%
    /// https://example.org/
    ///
    /// %LICENSE%
    /// GPL-3.0-or-later
    ///
    /// %ARCH%
    /// any
    ///
    /// %BUILDDATE%
    /// 1729181726
    ///
    /// %PACKAGER%
    /// Foobar McFooface <foobar@mcfooface.org>
    ///
    /// "#;
    ///
    /// let repo_desc_v2 = RepoDescFile::from_str_with_schema(
    ///     v2_data,
    ///     Some(RepoDescSchema::V2(SchemaVersion::new(Version::new(
    ///         2, 0, 0,
    ///     )))),
    /// )?;
    /// assert_eq!(repo_desc_v2.to_string(), v2_data);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the input cannot be parsed into a valid [`RepoDescFile`],
    /// - or the derived or provided schema does not match the detected format.
    fn from_str_with_schema(s: &str, schema: Option<RepoDescSchema>) -> Result<Self, Error> {
        let schema = match schema {
            Some(schema) => schema,
            None => RepoDescSchema::derive_from_str(s)?,
        };

        match schema {
            RepoDescSchema::V1(_) => Ok(RepoDescFile::V1(RepoDescFileV1::from_str(s)?)),
            RepoDescSchema::V2(_) => Ok(RepoDescFile::V2(RepoDescFileV2::from_str(s)?)),
        }
    }
}

impl Display for RepoDescFile {
    /// Returns the textual representation of the [`RepoDescFile`] in its corresponding
    /// [alpm-repo-desc] format.
    ///
    /// [alpm-repo-desc]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V1(file) => write!(f, "{file}"),
            Self::V2(file) => write!(f, "{file}"),
        }
    }
}

impl FromStr for RepoDescFile {
    type Err = Error;

    /// Creates a [`RepoDescFile`] from a string slice.
    ///
    /// Internally calls [`RepoDescFile::from_str_with_schema`] with `schema` set to [`None`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`RepoDescFile::from_str_with_schema`] fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_with_schema(s, None)
    }
}

impl Named for RepoDescFile {
    fn get_name(&self) -> &Name {
        match self {
            Self::V1(file) => file.get_name(),
            Self::V2(file) => file.get_name(),
        }
    }
}

impl Versioned for RepoDescFile {
    fn get_version(&self) -> &FullVersion {
        match self {
            Self::V1(file) => file.get_version(),
            Self::V2(file) => file.get_version(),
        }
    }
}

impl RuntimeRelations for RepoDescFile {
    fn get_run_time_dependencies(&self) -> &[RelationOrSoname] {
        match self {
            Self::V1(file) => file.get_run_time_dependencies(),
            Self::V2(file) => file.get_run_time_dependencies(),
        }
    }

    fn get_optional_dependencies(&self) -> &[OptionalDependency] {
        match self {
            Self::V1(file) => file.get_optional_dependencies(),
            Self::V2(file) => file.get_optional_dependencies(),
        }
    }

    fn get_provisions(&self) -> &[RelationOrSoname] {
        match self {
            Self::V1(file) => file.get_provisions(),
            Self::V2(file) => file.get_provisions(),
        }
    }

    fn get_conflicts(&self) -> &[PackageRelation] {
        match self {
            Self::V1(file) => file.get_conflicts(),
            Self::V2(file) => file.get_conflicts(),
        }
    }

    fn get_replacements(&self) -> &[PackageRelation] {
        match self {
            Self::V1(file) => file.get_replacements(),
            Self::V2(file) => file.get_replacements(),
        }
    }
}
