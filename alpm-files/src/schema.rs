//! Schema definition for the [alpm-files] format.
//!
//! [alpm-files]: https://alpm.archlinux.page/specifications/alpm-files.5.html

use std::{fs::File, io::Read, path::Path};

use alpm_common::FileFormatSchema;
use alpm_types::{SchemaVersion, semver_version::Version};
use fluent_i18n::t;

use crate::{Error, files::v1::FilesSection};

/// A schema for the [alpm-files] format.
///
/// [alpm-files]: https://alpm.archlinux.page/specifications/alpm-files.5.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FilesSchema {
    /// Version 1 of the [alpm-files] specification.
    ///
    /// [alpm-files]: https://alpm.archlinux.page/specifications/alpm-files.5.html
    V1(SchemaVersion),
}

impl FileFormatSchema for FilesSchema {
    type Err = Error;

    /// Returns a reference to the inner [`SchemaVersion`].
    fn inner(&self) -> &SchemaVersion {
        match self {
            FilesSchema::V1(v) => v,
        }
    }

    /// Creates a new [`FilesSchema`] from a file [`Path`].
    ///
    /// # Note
    ///
    /// Delegates to [`Self::derive_from_reader`] after opening `file` for reading.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - `file` cannot be opened for reading,
    /// - or [`Self::derive_from_reader`] fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Write;
    ///
    /// use alpm_common::FileFormatSchema;
    /// use alpm_files::FilesSchema;
    /// use tempfile::NamedTempFile;
    ///
    /// # fn main() -> testresult::TestResult {
    /// let data = r#"%FILES%
    /// usr/
    /// usr/bin/
    /// usr/bin/foo
    /// "#;
    /// let mut temp_file = NamedTempFile::new()?;
    /// write!(temp_file, "{data}");
    /// let schema = FilesSchema::derive_from_file(temp_file.path())?;
    /// matches!(schema, FilesSchema::V1(_));
    ///
    /// // Empty inputs may also be considered as a schema
    /// let mut temp_file = NamedTempFile::new()?;
    /// let schema = FilesSchema::derive_from_file(temp_file.path())?;
    /// matches!(schema, FilesSchema::V1(_));
    /// # Ok(())
    /// # }
    /// ```
    fn derive_from_file(file: impl AsRef<Path>) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        let file = file.as_ref();
        Self::derive_from_reader(File::open(file).map_err(|source| Error::IoPath {
            path: file.to_path_buf(),
            context: t!("error-io-path-context-deriving-schema-version-from-alpm-files-file"),
            source,
        })?)
    }

    /// Creates a new [`FilesSchema`] from a [`Read`] implementation.
    ///
    /// # Note
    ///
    /// Delegates to [`Self::derive_from_str`] after reading the `reader` to string.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - `reader` cannot be read to string,
    /// - or [`Self::derive_from_str`] fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Seek, SeekFrom, Write};
    ///
    /// use alpm_common::FileFormatSchema;
    /// use alpm_files::FilesSchema;
    /// use tempfile::tempfile;
    ///
    /// # fn main() -> testresult::TestResult {
    /// let data = r#"%FILES%
    /// usr/
    /// usr/bin/
    /// usr/bin/foo
    /// "#;
    /// let mut temp_file = tempfile()?;
    /// write!(temp_file, "{data}");
    /// temp_file.seek(SeekFrom::Start(0))?;
    ///
    /// let schema = FilesSchema::derive_from_reader(temp_file)?;
    /// matches!(schema, FilesSchema::V1(_));
    ///
    /// // Empty inputs may also be considered as a schema
    /// let mut temp_file = tempfile()?;
    /// let schema = FilesSchema::derive_from_reader(temp_file)?;
    /// matches!(schema, FilesSchema::V1(_));
    /// # Ok(())
    /// # }
    /// ```
    fn derive_from_reader(mut reader: impl Read) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .map_err(|source| Error::Io {
                context: t!("error-io-context-deriving-a-schema-version-from-alpm-files-data"),
                source,
            })?;
        Self::derive_from_str(&buf)
    }

    /// Creates a new [`FilesSchema`] from a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - a [`FilesSchema`] cannot be derived from `s`,
    /// - or a [`FilesV1`][`crate::FilesV1`] cannot be created from `s`.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_common::FileFormatSchema;
    /// use alpm_files::FilesSchema;
    ///
    /// # fn main() -> Result<(), alpm_files::Error> {
    /// let data = r#"%FILES%
    /// usr/
    /// usr/bin/
    /// usr/bin/foo
    /// "#;
    /// let schema = FilesSchema::derive_from_str(data)?;
    /// matches!(schema, FilesSchema::V1(_));
    ///
    /// // Empty inputs may also be considered as a schema
    /// let data = "";
    /// let schema = FilesSchema::derive_from_str(data)?;
    /// matches!(schema, FilesSchema::V1(_));
    /// # Ok(())
    /// # }
    /// ```
    fn derive_from_str(s: &str) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        // Return an error if there is a first line, but it doesn't contain the expected section
        // header.
        if s.lines()
            .next()
            .is_some_and(|line| line != FilesSection::SECTION_KEYWORD)
        {
            return Err(Error::SchemaVersionIsUnknown);
        }

        // If there are no lines (empty file) or the first line contains the expected section
        // header, we can assume to deal with a version 1.
        Ok(Self::V1(SchemaVersion::new(Version::new(1, 0, 0))))
    }
}

impl Default for FilesSchema {
    /// Returns the default schema variant ([`FilesSchema::V1`]).
    fn default() -> Self {
        Self::V1(SchemaVersion::new(Version::new(1, 0, 0)))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use rstest::rstest;
    use tempfile::NamedTempFile;
    use testresult::TestResult;

    use super::*;

    const ALPM_DB_FILES_FULL: &str = r#"%FILES%
usr/
usr/bin/
usr/bin/foo
"#;
    const ALPM_DB_FILES_EMPTY: &str = "";
    const ALPM_REPO_FILES_FULL: &str = r#"%FILES%
usr/
usr/bin/
usr/bin/foo"#;
    const ALPM_REPO_FILES_EMPTY: &str = "%FILES%";

    /// Ensures that different types of full and empty alpm-files files can be parsed from file.
    #[rstest]
    #[case::alpm_db_files_full(ALPM_DB_FILES_FULL)]
    #[case::alpm_db_files_empty(ALPM_DB_FILES_EMPTY)]
    #[case::alpm_repo_files_full(ALPM_REPO_FILES_FULL)]
    #[case::alpm_repo_files_full(ALPM_REPO_FILES_EMPTY)]
    fn files_schema_derive_from_file_succeeds(#[case] data: &str) -> TestResult {
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{data}")?;

        let schema = FilesSchema::derive_from_file(temp_file.path())?;

        assert!(matches!(schema, FilesSchema::V1(_)));

        Ok(())
    }

    /// Ensures that different types of full and empty alpm-files files can be parsed from file.
    #[test]
    fn files_schema_derive_from_file_fails() -> TestResult {
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "%WRONG%")?;

        let result = FilesSchema::derive_from_file(temp_file.path());
        match result {
            Ok(schema) => {
                return Err(format!(
                    "Expected to fail with an Error::SchemaVersionIsUnknown but succeeded: {schema:?}"
                ).into());
            }
            Err(Error::SchemaVersionIsUnknown) => {}
            Err(error) => {
                return Err(format!(
                    "Expected to fail with an Error::SchemaVersionIsUnknown but got another error instead: {error}"
                ).into());
            }
        }

        Ok(())
    }

    /// Ensures that [`FilesSchema::inner`] returns the correct schema.
    #[test]
    fn files_schema_inner() {
        let schema_version = SchemaVersion::new(Version::new(1, 0, 0));
        let schema = FilesSchema::V1(schema_version.clone());
        assert_eq!(schema.inner(), &schema_version)
    }

    /// Ensures that [`FilesSchema::V1`] is the default.
    #[test]
    fn files_schema_default() {
        assert_eq!(
            FilesSchema::default(),
            FilesSchema::V1(SchemaVersion::new(Version::new(1, 0, 0)))
        )
    }
}
