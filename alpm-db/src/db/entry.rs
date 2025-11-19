//! Representation of installed [alpm-db] entries.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::{
    fmt::{Display, Formatter},
    fs::{self, create_dir_all},
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_common::MetadataFile;
use alpm_files::{Files, FilesSchema, FilesStyle, FilesStyleToString};
use alpm_mtree::Mtree;
use alpm_types::{FullVersion, Name};
use winnow::{
    ModalResult,
    Parser,
    combinator::{cut_err, eof, opt, peek, repeat},
    error::{AddContext, ContextError, ErrMode, ParserError, StrContext, StrContextValue},
    stream::Stream,
    token::{literal, take_until},
};

use crate::{Error, desc::DbDescFile};

/// File name of the [alpm-db-desc] metadata in an entry directory.
///
/// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
pub const DESC_FILE_NAME: &str = "desc";

/// File name of the [alpm-files] metadata in an entry directory.
///
/// [alpm-files]: https://alpm.archlinux.page/specifications/alpm-files.5.html
pub const FILES_FILE_NAME: &str = "files";

/// File name of the [alpm-mtree] metadata in an entry directory.
///
/// [alpm-mtree]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
pub const MTREE_FILE_NAME: &str = "mtree";

/// The name of a directory that stores the metadata of an installed package.
///
/// This combines an [alpm-package-name] and a **full** [alpm-package-version].
///
/// [alpm-package-name]: https://alpm.archlinux.page/specifications/alpm-package-name.7.html)
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

    /// Returns the directory name representation.
    ///
    /// Delegates to the [`Display`] implementation.
    pub fn as_dir_name(&self) -> String {
        format!("{self}")
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
                    "a package name followed by '-' and a full version with pkgrel",
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

impl<'a> TryFrom<&'a Path> for DatabaseEntryName {
    type Error = Error;

    /// Parses a [`DatabaseEntryName`] from the a path.
    ///
    /// The last component of the path is used as the entry name.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the path does not have a file name,
    /// - the file name cannot be converted to a string,
    /// - or the resulting string is not a valid entry name (see [`FromStr`] implementation).
    fn try_from(value: &'a Path) -> Result<Self, Self::Error> {
        let file_name = value.file_name().ok_or_else(|| Error::InvalidFile {
            path: value.to_path_buf(),
            context: "extracting entry name from path".to_string(),
        })?;

        let raw = file_name.to_str().ok_or_else(|| Error::InvalidFileName {
            path: value.to_path_buf(),
            context: "converting entry name to string".to_string(),
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
        let raw = fs::read(path).map_err(|source| Error::IoPathError {
            path: path.to_path_buf(),
            context: "reading mtree file",
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
/// - a [alpm-files] file (`[Files]`),
/// - and an [alpm-mtree] file (`[DatabaseEntryMtree]`).
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
/// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
/// [alpm-files]: https://alpm.archlinux.page/specifications/alpm-files.5.html
/// [alpm-mtree]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
#[derive(Clone, Debug)]
pub struct DatabaseEntry {
    /// The entry name.
    pub name: DatabaseEntryName,

    /// The [alpm-db-desc] file.
    ///
    /// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    pub desc: DbDescFile,

    /// The [alpm-files] file.
    ///
    /// [alpm-files]: https://alpm.archlinux.page/specifications/alpm-files.5.html
    pub files: Files,

    /// The [alpm-mtree] file.
    ///
    /// [alpm-mtree]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    pub mtree: DatabaseEntryMtree,
}

impl DatabaseEntry {
    /// Creates a new database entry from individual components.
    pub fn new(
        name: DatabaseEntryName,
        desc: DbDescFile,
        files: Files,
        mtree: DatabaseEntryMtree,
    ) -> Self {
        Self {
            name,
            desc,
            files,
            mtree,
        }
    }

    /// Parses an entry directory on disk.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the path is not a directory,
    /// - the [`DatabaseEntryName`] cannot be parsed,
    /// - the [alpm-db-desc] file cannot be read or parsed,
    /// - the [alpm-files] file cannot be read or parsed,
    /// - or the [alpm-mtree] file cannot be read or parsed.
    ///
    /// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    /// [alpm-files]: https://alpm.archlinux.page/specifications/alpm-files.5.html
    /// [alpm-mtree]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    pub fn from_dir(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        if !path.is_dir() {
            return Err(alpm_common::Error::NotADirectory {
                path: path.to_path_buf(),
            }
            .into());
        }
        let name = DatabaseEntryName::try_from(path)?;

        let desc_path = path.join(DESC_FILE_NAME);
        let desc = DbDescFile::from_file(&desc_path)?;

        let files_path = path.join(FILES_FILE_NAME);
        let files = Files::from_file_with_schema(&files_path, Some(FilesSchema::default()))?;

        let mtree = DatabaseEntryMtree::from_file(path.join(MTREE_FILE_NAME))?;

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
    /// - writing the [alpm-files] file fails,
    /// - or writing the [alpm-mtree] file fails.
    ///
    /// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    /// [alpm-files]: https://alpm.archlinux.page/specifications/alpm-files.5.html
    /// [alpm-mtree]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    pub fn write_to(&self, root: impl AsRef<Path>) -> Result<PathBuf, Error> {
        let root = root.as_ref();
        let path = root.join(self.name.as_dir_name());
        create_dir_all(&path).map_err(|source| Error::IoPathError {
            path: path.clone(),
            context: "creating database entry directory",
            source,
        })?;

        let desc_path = path.join(DESC_FILE_NAME);
        fs::write(&desc_path, self.desc.to_string()).map_err(|source| Error::IoPathError {
            path: desc_path,
            context: "writing desc component",
            source,
        })?;

        let files_path = path.join(FILES_FILE_NAME);
        fs::write(&files_path, self.files.to_string(FilesStyle::Db)).map_err(|source| {
            Error::IoPathError {
                path: files_path,
                context: "writing files component",
                source,
            }
        })?;

        let mtree_path = path.join(MTREE_FILE_NAME);
        fs::write(&mtree_path, self.mtree.raw.clone()).map_err(|source| Error::IoPathError {
            path: mtree_path,
            context: "writing mtree component",
            source,
        })?;

        Ok(path)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::str::FromStr;

    use alpm_common::MetadataFile;
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
        let files = Files::from_str_with_schema(FILES_DATA, Some(FilesSchema::default()))?;
        let mtree = DatabaseEntryMtree::from_str(MTREE_DATA)?;
        Ok(DatabaseEntry::new(name, desc, files, mtree))
    }

    #[test]
    fn write_entry() -> TestResult {
        let tmp = tempfile::tempdir()?;

        let entry = create_sample_entry()?;
        entry.write_to(tmp.path())?;

        let loaded = DatabaseEntry::from_dir(tmp.path().join(entry.name.as_dir_name()))?;
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
}
