//! Schema definition for the [alpm-db-files] format.
//!
//! [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html

use std::{fs::File, io::Read, path::Path};

use alpm_common::FileFormatSchema;
use alpm_types::{SchemaVersion, semver_version::Version};
use fluent_i18n::t;

use crate::files::{Error, v1::FilesSection};

/// A schema for the [alpm-db-files] format.
///
/// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DbFilesSchema {
    /// Version 1 of the [alpm-db-files] specification.
    ///
    /// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
    V1(SchemaVersion),
}

impl FileFormatSchema for DbFilesSchema {
    type Err = Error;

    /// Returns a reference to the inner [`SchemaVersion`].
    fn inner(&self) -> &SchemaVersion {
        match self {
            DbFilesSchema::V1(v) => v,
        }
    }

    /// Creates a new [`DbFilesSchema`] from a file [`Path`].
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
    /// use alpm_db::files::DbFilesSchema;
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
    /// let schema = DbFilesSchema::derive_from_file(temp_file.path())?;
    /// matches!(schema, DbFilesSchema::V1(_));
    ///
    /// // Empty inputs may also be considered as a schema
    /// let mut temp_file = NamedTempFile::new()?;
    /// let schema = DbFilesSchema::derive_from_file(temp_file.path())?;
    /// matches!(schema, DbFilesSchema::V1(_));
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
            context: t!("error-io-path-context-deriving-schema-version-from-alpm-db-files-file"),
            source,
        })?)
    }

    /// Creates a new [`DbFilesSchema`] from a [`Read`] implementation.
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
    /// use alpm_db::files::DbFilesSchema;
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
    /// let schema = DbFilesSchema::derive_from_reader(temp_file)?;
    /// matches!(schema, DbFilesSchema::V1(_));
    ///
    /// // Empty inputs may also be considered as a schema
    /// let mut temp_file = tempfile()?;
    /// let schema = DbFilesSchema::derive_from_reader(temp_file)?;
    /// matches!(schema, DbFilesSchema::V1(_));
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
                context: t!("error-io-context-deriving-a-schema-version-from-alpm-db-files-data"),
                source,
            })?;
        Self::derive_from_str(&buf)
    }

    /// Creates a new [`DbFilesSchema`] from a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - a [`DbFilesSchema`] cannot be derived from `s`,
    /// - or a [`DbFilesV1`][`crate::files::DbFilesV1`] cannot be created from `s`.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_common::FileFormatSchema;
    /// use alpm_db::files::DbFilesSchema;
    ///
    /// # fn main() -> Result<(), alpm_db::files::Error> {
    /// let data = r#"%FILES%
    /// usr/
    /// usr/bin/
    /// usr/bin/foo
    /// "#;
    /// let schema = DbFilesSchema::derive_from_str(data)?;
    /// matches!(schema, DbFilesSchema::V1(_));
    ///
    /// // Empty inputs may also be considered as a schema
    /// let data = "";
    /// let schema = DbFilesSchema::derive_from_str(data)?;
    /// matches!(schema, DbFilesSchema::V1(_));
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
            return Err(Error::UnknownSchemaVersion);
        }

        // If there are no lines (empty file) or the first line contains the expected section
        // header, we can assume to deal with a version 1.
        Ok(Self::V1(SchemaVersion::new(Version::new(1, 0, 0))))
    }
}

impl Default for DbFilesSchema {
    /// Returns the default schema variant ([`DbFilesSchema::V1`]).
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

    /// Ensures that different types of full and empty alpm-db-files files can be parsed from file.
    #[rstest]
    #[case::alpm_db_files_full(ALPM_DB_FILES_FULL)]
    #[case::alpm_db_files_empty(ALPM_DB_FILES_EMPTY)]
    #[case::alpm_repo_files_full(ALPM_REPO_FILES_FULL)]
    #[case::alpm_repo_files_full(ALPM_REPO_FILES_EMPTY)]
    fn files_schema_derive_from_file_succeeds(#[case] data: &str) -> TestResult {
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{data}")?;

        let schema = DbFilesSchema::derive_from_file(temp_file.path())?;

        assert!(matches!(schema, DbFilesSchema::V1(_)));

        Ok(())
    }

    /// Ensures that different types of full and empty alpm-db-files files can be parsed from file.
    #[test]
    fn files_schema_derive_from_file_fails() -> TestResult {
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "%WRONG%")?;

        let result = DbFilesSchema::derive_from_file(temp_file.path());
        match result {
            Ok(schema) => {
                panic!(
                    "Expected to fail with an Error::UnknownSchemaVersion but succeeded: {schema:?}"
                );
            }
            Err(Error::UnknownSchemaVersion) => {}
            Err(error) => {
                panic!(
                    "Expected to fail with an Error::UnknownSchemaVersion but got another error instead: {error}"
                );
            }
        }

        Ok(())
    }

    /// Ensures that [`DbFilesSchema::inner`] returns the correct schema.
    #[test]
    fn files_schema_inner() {
        let schema_version = SchemaVersion::new(Version::new(1, 0, 0));
        let schema = DbFilesSchema::V1(schema_version.clone());
        assert_eq!(schema.inner(), &schema_version)
    }

    /// Ensures that [`DbFilesSchema::V1`] is the default.
    #[test]
    fn files_schema_default() {
        assert_eq!(
            DbFilesSchema::default(),
            DbFilesSchema::V1(SchemaVersion::new(Version::new(1, 0, 0)))
        )
    }
}
