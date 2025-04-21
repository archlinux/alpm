//! Handling for the ALPM-MTREE file format.

pub mod path_validation_error;
pub mod v2;
use std::{
    collections::HashSet,
    fmt::{Display, Write},
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_common::{FileFormatSchema, InputPath, InputPaths, MetadataFile};
use path_validation_error::{PathValidationError, PathValidationErrors};
#[cfg(doc)]
use v2::MTREE_PATH_PREFIX;

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

impl Mtree {
    /// Validates an [`InputPaths`].
    ///
    /// With `input_paths`a set of relative paths and a common base directory is provided.
    ///
    /// Each member of [`InputPaths::paths`] is compared with the data available in `self` by
    /// retrieving metadata from the on-disk files below [`InputPaths::base_dir`].
    /// For this, [`MTREE_PATH_PREFIX`] is stripped from each [`Path`][`crate::mtree::v2::Path`]
    /// tracked by the [`Mtree`] and afterwards each [`Path`][`crate::mtree::v2::Path`] is
    /// compared with the respective file in [`InputPaths::base_dir`].
    /// This includes checking if
    ///
    /// - each relative path in [`InputPaths::paths`] matches a record in the [ALPM-MTREE] data,
    /// - each relative path in [`InputPaths::paths`] relates to an existing file, directory or
    ///   symlink in [`InputPaths::base_dir`],
    /// - the target of each symlink in the [ALPM-MTREE] data matches that of the corresponding
    ///   on-disk file,
    /// - size and SHA-256 hash digest of each file in the [ALPM-MTREE] data matches that of the
    ///   corresponding on-disk file,
    /// - the [ALPM-MTREE] data file itself is included in the [ALPM-MTREE] data,
    /// - and the creation time, UID, GID and file mode of each file in the [ALPM-MTREE] data
    ///   matches that of the corresponding on-disk file.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - [`InputPaths::paths`] contains duplicates,
    /// - or one of the [ALPM-MTREE] data entries
    ///   - does not have a matching on-disk file, directory or symlink (depending on type),
    ///   - has a mismatching symlink target from that of a corresponding on-disk file,
    ///   - has a mismatching size or SHA-256 hash digest from that of a corresponding on-disk file,
    ///   - is the [ALPM-MTREE] file,
    ///   - or has a mismatching creation time, UID, GID or file mode from that of a corresponding
    ///     on-disk file,
    /// - or one of the file system paths in [`InputPaths::paths`] has no matching [ALPM-MTREE]
    ///   entry.
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    pub fn validate_paths(&self, input_paths: &InputPaths) -> Result<(), Error> {
        let base_dir = input_paths.base_dir();
        // Use paths in a HashSet for easier handling later.
        let mut hashed_paths = HashSet::new();
        let mut duplicates = HashSet::new();
        for path in input_paths.paths() {
            if hashed_paths.contains(path.as_path()) {
                duplicates.insert(path.to_path_buf());
            }
            hashed_paths.insert(path.as_path());
        }
        // If there are duplicate paths, return early.
        if !duplicates.is_empty() {
            return Err(Error::DuplicatePaths { paths: duplicates });
        }

        let mtree_paths = match self {
            Mtree::V1(mtree) | Mtree::V2(mtree) => mtree,
        };
        let mut errors = PathValidationErrors::new(base_dir.to_path_buf());
        let mut unmatched_paths = Vec::new();

        for mtree_path in mtree_paths.iter() {
            // Normalize the ALPM-MTREE path.
            let normalized_path = match mtree_path.as_normalized_path() {
                Ok(mtree_path) => mtree_path,
                Err(source) => {
                    let mut normalize_errors: Vec<PathValidationError> = vec![source.into()];
                    errors.append(&mut normalize_errors);
                    // Continue, as the ALPM-MTREE data is not as it should be.
                    continue;
                }
            };

            // If the normalized path exists in the hashed input paths, compare.
            if hashed_paths.remove(normalized_path) {
                if let Err(mut comparison_errors) =
                    mtree_path.equals_path(&InputPath::new(base_dir, normalized_path)?)
                {
                    errors.append(&mut comparison_errors);
                }
            } else {
                unmatched_paths.push(mtree_path);
            }
        }

        // Add dedicated error, if some file system paths are not covered by ALPM-MTREE data.
        if !hashed_paths.is_empty() {
            errors.append(&mut vec![PathValidationError::UnmatchedFileSystemPaths {
                paths: hashed_paths.iter().map(|path| path.to_path_buf()).collect(),
            }])
        }

        // Add dedicated error, if some ALPM-MTREE paths have no matching file system paths.
        if !unmatched_paths.is_empty() {
            errors.append(&mut vec![PathValidationError::UnmatchedMtreePaths {
                paths: unmatched_paths
                    .iter()
                    .map(|path| path.to_path_buf())
                    .collect(),
            }])
        }

        // Emit all error messages on stderr and fail if there are any errors.
        errors.check()?;

        Ok(())
    }
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
