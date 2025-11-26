//! The representation of [alpm-repo-files] files.
//!
//! [alpm-repo-files]: https://alpm.archlinux.page/specifications/alpm-repo-files.5.html

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
pub use schema::RepoFilesSchema;
pub use v1::RepoFilesV1;

/// The representation of [alpm-repo-files] data.
///
/// Tracks all known versions of the specification.
///
/// [alpm-repo-files]: https://alpm.archlinux.page/specifications/alpm-repo-files.5.html
#[derive(Clone, Debug, serde::Serialize)]
#[serde(untagged)]
pub enum RepoFiles {
    /// Version 1 of the [alpm-repo-files] specification.
    ///
    /// [alpm-repo-files]: https://alpm.archlinux.page/specifications/alpm-repo-files.5.html
    V1(RepoFilesV1),
}

impl Display for RepoFiles {
    /// Formats the [`RepoFiles`] as a string.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepoFiles::V1(files) => files.fmt(f),
        }
    }
}

impl AsRef<[PathBuf]> for RepoFiles {
    /// Returns a reference to the inner [`Vec`] of [`PathBuf`]s.
    fn as_ref(&self) -> &[PathBuf] {
        match self {
            RepoFiles::V1(files) => files.as_ref(),
        }
    }
}

impl MetadataFile<RepoFilesSchema> for RepoFiles {
    type Err = Error;

    /// Creates a new [`RepoFiles`] from a file [`Path`] and an optional [`RepoFilesSchema`].
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
    /// use alpm_repo_db::files::{RepoFiles, RepoFilesSchema};
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
    /// let files = RepoFiles::from_file_with_schema(
    ///     temp_file.path(),
    ///     Some(RepoFilesSchema::V1(SchemaVersion::new(Version::new(
    ///         1, 0, 0,
    ///     )))),
    /// )?;
    /// matches!(files, RepoFiles::V1(_));
    /// assert_eq!(files.as_ref().len(), 3);
    /// # Ok(())
    /// # }
    /// ```
    fn from_file_with_schema(
        file: impl AsRef<Path>,
        schema: Option<RepoFilesSchema>,
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

    /// Creates a new [`RepoFiles`] from a [`Read`] implementation and an optional
    /// [`RepoFilesSchema`].
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
    /// use alpm_repo_db::files::{RepoFiles, RepoFilesSchema};
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
    /// let files = RepoFiles::from_reader_with_schema(
    ///     temp_file,
    ///     Some(RepoFilesSchema::V1(SchemaVersion::new(Version::new(
    ///         1, 0, 0,
    ///     )))),
    /// )?;
    /// matches!(files, RepoFiles::V1(_));
    /// assert_eq!(files.as_ref().len(), 3);
    /// # Ok(())
    /// # }
    /// ```
    fn from_reader_with_schema(
        mut reader: impl Read,
        schema: Option<RepoFilesSchema>,
    ) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .map_err(|source| Error::Io {
                context: t!("error-io-context-reading-alpm-repo-files-data"),
                source,
            })?;
        Self::from_str_with_schema(&buf, schema)
    }

    /// Creates a new [`RepoFiles`] from a string slice and an optional [`RepoFilesSchema`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - `schema` is [`None`] and a [`RepoFilesSchema`] cannot be derived from `s`,
    /// - or a [`RepoFilesV1`] cannot be created from `s`.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_common::MetadataFile;
    /// use alpm_repo_db::files::{RepoFiles, RepoFilesSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> Result<(), alpm_repo_db::files::Error> {
    /// let data = r#"%FILES%
    /// usr/
    /// usr/bin/
    /// usr/bin/foo
    /// "#;
    /// let files = RepoFiles::from_str_with_schema(
    ///     data,
    ///     Some(RepoFilesSchema::V1(SchemaVersion::new(Version::new(
    ///         1, 0, 0,
    ///     )))),
    /// )?;
    /// matches!(files, RepoFiles::V1(_));
    /// assert_eq!(files.as_ref().len(), 3);
    /// # Ok(())
    /// # }
    /// ```
    fn from_str_with_schema(s: &str, schema: Option<RepoFilesSchema>) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        let schema = match schema {
            Some(schema) => schema,
            None => RepoFilesSchema::derive_from_str(s)?,
        };

        match schema {
            RepoFilesSchema::V1(_) => Ok(RepoFiles::V1(RepoFilesV1::from_str(s)?)),
        }
    }
}

impl FromStr for RepoFiles {
    type Err = Error;

    /// Creates a new [`RepoFiles`] from string slice.
    ///
    /// # Note
    ///
    /// Delegates to [`Self::from_str_with_schema`] while not providing a [`RepoFilesSchema`].
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
    use std::io::Write;

    use alpm_types::{SchemaVersion, semver_version::Version};
    use rstest::rstest;
    use tempfile::NamedTempFile;
    use testresult::TestResult;

    use super::*;

    /// Ensures that [`RepoFiles::to_string`] produces the expected output.
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
    #[case(Vec::new(), "%FILES%\n")]
    fn files_to_string(#[case] input: Vec<PathBuf>, #[case] expected_output: &str) -> TestResult {
        let files = RepoFiles::V1(RepoFilesV1::try_from(input)?);

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
        let files = RepoFiles::from_str(input)?;

        assert_eq!(files.as_ref(), expected_paths);

        Ok(())
    }

    /// Ensures that missing section headers are rejected when deriving the schema.
    #[test]
    fn files_from_str_fails_without_header() {
        let result = RepoFiles::from_str("");

        assert!(matches!(result, Err(Error::UnknownSchemaVersion)));
    }

    const ALPM_REPO_FILES_FULL: &str = r#"%FILES%
usr/
usr/bin/
usr/bin/foo
"#;
    const ALPM_REPO_FILES_EMPTY: &str = "%FILES%\n";
    const ALPM_REPO_FILES_EMPTY_NO_HEADER: &str = "";

    /// Ensures that full and empty alpm-repo-files files can be parsed from file.
    #[rstest]
    #[case::alpm_repo_files_full(ALPM_REPO_FILES_FULL, 3)]
    #[case::alpm_repo_files_empty(ALPM_REPO_FILES_EMPTY, 0)]
    fn files_from_file_with_schema_succeeds(#[case] data: &str, #[case] len: usize) -> TestResult {
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{data}")?;

        let files = RepoFiles::from_file_with_schema(
            temp_file.path(),
            Some(RepoFilesSchema::V1(SchemaVersion::new(Version::new(
                1, 0, 0,
            )))),
        )?;

        assert!(matches!(files, RepoFiles::V1(_)));
        assert_eq!(files.as_ref().len(), len);

        Ok(())
    }

    /// Ensures that missing headers prevent parsing alpm-repo-files files.
    #[test]
    fn files_from_file_with_schema_fails_without_header() -> TestResult {
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{ALPM_REPO_FILES_EMPTY_NO_HEADER}")?;

        let result = RepoFiles::from_file_with_schema(
            temp_file.path(),
            Some(RepoFilesSchema::V1(SchemaVersion::new(Version::new(
                1, 0, 0,
            )))),
        );

        assert!(matches!(result, Err(Error::ParseError(_))));

        Ok(())
    }
}
