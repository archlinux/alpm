use std::{
    fmt::Display,
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_common::{FileFormatSchema, MetadataFile};
use fluent_i18n::t;

use crate::{
    Error,
    desc::{DbDescFileV1, DbDescFileV2, DbDescSchema},
};

/// A representation of the [alpm-db-desc] file format.
///
/// Tracks all supported schema versions (`v1` and `v2`) of the database description file.
/// Each variant corresponds to a distinct layout of the format.
///
/// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(untagged)]
pub enum DbDescFile {
    /// The [alpm-db-descv1] file format.
    ///
    /// [alpm-db-descv1]: https://alpm.archlinux.page/specifications/alpm-db-descv1.5.html
    V1(DbDescFileV1),
    /// The [alpm-db-descv2] file format.
    ///
    /// This revision of the file format, adds the `%XDATA%` section.
    ///
    /// [alpm-db-descv2]: https://alpm.archlinux.page/specifications/alpm-db-descv2.5.html
    V2(DbDescFileV2),
}

impl MetadataFile<DbDescSchema> for DbDescFile {
    type Err = Error;

    /// Creates a [`DbDescFile`] from a file on disk, optionally validated using a [`DbDescSchema`].
    ///
    /// Opens the file and defers to [`DbDescFile::from_reader_with_schema`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_common::{FileFormatSchema, MetadataFile};
    /// use alpm_db::desc::{DbDescFile, DbDescSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// // Prepare a file with DB desc data (v1)
    /// let (file, desc_data) = {
    ///     let desc_data = r#"%NAME%
    /// foo
    ///
    /// %VERSION%
    /// 1.0.0-1
    ///
    /// %BASE%
    /// foo
    ///
    /// %DESC%
    /// An example package
    ///
    /// %URL%
    /// https://example.org/
    ///
    /// %ARCH%
    /// x86_64
    ///
    /// %BUILDDATE%
    /// 1733737242
    ///
    /// %INSTALLDATE%
    /// 1733737243
    ///
    /// %PACKAGER%
    /// Foobar McFooface <foobar@mcfooface.org>
    ///
    /// %SIZE%
    /// 123
    ///
    /// %VALIDATION%
    /// pgp
    ///
    /// "#;
    ///     let file = tempfile::NamedTempFile::new()?;
    ///     let mut output = File::create(&file)?;
    ///     write!(output, "{}", desc_data)?;
    ///     (file, desc_data)
    /// };
    ///
    /// let db_desc = DbDescFile::from_file_with_schema(
    ///     file.path(),
    ///     Some(DbDescSchema::V1(SchemaVersion::new(Version::new(1, 0, 0)))),
    /// )?;
    /// assert_eq!(db_desc.to_string(), desc_data);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the file cannot be opened for reading,
    /// - the contents cannot be parsed into any known [`DbDescFile`] variant,
    /// - or the provided [`DbDescSchema`] does not match the contents of the file.
    fn from_file_with_schema(
        file: impl AsRef<Path>,
        schema: Option<DbDescSchema>,
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

    /// Creates a [`DbDescFile`] from any readable stream, optionally validated using a
    /// [`DbDescSchema`].
    ///
    /// Reads the `reader` to a string buffer and defers to [`DbDescFile::from_str_with_schema`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_common::MetadataFile;
    /// use alpm_db::desc::{DbDescFile, DbDescSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// // Prepare a reader with DB desc data (v2)
    /// let (reader, desc_data) = {
    ///     let desc_data = r#"%NAME%
    /// foo
    ///
    /// %VERSION%
    /// 1.0.0-1
    ///
    /// %BASE%
    /// foo
    ///
    /// %DESC%
    /// An example package
    ///
    /// %URL%
    /// https://example.org/
    ///
    /// %ARCH%
    /// x86_64
    ///
    /// %BUILDDATE%
    /// 1733737242
    ///
    /// %INSTALLDATE%
    /// 1733737243
    ///
    /// %PACKAGER%
    /// Foobar McFooface <foobar@mcfooface.org>
    ///
    /// %SIZE%
    /// 123
    ///
    /// %VALIDATION%
    /// pgp
    ///
    /// %XDATA%
    /// pkgtype=pkg
    ///
    /// "#;
    ///     let file = tempfile::NamedTempFile::new()?;
    ///     let mut output = File::create(&file)?;
    ///     write!(output, "{}", desc_data)?;
    ///     (File::open(&file.path())?, desc_data)
    /// };
    ///
    /// let db_desc = DbDescFile::from_reader_with_schema(
    ///     reader,
    ///     Some(DbDescSchema::V2(SchemaVersion::new(Version::new(2, 0, 0)))),
    /// )?;
    /// assert_eq!(db_desc.to_string(), desc_data);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the `reader` cannot be read to string,
    /// - the data cannot be parsed into a known [`DbDescFile`] variant,
    /// - or the provided [`DbDescSchema`] does not match the parsed content.
    fn from_reader_with_schema(
        mut reader: impl std::io::Read,
        schema: Option<DbDescSchema>,
    ) -> Result<Self, Error> {
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .map_err(|source| Error::IoReadError {
                context: t!("error-io-read-db-desc"),
                source,
            })?;
        Self::from_str_with_schema(&buf, schema)
    }

    /// Creates a [`DbDescFile`] from a string slice, optionally validated using a [`DbDescSchema`].
    ///
    /// If `schema` is [`None`], automatically infers the schema version by inspecting the input
    /// (`v1` = no `%XDATA%` section, `v2` = has `%XDATA%`).
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_common::MetadataFile;
    /// use alpm_db::desc::{DbDescFile, DbDescSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// let v1_data = r#"%NAME%
    /// foo
    ///
    /// %VERSION%
    /// 1.0.0-1
    ///
    /// %BASE%
    /// foo
    ///
    /// %DESC%
    /// An example package
    ///
    /// %URL%
    /// https://example.org/
    ///
    /// %ARCH%
    /// x86_64
    ///
    /// %BUILDDATE%
    /// 1733737242
    ///
    /// %INSTALLDATE%
    /// 1733737243
    ///
    /// %PACKAGER%
    /// Foobar McFooface <foobar@mcfooface.org>
    ///
    /// %SIZE%
    /// 123
    ///
    /// %VALIDATION%
    /// pgp
    ///
    /// "#;
    ///
    /// let dbdesc_v1 = DbDescFile::from_str_with_schema(
    ///     v1_data,
    ///     Some(DbDescSchema::V1(SchemaVersion::new(Version::new(1, 0, 0)))),
    /// )?;
    /// assert_eq!(dbdesc_v1.to_string(), v1_data);
    ///
    /// let v2_data = r#"%NAME%
    /// foo
    ///
    /// %VERSION%
    /// 1.0.0-1
    ///
    /// %BASE%
    /// foo
    ///
    /// %DESC%
    /// An example package
    ///
    /// %URL%
    /// https://example.org/
    ///
    /// %ARCH%
    /// x86_64
    ///
    /// %BUILDDATE%
    /// 1733737242
    ///
    /// %INSTALLDATE%
    /// 1733737243
    ///
    /// %PACKAGER%
    /// Foobar McFooface <foobar@mcfooface.org>
    ///
    /// %SIZE%
    /// 123
    ///
    /// %VALIDATION%
    /// pgp
    ///
    /// %XDATA%
    /// pkgtype=pkg
    ///
    /// "#;
    ///
    /// let dbdesc_v2 = DbDescFile::from_str_with_schema(
    ///     v2_data,
    ///     Some(DbDescSchema::V2(SchemaVersion::new(Version::new(2, 0, 0)))),
    /// )?;
    /// assert_eq!(dbdesc_v2.to_string(), v2_data);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the input cannot be parsed into a valid [`DbDescFile`],
    /// - or the derived or provided schema does not match the detected format.
    fn from_str_with_schema(s: &str, schema: Option<DbDescSchema>) -> Result<Self, Error> {
        let schema = match schema {
            Some(schema) => schema,
            None => DbDescSchema::derive_from_str(s)?,
        };

        match schema {
            DbDescSchema::V1(_) => Ok(DbDescFile::V1(DbDescFileV1::from_str(s)?)),
            DbDescSchema::V2(_) => Ok(DbDescFile::V2(DbDescFileV2::from_str(s)?)),
        }
    }
}

impl Display for DbDescFile {
    /// Returns the textual representation of the [`DbDescFile`] in its corresponding
    /// [alpm-db-desc] format.
    ///
    /// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V1(file) => write!(f, "{file}"),
            Self::V2(file) => write!(f, "{file}"),
        }
    }
}

impl FromStr for DbDescFile {
    type Err = Error;

    /// Creates a [`DbDescFile`] from a string slice.
    ///
    /// Internally calls [`DbDescFile::from_str_with_schema`] with `schema` set to [`None`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`DbDescFile::from_str_with_schema`] fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_with_schema(s, None)
    }
}
