//! Handling for the ALPM-MTREE file format.

pub mod v2;
use std::{
    fmt::{Display, Write},
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_common::{FileFormatSchema, MetadataFile};

use crate::{Error, MtreeSchema, mtree_buffer_to_string, parse_mtree_v2};

/// A representation of the [ALPM-MTREE] file format.
///
/// Tracks all available versions of the file format.
///
/// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(untagged)]
pub enum Mtree {
    V1(Vec<crate::mtree::v2::Path>),
    V2(Vec<crate::mtree::v2::Path>),
}

impl MetadataFile<MtreeSchema> for Mtree {
    type Err = Error;

    /// Creates a [`Mtree`] from `file`, optionally validated using a [`MtreeSchema`].
    ///
    /// Opens the `file` and defers to [`Mtree::from_reader_with_schema`].
    ///
    /// # Note
    ///
    /// To automatically derive the [`MtreeSchema`], use [`Mtree::from_file`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_common::{FileFormatSchema, MetadataFile};
    /// use alpm_mtree::{Mtree, MtreeSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// // Prepare a file with ALPM-MTREE data
    /// let file = {
    ///     let mtree_data = r#"#mtree
    /// /set mode=644 uid=0 gid=0 type=file
    /// ./some_file time=1700000000.0 size=1337 sha256digest=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
    /// ./some_link type=link link=some_file time=1700000000.0
    /// ./some_dir type=dir time=1700000000.0
    /// "#;
    ///     let mtree_file = tempfile::NamedTempFile::new()?;
    ///     let mut output = File::create(&mtree_file)?;
    ///     write!(output, "{}", mtree_data)?;
    ///     mtree_file
    /// };
    ///
    /// let mtree = Mtree::from_file_with_schema(
    ///     file.path().to_path_buf(),
    ///     Some(MtreeSchema::V2(SchemaVersion::new(Version::new(2, 0, 0)))),
    /// )?;
    /// # let mtree_version = match mtree {
    /// #     Mtree::V1(_) => "1",
    /// #     Mtree::V2(_) => "2",
    /// # };
    /// # assert_eq!("2", mtree_version);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - the `file` cannot be opened for reading,
    /// - no variant of [`Mtree`] can be constructed from the contents of `file`,
    /// - or `schema` is [`Some`] and the [`MtreeSchema`] does not match the contents of `file`.
    fn from_file_with_schema(
        file: impl AsRef<Path>,
        schema: Option<MtreeSchema>,
    ) -> Result<Self, Error> {
        let file = file.as_ref();
        Self::from_reader_with_schema(
            File::open(file).map_err(|source| {
                Error::IoPath(PathBuf::from(file), "opening the file for reading", source)
            })?,
            schema,
        )
    }

    /// Creates a [`Mtree`] from a `reader`, optionally validated using a
    /// [`MtreeSchema`].
    ///
    /// Reads the `reader` to string (and decompresses potentially gzip compressed data on-the-fly).
    /// Then defers to [`Mtree::from_str_with_schema`].
    ///
    /// # Note
    ///
    /// To automatically derive the [`MtreeSchema`], use [`Mtree::from_reader`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_common::MetadataFile;
    /// use alpm_mtree::{Mtree, MtreeSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// // Prepare a reader with ALPM-MTREE data
    /// let reader = {
    ///     let mtree_data = r#"#mtree
    /// /set mode=644 uid=0 gid=0 type=file
    /// ./some_file time=1700000000.0 size=1337 sha256digest=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
    /// ./some_link type=link link=some_file time=1700000000.0
    /// ./some_dir type=dir time=1700000000.0
    /// "#;
    ///     let mtree_file = tempfile::NamedTempFile::new()?;
    ///     let mut output = File::create(&mtree_file)?;
    ///     write!(output, "{}", mtree_data)?;
    ///     File::open(&mtree_file.path())?
    /// };
    ///
    /// let mtree = Mtree::from_reader_with_schema(
    ///     reader,
    ///     Some(MtreeSchema::V2(SchemaVersion::new(Version::new(2, 0, 0)))),
    /// )?;
    /// # let mtree_version = match mtree {
    /// #     Mtree::V1(_) => "1",
    /// #     Mtree::V2(_) => "2",
    /// # };
    /// # assert_eq!("2", mtree_version);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - the `reader` cannot be read to string,
    /// - no variant of [`Mtree`] can be constructed from the contents of the `reader`,
    /// - or `schema` is [`Some`] and the [`MtreeSchema`] does not match the contents of the
    ///   `reader`.
    fn from_reader_with_schema(
        reader: impl std::io::Read,
        schema: Option<MtreeSchema>,
    ) -> Result<Self, Error> {
        let mut buffer = Vec::new();
        let mut buf_reader = BufReader::new(reader);
        buf_reader
            .read_to_end(&mut buffer)
            .map_err(|source| Error::Io("reading ALPM-MTREE data", source))?;
        Self::from_str_with_schema(&mtree_buffer_to_string(buffer)?, schema)
    }

    /// Creates a [`Mtree`] from string slice, optionally validated using a
    /// [`MtreeSchema`].
    ///
    /// If `schema` is [`None`] attempts to detect the [`MtreeSchema`] from `s`.
    /// Attempts to create a [`Mtree`] variant that corresponds to the [`MtreeSchema`].
    ///
    /// # Note
    ///
    /// To automatically derive the [`MtreeSchema`], use [`Mtree::from_str`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_common::MetadataFile;
    /// use alpm_mtree::{Mtree, MtreeSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// let mtree_v2 = r#"
    /// #mtree
    /// /set mode=644 uid=0 gid=0 type=file
    /// ./some_file time=1700000000.0 size=1337 sha256digest=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
    /// ./some_link type=link link=some_file time=1700000000.0
    /// ./some_dir type=dir time=1700000000.0
    /// "#;
    /// let mtree = Mtree::from_str_with_schema(
    ///     mtree_v2,
    ///     Some(MtreeSchema::V2(SchemaVersion::new(Version::new(2, 0, 0)))),
    /// )?;
    /// # let mtree_version = match mtree {
    /// #     Mtree::V1(_) => "1",
    /// #     Mtree::V2(_) => "2",
    /// # };
    /// # assert_eq!("2", mtree_version);
    ///
    /// let mtree_v1 = r#"
    /// #mtree
    /// /set mode=644 uid=0 gid=0 type=file
    /// ./some_file time=1700000000.0 size=1337 sha256digest=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef md5digest=d3b07384d113edec49eaa6238ad5ff00
    /// ./some_link type=link link=some_file time=1700000000.0
    /// ./some_dir type=dir time=1700000000.0
    /// "#;
    /// let mtree = Mtree::from_str_with_schema(
    ///     mtree_v1,
    ///     Some(MtreeSchema::V1(SchemaVersion::new(Version::new(1, 0, 0)))),
    /// )?;
    /// # let mtree_version = match mtree {
    /// #     Mtree::V1(_) => "1",
    /// #     Mtree::V2(_) => "2",
    /// # };
    /// # assert_eq!("1", mtree_version);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - `schema` is [`Some`] and the specified variant of [`Mtree`] cannot be constructed from
    ///   `s`,
    /// - `schema` is [`None`] and
    ///   - a [`MtreeSchema`] cannot be derived from `s`,
    ///   - or the detected variant of [`Mtree`] cannot be constructed from `s`.
    fn from_str_with_schema(s: &str, schema: Option<MtreeSchema>) -> Result<Self, Error> {
        let schema = match schema {
            Some(schema) => schema,
            None => MtreeSchema::derive_from_str(s)?,
        };

        match schema {
            MtreeSchema::V1(_) => Ok(Mtree::V1(parse_mtree_v2(s.to_string())?)),
            MtreeSchema::V2(_) => Ok(Mtree::V2(parse_mtree_v2(s.to_string())?)),
        }
    }
}

impl Display for Mtree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::V1(paths) | Self::V2(paths) => {
                    paths.iter().fold(String::new(), |mut output, path| {
                        let _ = write!(output, "{path:?}");
                        output
                    })
                }
            },
        )
    }
}

impl FromStr for Mtree {
    type Err = Error;

    /// Creates a [`Mtree`] from string slice `s`.
    ///
    /// Calls [`Mtree::from_str_with_schema`] with `schema` set to [`None`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - a [`MtreeSchema`] cannot be derived from `s`,
    /// - or the detected variant of [`Mtree`] cannot be constructed from `s`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_with_schema(s, None)
    }
}
