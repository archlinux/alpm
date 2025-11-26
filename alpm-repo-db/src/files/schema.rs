//! Schema definition for the [alpm-repo-files] format.
//!
//! [alpm-repo-files]: https://alpm.archlinux.page/specifications/alpm-repo-files.5.html

use std::{fs::File, io::Read, path::Path};

use alpm_common::FileFormatSchema;
use alpm_types::{SchemaVersion, semver_version::Version};
use fluent_i18n::t;

use crate::files::{Error, v1::FilesSection};

/// A schema for the [alpm-repo-files] format.
///
/// [alpm-repo-files]: https://alpm.archlinux.page/specifications/alpm-repo-files.5.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RepoFilesSchema {
    /// Version 1 of the [alpm-repo-files] specification.
    ///
    /// [alpm-repo-files]: https://alpm.archlinux.page/specifications/alpm-repo-files.5.html
    V1(SchemaVersion),
}

impl FileFormatSchema for RepoFilesSchema {
    type Err = Error;

    /// Returns a reference to the inner [`SchemaVersion`].
    fn inner(&self) -> &SchemaVersion {
        match self {
            RepoFilesSchema::V1(v) => v,
        }
    }

    /// Creates a new [`RepoFilesSchema`] from a file [`Path`].
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
    /// use alpm_repo_db::files::RepoFilesSchema;
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
    /// let schema = RepoFilesSchema::derive_from_file(temp_file.path())?;
    /// matches!(schema, RepoFilesSchema::V1(_));
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
            context: t!("error-io-path-context-deriving-schema-version-from-alpm-repo-files-file"),
            source,
        })?)
    }

    /// Creates a new [`RepoFilesSchema`] from a [`Read`] implementation.
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
    /// use alpm_repo_db::files::RepoFilesSchema;
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
    /// let schema = RepoFilesSchema::derive_from_reader(temp_file)?;
    /// matches!(schema, RepoFilesSchema::V1(_));
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
                context: t!("error-io-context-deriving-a-schema-version-from-alpm-repo-files-data"),
                source,
            })?;
        Self::derive_from_str(&buf)
    }

    /// Creates a new [`RepoFilesSchema`] from a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - a [`RepoFilesSchema`] cannot be derived from `s`,
    /// - or a [`RepoFilesV1`][`crate::files::RepoFilesV1`] cannot be created from `s`.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_common::FileFormatSchema;
    /// use alpm_repo_db::files::RepoFilesSchema;
    ///
    /// # fn main() -> Result<(), alpm_repo_db::files::Error> {
    /// let data = r#"%FILES%
    /// usr/
    /// usr/bin/
    /// usr/bin/foo
    /// "#;
    /// let schema = RepoFilesSchema::derive_from_str(data)?;
    /// matches!(schema, RepoFilesSchema::V1(_));
    /// # Ok(())
    /// # }
    /// ```
    fn derive_from_str(s: &str) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        match s.lines().next() {
            Some(line) if line == FilesSection::SECTION_KEYWORD => {
                Ok(Self::V1(SchemaVersion::new(Version::new(1, 0, 0))))
            }
            _ => Err(Error::UnknownSchemaVersion),
        }
    }
}

impl Default for RepoFilesSchema {
    /// Returns the default schema variant ([`RepoFilesSchema::V1`]).
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

    const ALPM_REPO_FILES_FULL: &str = r#"%FILES%
usr/
usr/bin/
usr/bin/foo
"#;
    const ALPM_REPO_FILES_EMPTY: &str = "%FILES%\n";
    const ALPM_REPO_FILES_EMPTY_NO_HEADER: &str = "";

    /// Ensures that different types of full and empty alpm-repo-files files can be parsed from
    /// file.
    #[rstest]
    #[case::alpm_repo_files_full(ALPM_REPO_FILES_FULL)]
    #[case::alpm_repo_files_empty(ALPM_REPO_FILES_EMPTY)]
    fn files_schema_derive_from_file_succeeds(#[case] data: &str) -> TestResult {
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{data}")?;

        let schema = RepoFilesSchema::derive_from_file(temp_file.path())?;

        assert!(matches!(schema, RepoFilesSchema::V1(_)));

        Ok(())
    }

    /// Ensures that files with wrong headers fail to derive the schema.
    #[rstest]
    #[case::wrong_header("%WRONG%")]
    #[case::missing_header(ALPM_REPO_FILES_EMPTY_NO_HEADER)]
    fn files_schema_derive_from_file_fails(#[case] data: &str) -> TestResult {
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{data}")?;

        let result = RepoFilesSchema::derive_from_file(temp_file.path());
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

    /// Ensures that [`RepoFilesSchema::inner`] returns the correct schema.
    #[test]
    fn files_schema_inner() {
        let schema_version = SchemaVersion::new(Version::new(1, 0, 0));
        let schema = RepoFilesSchema::V1(schema_version.clone());
        assert_eq!(schema.inner(), &schema_version)
    }

    /// Ensures that [`RepoFilesSchema::V1`] is the default.
    #[test]
    fn files_schema_default() {
        assert_eq!(
            RepoFilesSchema::default(),
            RepoFilesSchema::V1(SchemaVersion::new(Version::new(1, 0, 0)))
        )
    }
}
