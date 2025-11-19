//! Representation of installed [alpm-db] entries.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::{
    fmt::{Display, Formatter},
    fs::{create_dir_all, read, symlink_metadata, write as fs_write},
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_common::MetadataFile;
use alpm_mtree::Mtree;
use alpm_types::{DESC_FILE_NAME, FILES_FILE_NAME, FullVersion, MTREE_FILE_NAME, Name};
use fluent_i18n::t;
use winnow::{
    ModalResult,
    Parser,
    combinator::{cut_err, eof, opt, peek, repeat},
    error::{AddContext, ContextError, ErrMode, ParserError, StrContext, StrContextValue},
    stream::Stream,
    token::{literal, take_until},
};

use crate::{
    Error,
    desc::DbDescFile,
    files::{DbFiles, DbFilesSchema},
};

/// The name of a directory that stores the metadata of an installed package.
///
/// This combines an [alpm-package-name] and a **full** [alpm-package-version].
///
/// [alpm-package-name]: https://alpm.archlinux.page/specifications/alpm-package-name.7.html
/// [alpm-package-version]: https://alpm.archlinux.page/specifications/alpm-package-version.7.html
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DatabaseEntryName {
    /// The package name.
    ///
    /// See
    /// [alpm-package-name](https://alpm.archlinux.page/specifications/alpm-package-name.7.html).
    pub name: Name,

    /// The full package version.
    ///
    /// See
    /// [alpm-package-version](https://alpm.archlinux.page/specifications/alpm-package-version.7.html).
    pub version: FullVersion,
}

impl DatabaseEntryName {
    /// Creates a new entry name.
    pub fn new(name: Name, version: FullVersion) -> Self {
        Self { name, version }
    }

    /// Returns the directory name as a [`PathBuf`].
    ///
    /// Delegates to [`ToString::to_string`] and converts the result to a [`PathBuf`].
    pub fn as_path_buf(&self) -> PathBuf {
        PathBuf::from(self.to_string())
    }

    /// Parses a [`DatabaseEntryName`] from a string slice.
    ///
    /// Here is a brief overview of the parsing logic:
    ///
    /// - Counts dashes to determine how many belong to the package name,
    /// - parses that segment with [`Name::parser`],
    /// - consumes the separating dash,
    /// - and finally parses the trailing full version using [`FullVersion::parser`].
    ///
    /// Also ensures that the entire input is consumed.
    pub fn parser(input: &mut &str) -> ModalResult<DatabaseEntryName> {
        let dashes = input.chars().filter(|c| *c == '-').count();
        if dashes < 2 {
            let context_error = ContextError::from_input(input).add_context(
                input,
                &input.checkpoint(),
                StrContext::Expected(StrContextValue::Description(
                    "a package name followed by '-' and a full alpm-package-version",
                )),
            );
            return Err(ErrMode::Cut(context_error));
        }

        let dashes_in_name = dashes.saturating_sub(2);
        let name = cut_err(
            repeat::<_, _, (), _, _>(
                dashes_in_name + 1,
                (opt("-"), take_until(0.., "-"), peek("-")),
            )
            .take()
            .and_then(Name::parser),
        )
        .context(StrContext::Label("alpm-package-name"))
        .parse_next(input)?;

        literal("-").parse_next(input)?;

        let version = cut_err(FullVersion::parser.context(StrContext::Expected(
            StrContextValue::Description("a full alpm-package-version (full or full with epoch)"),
        )))
        .parse_next(input)?;

        eof.parse_next(input)?;

        Ok(DatabaseEntryName::new(name, version))
    }
}

impl TryFrom<&Path> for DatabaseEntryName {
    type Error = Error;

    /// Creates a [`DatabaseEntryName`] from the file name of a [`Path`].
    ///
    /// The last component of the path is used as the entry name.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the path is not a directory,
    /// - the path is a symlink,
    /// - the path does not have a file name,
    /// - the file name cannot be converted to a string,
    /// - or the file name cannot be parsed as a valid entry name (see [`FromStr`] implementation).
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let metadata = symlink_metadata(value).map_err(|source| Error::IoPathError {
            path: value.to_path_buf(),
            context: t!("error-io-path-entry-name-metadata"),
            source,
        })?;
        if metadata.file_type().is_symlink() {
            return Err(Error::InvalidFile {
                path: value.to_path_buf(),
                context: t!("error-invalid-file-context-entry-name-symlink"),
            });
        }
        if !metadata.is_dir() {
            return Err(alpm_common::Error::NotADirectory {
                path: value.to_path_buf(),
            }
            .into());
        }

        let file_name = value.file_name().ok_or_else(|| Error::InvalidFile {
            path: value.to_path_buf(),
            context: t!("error-invalid-file-context-entry-name"),
        })?;

        let raw = file_name.to_str().ok_or_else(|| Error::InvalidFileName {
            path: value.to_path_buf(),
            context: t!("error-invalid-file-name-context-to-string"),
        })?;

        Self::from_str(raw)
    }
}

impl Display for DatabaseEntryName {
    /// Formats the entry name as `name-version`.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.name, self.version)
    }
}

impl FromStr for DatabaseEntryName {
    type Err = Error;

    /// Parses a [`DatabaseEntryName`] from a string slice.
    ///
    /// Delegates to [`DatabaseEntryName::parser`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`DatabaseEntryName::parser`] fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parser.parse(s).map_err(Error::from)
    }
}

/// Represents the [alpm-mtree] component of a database entry.
///
/// [alpm-mtree]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
#[derive(Clone, Debug)]
pub struct DatabaseEntryMtree {
    /// The raw bytes of the mtree data.
    ///
    /// These are stored to allow writing the data back to disk without compression.
    pub raw: Vec<u8>,

    /// The parsed mtree representation.
    pub parsed: Mtree,
}

impl FromStr for DatabaseEntryMtree {
    type Err = Error;

    /// Creates a [`DatabaseEntryMtree`] from a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if [`DatabaseEntryMtree::from_bytes`] fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_bytes(s.as_bytes().to_vec())
    }
}

impl DatabaseEntryMtree {
    /// Reads mtree data the given `path`.
    ///
    /// # Errors
    ///
    /// Returns an error when reading the file or parsing the data fails.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let raw = read(path).map_err(|source| Error::IoPathError {
            path: path.to_path_buf(),
            context: t!("error-io-path-mtree-file-read"),
            source,
        })?;
        Self::from_bytes(raw)
    }

    /// Creates a [`DatabaseEntryMtree`] from a byte vector.
    ///
    /// Delegates to [`Mtree::from_reader_with_schema`].
    ///
    /// # Errors
    ///
    /// Returns an error if the bytes cannot be interpreted as [`Mtree`] data.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        let parsed = Mtree::from_reader_with_schema(bytes.as_slice(), None)?;
        Ok(Self { raw: bytes, parsed })
    }
}

/// A complete entry inside an [alpm-db] database.
///
/// A database entry consists of:
///
/// - a directory name (`[DatabaseEntryName]`),
/// - a [alpm-db-desc] file (`[DbDescFile]`),
/// - a [alpm-db-files] file (`[DbFiles]`),
/// - and an [alpm-mtree] file (`[DatabaseEntryMtree]`).
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
/// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
/// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
/// [alpm-mtree]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
#[derive(Clone, Debug)]
pub struct DatabaseEntry {
    /// The entry name.
    pub name: DatabaseEntryName,

    /// The [alpm-db-desc] file.
    ///
    /// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    pub desc: DbDescFile,

    /// The [alpm-db-files] file.
    ///
    /// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
    pub files: DbFiles,

    /// The [alpm-mtree] file.
    ///
    /// [alpm-mtree]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    pub mtree: DatabaseEntryMtree,
}

impl PartialEq for DatabaseEntry {
    /// Compares two database entries for equality.
    ///
    /// Only compares the name component.
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialOrd for DatabaseEntry {
    /// Compares two database entries for ordering.
    ///
    /// Only compares the name component.
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for DatabaseEntry {}

impl Ord for DatabaseEntry {
    /// Compares two database entries for ordering.
    ///
    /// Only compares the name component.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl TryFrom<&Path> for DatabaseEntry {
    type Error = Error;

    /// Parses an entry directory on disk.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the path is not a directory,
    /// - the [`DatabaseEntryName`] cannot be parsed,
    /// - the [alpm-db-desc] file cannot be read or parsed,
    /// - the [alpm-db-files] file cannot be read or parsed,
    /// - or the [alpm-mtree] file cannot be read or parsed.
    ///
    /// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    /// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
    /// [alpm-mtree]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        if !path.is_dir() {
            return Err(alpm_common::Error::NotADirectory {
                path: path.to_path_buf(),
            }
            .into());
        }
        let name = DatabaseEntryName::try_from(path)?;

        let desc_path = path.join(DESC_FILE_NAME);
        let desc = DbDescFile::from_file(&desc_path)?;
        let (desc_name, desc_version) = match &desc {
            DbDescFile::V1(data) => (&data.name, &data.version),
            DbDescFile::V2(data) => (&data.name, &data.version),
        };
        if desc_name != &name.name || desc_version != &name.version {
            return Err(Error::DatabaseEntryNameMismatch(Box::new(
                crate::error::DatabaseEntryNameMismatch {
                    path: Some(path.to_path_buf()),
                    entry_name: name.clone(),
                    desc_name: desc_name.clone(),
                    desc_version: desc_version.clone(),
                },
            )));
        }

        let files_path = path.join(FILES_FILE_NAME);
        let files = DbFiles::from_file_with_schema(&files_path, Some(DbFilesSchema::default()))?;

        let mtree = DatabaseEntryMtree::from_file(path.join(MTREE_FILE_NAME))?;

        Ok(Self {
            name,
            desc,
            files,
            mtree,
        })
    }
}

impl TryFrom<PathBuf> for DatabaseEntry {
    type Error = Error;

    /// Parses an entry directory on disk.
    ///
    /// Delegates to the [`TryFrom<&Path>`] implementation.
    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        Self::try_from(value.as_path())
    }
}

impl DatabaseEntry {
    /// Creates a new database entry from individual components.
    ///
    /// # Errors
    ///
    /// Returns an error if the [`DatabaseEntryName`] does not match the name and version from
    /// [`DbDescFile`].
    pub fn new(
        name: DatabaseEntryName,
        desc: DbDescFile,
        files: DbFiles,
        mtree: DatabaseEntryMtree,
    ) -> Result<Self, Error> {
        let (desc_name, desc_version) = match &desc {
            DbDescFile::V1(data) => (&data.name, &data.version),
            DbDescFile::V2(data) => (&data.name, &data.version),
        };
        if desc_name != &name.name || desc_version != &name.version {
            return Err(Error::DatabaseEntryNameMismatch(Box::new(
                crate::error::DatabaseEntryNameMismatch {
                    path: None,
                    entry_name: name,
                    desc_name: desc_name.clone(),
                    desc_version: desc_version.clone(),
                },
            )));
        }
        Ok(Self {
            name,
            desc,
            files,
            mtree,
        })
    }

    /// Writes this entry to a database directory.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - creating the entry directory fails,
    /// - writing the [alpm-db-desc] file fails,
    /// - writing the [alpm-db-files] file fails,
    /// - or writing the [alpm-mtree] file fails.
    ///
    /// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    /// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
    /// [alpm-mtree]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    pub(crate) fn write_to_db(&self, root: impl AsRef<Path>) -> Result<PathBuf, Error> {
        let root = root.as_ref();
        let path = root.join(self.name.as_path_buf());
        create_dir_all(&path).map_err(|source| Error::IoPathError {
            path: path.clone(),
            context: t!("error-io-path-entry-dir-create"),
            source,
        })?;

        let desc_path = path.join(DESC_FILE_NAME);
        fs_write(&desc_path, self.desc.to_string()).map_err(|source| Error::IoPathError {
            path: desc_path,
            context: t!("error-io-path-write-desc"),
            source,
        })?;

        let files_path = path.join(FILES_FILE_NAME);
        fs_write(&files_path, self.files.to_string()).map_err(|source| Error::IoPathError {
            path: files_path,
            context: t!("error-io-path-write-files"),
            source,
        })?;

        let mtree_path = path.join(MTREE_FILE_NAME);
        fs_write(&mtree_path, self.mtree.raw.clone()).map_err(|source| Error::IoPathError {
            path: mtree_path,
            context: t!("error-io-path-write-mtree"),
            source,
        })?;

        Ok(path)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::str::FromStr;

    use alpm_common::MetadataFile;
    use alpm_types::FullVersion;
    use rstest::rstest;
    use testresult::TestResult;

    use super::*;

    const DESC_DATA: &str = r#"%NAME%
foo

%VERSION%
1.0.0-1

%BASE%
foo

%DESC%
An example package

%URL%
https://example.org

%ARCH%
x86_64

%BUILDDATE%
1733737242

%INSTALLDATE%
1733737243

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%SIZE%
123

%VALIDATION%
pgp
"#;

    const FILES_DATA: &str = r#"%FILES%
usr/
usr/bin/
usr/bin/foo
"#;

    const MTREE_DATA: &str = r#"#mtree
/set mode=644 uid=0 gid=0 type=file
./some_file time=1700000000.0 size=1337 sha256digest=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
./some_link type=link link=some_file time=1700000000.0
./some_dir type=dir time=1700000000.0
"#;

    pub fn create_sample_entry() -> Result<DatabaseEntry, Error> {
        let name = DatabaseEntryName::from_str("foo-1.0.0-1")?;
        let desc = crate::desc::DbDescFile::from_str_with_schema(DESC_DATA, None)?;
        let files = DbFiles::from_str_with_schema(FILES_DATA, Some(DbFilesSchema::default()))?;
        let mtree = DatabaseEntryMtree::from_str(MTREE_DATA)?;
        DatabaseEntry::new(name, desc, files, mtree)
    }

    #[test]
    fn write_entry() -> TestResult {
        let tmp = tempfile::tempdir()?;

        let entry = create_sample_entry()?;
        entry.write_to_db(tmp.path())?;

        let loaded = DatabaseEntry::try_from(tmp.path().join(entry.name.as_path_buf()))?;
        assert_eq!(loaded.name, DatabaseEntryName::from_str("foo-1.0.0-1")?);
        assert_eq!(loaded.files.as_ref(), entry.files.as_ref());
        assert_eq!(loaded.mtree.raw, entry.mtree.raw);
        assert_eq!(loaded.mtree.parsed, entry.mtree.parsed);
        Ok(())
    }

    #[rstest]
    #[case("foo-1.0.0-1", "foo", "1.0.0-1")]
    #[case("foo-bar-1:1.0.0-1", "foo-bar", "1:1.0.0-1")]
    #[case("foo-bar-baz-2:3.0.0-2", "foo-bar-baz", "2:3.0.0-2")]
    fn parse_entry_name(
        #[case] input: &str,
        #[case] expected_name: &str,
        #[case] expected_version: &str,
    ) -> TestResult {
        let name = DatabaseEntryName::from_str(input)?;
        assert_eq!(name.name.as_ref(), expected_name);
        assert_eq!(name.version.to_string(), expected_version);
        Ok(())
    }

    #[test]
    fn new_entry_mismatch_returns_error() -> TestResult {
        let name = DatabaseEntryName::from_str("foo-1.0.0-1")?;
        let mut desc = DbDescFile::from_str_with_schema(DESC_DATA, None)?;

        match &mut desc {
            DbDescFile::V1(data) => data.version = FullVersion::from_str("2.0.0-1")?,
            DbDescFile::V2(data) => data.version = FullVersion::from_str("2.0.0-1")?,
        }

        let files = DbFiles::from_str_with_schema(FILES_DATA, Some(DbFilesSchema::default()))?;
        let mtree = DatabaseEntryMtree::from_str(MTREE_DATA)?;

        let err = DatabaseEntry::new(name, desc, files, mtree).unwrap_err();
        assert!(matches!(
            err,
            Error::DatabaseEntryNameMismatch(details) if details.path.is_none()
        ));
        Ok(())
    }
}
