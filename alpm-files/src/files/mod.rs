//! The representation of [alpm-files] files.
//!
//! [alpm-files]: https://alpm.archlinux.page/specifications/alpm-files.5.html

pub mod v1;

use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_common::{FileFormatSchema, MetadataFile};
use fluent_i18n::t;

use crate::{Error, FilesSchema, FilesV1};

/// The different styles of the [alpm-files] format.
///
/// [alpm-files]: https://alpm.archlinux.page/specifications/alpm-files.5.html
#[derive(Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum FilesStyle {
    /// The [alpm-db-files] style of the format.
    ///
    /// This style
    ///
    /// - always produces an empty file, if no paths are tracked,
    /// - and always has a trailing empty line, if paths are tracked.
    ///
    /// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
    #[cfg_attr(feature = "cli", value(help = t!("cli-style-db-help")))]
    Db,

    /// The [alpm-repo-files] style of the format.
    ///
    /// This style
    ///
    /// - always produces the section header, if no paths are tracked,
    /// - and never has a trailing empty line.
    ///
    /// [alpm-repo-files]: https://alpm.archlinux.page/specifications/alpm-repo-files.5.html
    #[cfg_attr(feature = "cli", value(help = t!("cli-style-repo-help")))]
    Repo,
}

/// An interface to guarantee the creation of a [`String`] based on a [`FilesStyle`].
pub trait FilesStyleToString {
    /// Returns the [`String`] representation of the implementation based on a [`FilesStyle`].
    fn to_string(&self, style: FilesStyle) -> String;
}

/// The representation of [alpm-files] data.
///
/// Tracks all known versions of the specification.
///
/// [alpm-files]: https://alpm.archlinux.page/specifications/alpm-files.5.html
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Files {
    /// Version 1 of the [alpm-files] specification.
    ///
    /// [alpm-files]: https://alpm.archlinux.page/specifications/alpm-files.5.html
    V1(FilesV1),
}

impl AsRef<[PathBuf]> for Files {
    /// Returns a reference to the inner [`Vec`] of [`PathBuf`]s.
    fn as_ref(&self) -> &[PathBuf] {
        match self {
            Files::V1(files) => files.as_ref(),
        }
    }
}

impl FilesStyleToString for Files {
    /// Returns the [`String`] representation of the [`Files`].
    ///
    /// The formatting of the returned string depends on the provided [`FilesStyle`].
    fn to_string(&self, format: FilesStyle) -> String {
        match self {
            Files::V1(files) => files.to_string(format),
        }
    }
}

impl MetadataFile<FilesSchema> for Files {
    type Err = Error;

    /// Creates a new [`Files`] from a file [`Path`] and an optional [`FilesSchema`].
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
    /// use alpm_files::{Files, FilesSchema};
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
    /// let files = Files::from_file_with_schema(
    ///     temp_file.path(),
    ///     Some(FilesSchema::V1(SchemaVersion::new(Version::new(1, 0, 0)))),
    /// )?;
    /// matches!(files, Files::V1(_));
    /// assert_eq!(files.as_ref().len(), 3);
    /// # Ok(())
    /// # }
    /// ```
    fn from_file_with_schema(
        file: impl AsRef<Path>,
        schema: Option<FilesSchema>,
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

    /// Creates a new [`Files`] from a [`Read`] implementation and an optional [`FilesSchema`].
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
    /// use alpm_files::{Files, FilesSchema};
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
    /// let files = Files::from_reader_with_schema(
    ///     temp_file,
    ///     Some(FilesSchema::V1(SchemaVersion::new(Version::new(1, 0, 0)))),
    /// )?;
    /// matches!(files, Files::V1(_));
    /// assert_eq!(files.as_ref().len(), 3);
    /// # Ok(())
    /// # }
    /// ```
    fn from_reader_with_schema(
        mut reader: impl Read,
        schema: Option<FilesSchema>,
    ) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .map_err(|source| Error::Io {
                context: t!("error-io-context-reading-alpm-files-data"),
                source,
            })?;
        Self::from_str_with_schema(&buf, schema)
    }

    /// Creates a new [`Files`] from a string slice and an optional [`FilesSchema`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - `schema` is [`None`] and a [`FilesSchema`] cannot be derived from `s`,
    /// - or a [`FilesV1`] cannot be created from `s`.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_common::MetadataFile;
    /// use alpm_files::{Files, FilesSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> Result<(), alpm_files::Error> {
    /// let data = r#"%FILES%
    /// usr/
    /// usr/bin/
    /// usr/bin/foo
    /// "#;
    /// let files = Files::from_str_with_schema(
    ///     data,
    ///     Some(FilesSchema::V1(SchemaVersion::new(Version::new(1, 0, 0)))),
    /// )?;
    /// matches!(files, Files::V1(_));
    /// assert_eq!(files.as_ref().len(), 3);
    /// # Ok(())
    /// # }
    /// ```
    fn from_str_with_schema(s: &str, schema: Option<FilesSchema>) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        let schema = match schema {
            Some(schema) => schema,
            None => FilesSchema::derive_from_str(s)?,
        };

        match schema {
            FilesSchema::V1(_) => Ok(Files::V1(FilesV1::from_str(s)?)),
        }
    }
}

impl FromStr for Files {
    type Err = Error;

    /// Creates a new [`Files`] from string slice.
    ///
    /// # Note
    ///
    /// Delegates to [`Self::from_str_with_schema`] while not providing a [`FilesSchema`].
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

    /// Ensures that the [`FilesStyleToString`] implementation for [`Files`] works as intended.
    #[rstest]
    #[case(
        vec![
            PathBuf::from("usr/"),
            PathBuf::from("usr/bin/"),
            PathBuf::from("usr/bin/foo"),
        ],
        FilesStyle::Db,
        r#"%FILES%
usr/
usr/bin/
usr/bin/foo

"#
    )]
    #[case(Vec::new(), FilesStyle::Db, "")]
    #[case(
        vec![
            PathBuf::from("usr/"),
            PathBuf::from("usr/bin/"),
            PathBuf::from("usr/bin/foo"),
        ],
        FilesStyle::Repo,
        r#"%FILES%
usr/
usr/bin/
usr/bin/foo
"#
    )]
    #[case(Vec::new(), FilesStyle::Repo, "%FILES%\n")]
    fn files_to_string(
        #[case] input: Vec<PathBuf>,
        #[case] format: FilesStyle,
        #[case] expected_output: &str,
    ) -> TestResult {
        let files = Files::V1(FilesV1::try_from(input)?);

        assert_eq!(files.to_string(format), expected_output);

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
        let files = Files::from_str(input)?;

        assert_eq!(files.as_ref(), expected_paths);

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

    /// Ensures that different types of full and empty alpm-files files can be parsed from file.
    #[rstest]
    #[case::alpm_db_files_full(ALPM_DB_FILES_FULL, 3)]
    #[case::alpm_db_files_empty(ALPM_DB_FILES_EMPTY, 0)]
    #[case::alpm_repo_files_full(ALPM_REPO_FILES_FULL, 3)]
    #[case::alpm_repo_files_full(ALPM_REPO_FILES_EMPTY, 0)]
    fn files_from_file_with_schema_succeeds(#[case] data: &str, #[case] len: usize) -> TestResult {
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{data}")?;

        let files = Files::from_file_with_schema(
            temp_file.path(),
            Some(FilesSchema::V1(SchemaVersion::new(Version::new(1, 0, 0)))),
        )?;

        assert!(matches!(files, Files::V1(_)));
        assert_eq!(files.as_ref().len(), len);

        Ok(())
    }
}
