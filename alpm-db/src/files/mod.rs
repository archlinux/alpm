//! The representation of [alpm-db-files] files.
//!
//! [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;

mod error;
mod schema;
pub mod v1;

use std::{
    fmt::Display,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_common::{FileFormatSchema, MetadataFile};
pub use error::Error;
use fluent_i18n::t;
pub use schema::DbFilesSchema;
pub use v1::{BackupEntry, DbFilesV1};

/// The representation of [alpm-db-files] data.
///
/// Tracks all known versions of the specification.
///
/// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
#[derive(Clone, Debug, serde::Serialize)]
#[serde(untagged)]
pub enum DbFiles {
    /// Version 1 of the [alpm-db-files] specification.
    ///
    /// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
    V1(DbFilesV1),
}

impl Display for DbFiles {
    /// Formats the [`DbFiles`] as a string according to its style.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbFiles::V1(files) => files.fmt(f),
        }
    }
}

impl AsRef<[PathBuf]> for DbFiles {
    /// Returns a reference to the inner [`Vec`] of [`PathBuf`]s.
    fn as_ref(&self) -> &[PathBuf] {
        match self {
            DbFiles::V1(files) => files.as_ref(),
        }
    }
}

impl DbFiles {
    /// Returns the backup entries associated with this [`DbFiles`].
    pub fn backups(&self) -> &[BackupEntry] {
        match self {
            DbFiles::V1(files) => files.backups(),
        }
    }
}

impl MetadataFile<DbFilesSchema> for DbFiles {
    type Err = Error;

    /// Creates a new [`DbFiles`] from a file [`Path`] and an optional [`DbFilesSchema`].
    ///
    /// # Note
    ///
    /// Delegates to [`Self::from_reader_with_schema`] after opening `file` for reading.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - the `file` cannot be opened for reading,
    /// - or [`Self::from_reader_with_schema`] fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Write;
    ///
    /// use alpm_common::MetadataFile;
    /// use alpm_db::files::{DbFiles, DbFilesSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    /// use tempfile::NamedTempFile;
    ///
    /// # fn main() -> testresult::TestResult {
    /// let data = r#"%FILES%
    /// usr/
    /// usr/bin/
    /// usr/bin/foo
    /// "#;
    /// let mut temp_file = NamedTempFile::new()?;
    /// write!(temp_file, "{data}")?;
    /// let files = DbFiles::from_file_with_schema(
    ///     temp_file.path(),
    ///     Some(DbFilesSchema::V1(SchemaVersion::new(Version::new(1, 0, 0)))),
    /// )?;
    /// matches!(files, DbFiles::V1(_));
    /// assert_eq!(files.as_ref().len(), 3);
    /// # Ok(())
    /// # }
    /// ```
    fn from_file_with_schema(
        file: impl AsRef<Path>,
        schema: Option<DbFilesSchema>,
    ) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        let path = file.as_ref();
        Self::from_reader_with_schema(
            File::open(path).map_err(|source| Error::IoPath {
                path: path.to_path_buf(),
                context: t!("error-io-path-context-opening-the-file-for-reading"),
                source,
            })?,
            schema,
        )
    }

    /// Creates a new [`DbFiles`] from a [`Read`] implementation and an optional [`DbFilesSchema`].
    ///
    /// # Note
    ///
    /// Delegates to [`Self::from_str_with_schema`] after reading `reader` to string.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - the `reader` cannot be read to string,
    /// - or [`Self::from_str_with_schema`] fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Seek, SeekFrom, Write};
    ///
    /// use alpm_common::MetadataFile;
    /// use alpm_db::files::{DbFiles, DbFilesSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    /// use tempfile::tempfile;
    ///
    /// # fn main() -> testresult::TestResult {
    /// let data = r#"%FILES%
    /// usr/
    /// usr/bin/
    /// usr/bin/foo
    /// "#;
    /// let mut temp_file = tempfile()?;
    /// write!(temp_file, "{data}")?;
    /// temp_file.seek(SeekFrom::Start(0))?;
    /// let files = DbFiles::from_reader_with_schema(
    ///     temp_file,
    ///     Some(DbFilesSchema::V1(SchemaVersion::new(Version::new(1, 0, 0)))),
    /// )?;
    /// matches!(files, DbFiles::V1(_));
    /// assert_eq!(files.as_ref().len(), 3);
    /// # Ok(())
    /// # }
    /// ```
    fn from_reader_with_schema(
        mut reader: impl Read,
        schema: Option<DbFilesSchema>,
    ) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .map_err(|source| Error::Io {
                context: t!("error-io-context-reading-alpm-db-files-data"),
                source,
            })?;
        Self::from_str_with_schema(&buf, schema)
    }

    /// Creates a new [`DbFiles`] from a string slice and an optional [`DbFilesSchema`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - `schema` is [`None`] and a [`DbFilesSchema`] cannot be derived from `s`,
    /// - or a [`DbFilesV1`] cannot be created from `s`.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_common::MetadataFile;
    /// use alpm_db::files::{DbFiles, DbFilesSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> Result<(), alpm_db::files::Error> {
    /// let data = r#"%FILES%
    /// usr/
    /// usr/bin/
    /// usr/bin/foo
    /// "#;
    /// let files = DbFiles::from_str_with_schema(
    ///     data,
    ///     Some(DbFilesSchema::V1(SchemaVersion::new(Version::new(1, 0, 0)))),
    /// )?;
    /// matches!(files, DbFiles::V1(_));
    /// assert_eq!(files.as_ref().len(), 3);
    /// # Ok(())
    /// # }
    /// ```
    fn from_str_with_schema(s: &str, schema: Option<DbFilesSchema>) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        let schema = match schema {
            Some(schema) => schema,
            None => DbFilesSchema::derive_from_str(s)?,
        };

        match schema {
            DbFilesSchema::V1(_) => Ok(DbFiles::V1(DbFilesV1::from_str(s)?)),
        }
    }
}

impl FromStr for DbFiles {
    type Err = Error;

    /// Creates a new [`DbFiles`] from string slice.
    ///
    /// # Note
    ///
    /// Delegates to [`Self::from_str_with_schema`] while not providing a [`DbFilesSchema`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`Self::from_str_with_schema`] fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_with_schema(s, None)
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Write, str::FromStr};

    use alpm_types::{Md5Checksum, RelativeFilePath, SchemaVersion, semver_version::Version};
    use rstest::rstest;
    use tempfile::NamedTempFile;
    use testresult::TestResult;

    use super::*;

    /// Ensures that [`DbFiles::to_string`] produces the expected output.
    #[rstest]
    #[case(
        vec![
            PathBuf::from("usr/"),
            PathBuf::from("usr/bin/"),
            PathBuf::from("usr/bin/foo"),
        ],
        r#"%FILES%
usr/
usr/bin/
usr/bin/foo

"#
    )]
    #[case(Vec::new(), "")]
    fn files_to_string(#[case] input: Vec<PathBuf>, #[case] expected_output: &str) -> TestResult {
        let files = DbFiles::V1(DbFilesV1::try_from(input)?);

        assert_eq!(files.to_string(), expected_output);

        Ok(())
    }

    #[test]
    fn files_to_string_with_backup() -> TestResult {
        let files = DbFiles::V1(DbFilesV1::try_from((
            vec![
                PathBuf::from("usr/"),
                PathBuf::from("usr/bin/"),
                PathBuf::from("usr/bin/foo"),
            ],
            vec![BackupEntry {
                path: RelativeFilePath::from_str("usr/bin/foo")?,
                md5: Md5Checksum::from_str("d41d8cd98f00b204e9800998ecf8427e")?,
            }],
        ))?);

        let expected_output = r#"%FILES%
usr/
usr/bin/
usr/bin/foo

%BACKUP%
usr/bin/foo	d41d8cd98f00b204e9800998ecf8427e
"#;

        assert_eq!(files.to_string(), expected_output);

        Ok(())
    }

    #[test]
    fn files_from_str() -> TestResult {
        let input = r#"%FILES%
usr/
usr/bin/
usr/bin/foo

"#;
        let expected_paths = vec![
            PathBuf::from("usr/"),
            PathBuf::from("usr/bin/"),
            PathBuf::from("usr/bin/foo"),
        ];
        let files = DbFiles::from_str(input)?;

        assert_eq!(files.as_ref(), expected_paths);

        Ok(())
    }

    #[test]
    fn files_from_str_with_backup() -> TestResult {
        let input = r#"%FILES%
usr/
usr/bin/
usr/bin/foo

%BACKUP%
usr/bin/foo	d41d8cd98f00b204e9800998ecf8427e
"#;
        let files = DbFiles::from_str(input)?;

        let expected_backup = BackupEntry {
            path: RelativeFilePath::from_str("usr/bin/foo")?,
            md5: Md5Checksum::from_str("d41d8cd98f00b204e9800998ecf8427e")?,
        };

        assert_eq!(
            files.as_ref(),
            &[
                PathBuf::from("usr/"),
                PathBuf::from("usr/bin/"),
                PathBuf::from("usr/bin/foo")
            ]
        );

        assert_eq!(files.backups(), &[expected_backup]);

        Ok(())
    }

    const ALPM_DB_FILES_FULL: &str = r#"%FILES%
usr/
usr/bin/
usr/bin/foo

"#;
    const ALPM_DB_FILES_EMPTY: &str = "";
    const ALPM_REPO_FILES_FULL: &str = r#"%FILES%
usr/
usr/bin/
usr/bin/foo
"#;
    const ALPM_REPO_FILES_EMPTY: &str = "%FILES%";

    /// Ensures that different types of full and empty alpm-db-files files can be parsed from file.
    #[rstest]
    #[case::alpm_db_files_full(ALPM_DB_FILES_FULL, 3)]
    #[case::alpm_db_files_empty(ALPM_DB_FILES_EMPTY, 0)]
    #[case::alpm_repo_files_full(ALPM_REPO_FILES_FULL, 3)]
    #[case::alpm_repo_files_full(ALPM_REPO_FILES_EMPTY, 0)]
    fn files_from_file_with_schema_succeeds(#[case] data: &str, #[case] len: usize) -> TestResult {
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{data}")?;

        let files = DbFiles::from_file_with_schema(
            temp_file.path(),
            Some(DbFilesSchema::V1(SchemaVersion::new(Version::new(1, 0, 0)))),
        )?;

        assert!(matches!(files, DbFiles::V1(_)));
        assert_eq!(files.as_ref().len(), len);

        Ok(())
    }

    #[test]
    fn files_from_file_with_backup_section() -> TestResult {
        let data = r#"%FILES%
usr/
usr/bin/
usr/bin/foo

%BACKUP%
usr/bin/foo	d41d8cd98f00b204e9800998ecf8427e
"#;
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{data}")?;

        let files = DbFiles::from_file_with_schema(
            temp_file.path(),
            Some(DbFilesSchema::V1(SchemaVersion::new(Version::new(1, 0, 0)))),
        )?;

        assert!(matches!(files, DbFiles::V1(_)));
        assert_eq!(files.backups().len(), 1);

        Ok(())
    }
}
