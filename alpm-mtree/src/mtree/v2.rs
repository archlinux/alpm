//! Interpreter for ALPM-MTREE v1 and v2.

use std::{fs::Metadata, io::Read, os::linux::fs::MetadataExt, path::PathBuf};

use alpm_types::{Checksum, Digest, Md5Checksum, Sha256Checksum};
use log::trace;
use serde::{Serialize, Serializer, ser::Error as SerdeError}; // codespell:ignore ser
use winnow::Parser;

#[cfg(doc)]
use crate::Mtree;
pub use crate::parser::PathType;
use crate::{
    Error,
    mtree::path_validation_error::PathValidationError,
    parser::{self, SetProperty, UnsetProperty},
};

pub const MTREE_PATH_PREFIX: &str = "./";

/// Represents a `/set` line in an MTREE file.
///
/// This struct also internally serves as the representation of default values
/// that're passed to all following path type lines.
#[derive(Debug, Clone, Default)]
pub struct PathDefaults {
    uid: Option<u32>,
    gid: Option<u32>,
    mode: Option<String>,
    path_type: Option<PathType>,
}

impl PathDefaults {
    /// Apply a parsed `/set` statement's properties onto the current set of [PathDefaults].
    fn apply_set(&mut self, properties: Vec<SetProperty>) {
        for property in properties {
            match property {
                SetProperty::Uid(uid) => self.uid = Some(uid),
                SetProperty::Gid(gid) => self.gid = Some(gid),
                SetProperty::Mode(mode) => self.mode = Some(mode.to_string()),
                SetProperty::Type(path_type) => self.path_type = Some(path_type),
            }
        }
    }

    /// Apply a parsed `/unset` statement's properties onto the current set of [PathDefaults].
    fn apply_unset(&mut self, properties: Vec<UnsetProperty>) {
        for property in properties {
            match property {
                UnsetProperty::Uid => self.uid = None,
                UnsetProperty::Gid => self.gid = None,
                UnsetProperty::Mode => self.mode = None,
                UnsetProperty::Type => self.path_type = None,
            }
        }
    }
}

/// Validates common path features against relevant [`Mtree`] data.
///
/// Returns a list of zero or more [`PathValidationError`]s.
/// Checks that
///
/// - `mtree_time` matches the modification time available in `metadata`,
/// - `mtree_uid` matches the UID available in the `metadata`,
/// - `mtree_gid` matches the GID available in the `metadata`,
/// - and the mode available in `metadata` ends in `mtree_mode`.
fn validate_path_common(
    mtree_path: impl AsRef<std::path::Path>,
    mtree_time: i64,
    mtree_uid: u32,
    mtree_gid: u32,
    mtree_mode: &str,
    path: impl AsRef<std::path::Path>,
    metadata: &Metadata,
) -> Vec<PathValidationError> {
    let mtree_path = mtree_path.as_ref();
    let path = path.as_ref();
    let mut errors = Vec::new();

    // Ensure that the path modification time recorded in the ALPM-MTREE data matches the
    // on-disk file.
    if mtree_time != metadata.st_mtime() {
        errors.push(PathValidationError::PathTimeMismatch {
            mtree_path: mtree_path.to_path_buf(),
            mtree_time,
            path: path.to_path_buf(),
            path_time: metadata.st_mtime(),
        });
    }

    // Ensure that the path UID recorded in the ALPM-MTREE data matches the
    // on-disk file.
    if mtree_uid != metadata.st_uid() {
        errors.push(PathValidationError::PathUidMismatch {
            mtree_path: mtree_path.to_path_buf(),
            mtree_uid,
            path: path.to_path_buf(),
            path_uid: metadata.st_uid(),
        });
    }

    // Ensure that the path GID recorded in the ALPM-MTREE data matches the
    // on-disk file.
    if mtree_gid != metadata.st_gid() {
        errors.push(PathValidationError::PathGidMismatch {
            mtree_path: mtree_path.to_path_buf(),
            mtree_gid,
            path: path.to_path_buf(),
            path_gid: metadata.st_gid(),
        });
    }

    // Ensure that the path mode recorded in the ALPM-MTREE data matches the
    // on-disk file.
    let path_mode = format!("{:o}", metadata.st_mode());
    if !path_mode.ends_with(mtree_mode) {
        errors.push(PathValidationError::PathModeMismatch {
            mtree_path: mtree_path.to_path_buf(),
            mtree_mode: mtree_mode.to_string(),
            path: path.to_path_buf(),
            path_mode: path_mode.to_string(),
        });
    }

    errors
}

/// A directory type path statement in an mtree file.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Directory {
    pub path: PathBuf,
    pub uid: u32,
    pub gid: u32,
    pub mode: String,
    pub time: i64,
}

impl Directory {
    /// Checks whether `path` in `base_dir` equals self.
    ///
    /// More specifically, checks that
    ///
    /// - the default mtree path prefix can be stripped from `self.path`,
    /// - `path` and `self.path` match (after stripping),
    /// - `path` exists in `base_dir`,
    /// - metadata can be retrieved for `path` (in `base_dir`),
    /// - `path` (in `base_dir`) is a directory,
    /// - the modification time of `path` (in `base_dir`) matches that of `self.time`,
    /// - the UID of `path` (in `base_dir`) matches that of `self.uid`,
    /// - the GID of `path` (in `base_dir`) matches that of `self.gid`,
    /// - the mode of `path` (in `base_dir`) matches that of `self.mode`.
    ///
    /// # Errors
    ///
    /// Returns a list of [`PathValidationError`]s if issues have been found during validation of
    /// `path` in `base_dir`.
    pub fn equals_path(
        &self,
        base_dir: impl AsRef<std::path::Path>,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), Vec<PathValidationError>> {
        let base_dir = base_dir.as_ref();
        let path = path.as_ref();
        let mut errors = Vec::new();

        trace!(
            "Comparing ALPM-MTREE directory path {self:?} with path {path:?} below {base_dir:?}"
        );

        // Normalize the ALPM-MTREE path.
        let mtree_path = match self.path.as_path().strip_prefix(MTREE_PATH_PREFIX) {
            Ok(mtree_path) => mtree_path,
            Err(source) => {
                errors.push(
                    alpm_common::Error::PathStripPrefix {
                        prefix: PathBuf::from(MTREE_PATH_PREFIX),
                        path: self.path.to_path_buf(),
                        source,
                    }
                    .into(),
                );
                // Return early, as the ALPM-MTREE data is not as it should be.
                return Err(errors);
            }
        };

        // Ensure `self.path` and `path` match.
        if mtree_path != path {
            errors.push(PathValidationError::PathMismatch {
                mtree_path: self.path.clone(),
                path: path.to_path_buf(),
            });
            // Return early as the paths mismatch.
            return Err(errors);
        }

        let path = base_dir.join(path);

        // Ensure path exists.
        if !path.exists() {
            errors.push(PathValidationError::PathMissing {
                mtree_path: self.path.clone(),
                path: path.clone(),
            });
            // Return early, as there is no reason to continue doing file checks.
            return Err(errors);
        }

        // Retrieve metadata of directory.
        let metadata = match path.metadata() {
            Ok(metadata) => metadata,
            Err(source) => {
                errors.push(PathValidationError::PathMetadata {
                    path: path.clone(),
                    source,
                });
                // Return early, as the following checks are based on metadata.
                return Err(errors);
            }
        };

        // Ensure that the on-disk path is a directory.
        if !metadata.is_dir() {
            errors.push(PathValidationError::PathNotADir {
                mtree_path: mtree_path.to_path_buf(),
                path: path.to_path_buf(),
            });
            // Return early, because further checks are (mostly) based on whether this is a
            // directory.
            return Err(errors);
        }

        let mut common_errors = validate_path_common(
            mtree_path,
            self.time,
            self.uid,
            self.gid,
            &self.mode,
            path.as_path(),
            &metadata,
        );
        errors.append(&mut common_errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// A file type path statement in an mtree file.
///
/// The md5_digest is accepted for backwards compatibility reasons in v2 as well.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct File {
    pub path: PathBuf,
    pub uid: u32,
    pub gid: u32,
    pub mode: String,
    pub size: u64,
    pub time: i64,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_optional_checksum_as_hex"
    )]
    pub md5_digest: Option<Md5Checksum>,
    #[serde(serialize_with = "serialize_checksum_as_hex")]
    pub sha256_digest: Sha256Checksum,
}

impl File {
    /// Checks whether `path` in `base_dir` equals self.
    ///
    /// More specifically, checks that
    ///
    /// - the default mtree path prefix can be stripped from `self.path`,
    /// - `path` and `self.path` match (after stripping),
    /// - `path` exists in `base_dir`,
    /// - metadata can be retrieved for `path` (in `base_dir`),
    /// - `path` (in `base_dir`) is a file,
    /// - the size of `path` (in `base_dir`) matches that of `self.size`,
    /// - the SHA-256 hash digest of `path` (in `base_dir`) matches that of `self.sha256_digest`,
    /// - the modification time of `path` (in `base_dir`) matches that of `self.time`,
    /// - the UID of `path` (in `base_dir`) matches that of `self.uid`,
    /// - the GID of `path` (in `base_dir`) matches that of `self.gid`,
    /// - the mode of `path` (in `base_dir`) matches that of `self.mode`.
    ///
    /// # Errors
    ///
    /// Returns a list of [`PathValidationError`]s if issues have been found during validation of
    /// `path` in `base_dir`.
    pub fn equals_path(
        &self,
        base_dir: impl AsRef<std::path::Path>,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), Vec<PathValidationError>> {
        let base_dir = base_dir.as_ref();
        let path = path.as_ref();
        let mut errors = Vec::new();

        trace!("Comparing ALPM-MTREE file path {self:?} with path {path:?} below {base_dir:?}");

        // Normalize the ALPM-MTREE path.
        let mtree_path = match self.path.as_path().strip_prefix(MTREE_PATH_PREFIX) {
            Ok(mtree_path) => mtree_path,
            Err(source) => {
                errors.push(
                    alpm_common::Error::PathStripPrefix {
                        prefix: PathBuf::from(MTREE_PATH_PREFIX),
                        path: self.path.to_path_buf(),
                        source,
                    }
                    .into(),
                );
                // Return early, as the ALPM-MTREE data is not as it should be.
                return Err(errors);
            }
        };

        // Ensure `self.path` and `path` match.
        if mtree_path != path {
            errors.push(PathValidationError::PathMismatch {
                mtree_path: self.path.clone(),
                path: path.to_path_buf(),
            });
            // Return early as the paths mismatch.
            return Err(errors);
        }

        let path = base_dir.join(path);

        // Ensure path exists.
        if !path.exists() {
            errors.push(PathValidationError::PathMissing {
                mtree_path: self.path.clone(),
                path: path.clone(),
            });
            // Return early, as there is no reason to continue doing file checks.
            return Err(errors);
        }

        // Retrieve metadata of file.
        let metadata = match path.metadata() {
            Ok(metadata) => metadata,
            Err(source) => {
                errors.push(PathValidationError::PathMetadata {
                    path: path.clone(),
                    source,
                });
                // Return early, as the following checks are based on metadata.
                return Err(errors);
            }
        };

        // Ensure that the on-disk path is a file.
        if !metadata.is_file() {
            errors.push(PathValidationError::PathNotAFile {
                mtree_path: mtree_path.to_path_buf(),
                path: path.to_path_buf(),
            });
            // Return early, because further checks are (mostly) based on whether this is a file.
            return Err(errors);
        }

        // Create the hash digest.
        let path_digest = {
            let mut file = match std::fs::File::open(path.as_path()) {
                Ok(file) => file,
                Err(source) => {
                    errors.push(PathValidationError::CreateHashDigest {
                        path: path.to_path_buf(),
                        source,
                    });
                    // Return early, because not being able to open the file points at file system
                    // issues.
                    return Err(errors);
                }
            };

            let mut buf = Vec::new();
            match file.read_to_end(&mut buf) {
                Ok(_) => {}
                Err(source) => {
                    errors.push(PathValidationError::CreateHashDigest {
                        path: path.to_path_buf(),
                        source,
                    });
                    // Return early, because not being able to read the file points at file system
                    // issues.
                    return Err(errors);
                }
            }

            Sha256Checksum::calculate_from(buf)
        };

        // Compare the file size.
        if metadata.st_size() != self.size {
            errors.push(PathValidationError::PathSizeMismatch {
                mtree_path: self.path.clone(),
                mtree_size: self.size,
                path: path.to_path_buf(),
                path_size: metadata.st_size(),
            });
        }

        // Compare the hash digests.
        if self.sha256_digest != path_digest {
            errors.push(PathValidationError::PathDigestMismatch {
                mtree_path: mtree_path.to_path_buf(),
                mtree_digest: self.sha256_digest.clone(),
                path: path.to_path_buf(),
                path_digest,
            });
        }

        let mut common_errors = validate_path_common(
            mtree_path,
            self.time,
            self.uid,
            self.gid,
            &self.mode,
            path.as_path(),
            &metadata,
        );
        errors.append(&mut common_errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Serialize an `Option<Checksum<D>>` as a HexString.
///
/// # Errors
///
/// Returns an error if the `checksum` can not be serialized using the `serializer`.
fn serialize_checksum_as_hex<S, D>(checksum: &Checksum<D>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    D: Digest,
{
    let hex_string = checksum.to_string();
    serializer.serialize_str(&hex_string)
}

/// Serialize an `Option<Checksum<D>>`
///
/// Sadly this is needed in addition to the function above, even though we know that it won't be
/// called due to the `skip_serializing_if` check above.
fn serialize_optional_checksum_as_hex<S, D>(
    checksum: &Option<Checksum<D>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    D: Digest,
{
    let hex_string = checksum
        .as_ref()
        .ok_or_else(|| S::Error::custom("Empty checksums won't be serialized"))?
        .to_string();
    serializer.serialize_str(&hex_string)
}

/// A link type path in an mtree file that points to a file somewhere on the system.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Link {
    pub path: PathBuf,
    pub uid: u32,
    pub gid: u32,
    pub mode: String,
    pub time: i64,
    pub link_path: PathBuf,
}

impl Link {
    /// Checks whether `path` in `base_dir` equals self.
    ///
    /// More specifically, checks that
    ///
    /// - the default mtree path prefix can be stripped from `self.path`,
    /// - `path` and `self.path` match (after stripping),
    /// - `path` exists in `base_dir`,
    /// - metadata can be retrieved for `path` (in `base_dir`),
    /// - `path` (in `base_dir`) is a symlink,
    /// - the link path of `path` (in `base_dir`) matches that of `self.link_path`,
    /// - the modification time of `path` (in `base_dir`) matches that of `self.time`,
    /// - the UID of `path` (in `base_dir`) matches that of `self.uid`,
    /// - the GID of `path` (in `base_dir`) matches that of `self.gid`,
    /// - the mode of `path` (in `base_dir`) matches that of `self.mode`.
    ///
    /// # Errors
    ///
    /// Returns a list of [`PathValidationError`]s if issues have been found during validation of
    /// `path` in `base_dir`.
    pub fn equals_path(
        &self,
        base_dir: impl AsRef<std::path::Path>,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), Vec<PathValidationError>> {
        let base_dir = base_dir.as_ref();
        let path = path.as_ref();
        let mut errors = Vec::new();

        trace!("Comparing ALPM-MTREE symlink path {self:?} with path {path:?} below {base_dir:?}");

        // Normalize the ALPM-MTREE path.
        let mtree_path = match self.path.as_path().strip_prefix(MTREE_PATH_PREFIX) {
            Ok(mtree_path) => mtree_path,
            Err(source) => {
                errors.push(
                    alpm_common::Error::PathStripPrefix {
                        prefix: PathBuf::from(MTREE_PATH_PREFIX),
                        path: self.path.to_path_buf(),
                        source,
                    }
                    .into(),
                );
                // Return early, as the ALPM-MTREE data is not as it should be.
                return Err(errors);
            }
        };

        // Ensure `self.path` and `path` match.
        if mtree_path != path {
            errors.push(PathValidationError::PathMismatch {
                mtree_path: self.path.clone(),
                path: path.to_path_buf(),
            });
            // Return early as the paths mismatch.
            return Err(errors);
        }

        let path = base_dir.join(path);

        // Note: We don't check for path existence, because Path::exists would traverse symlinks.

        // Retrieve metadata of symlink.
        let metadata = match path.symlink_metadata() {
            Ok(metadata) => metadata,
            Err(source) => {
                errors.push(PathValidationError::PathMetadata {
                    path: path.clone(),
                    source,
                });
                // Return early, as the following checks are based on metadata.
                return Err(errors);
            }
        };

        // Ensure that the on-disk path is a symlink.
        if !metadata.is_symlink() {
            errors.push(PathValidationError::PathNotASymlink {
                mtree_path: mtree_path.to_path_buf(),
                path,
            });
            // Return early because the remaining checks assume that we have a symlink.
            return Err(errors);
        }

        // Get the target path of the symlink and ensure it matches.
        match path.read_link() {
            Ok(link_path) => {
                if self.link_path != link_path.as_path() {
                    errors.push(PathValidationError::PathSymlinkMismatch {
                        mtree_path: mtree_path.to_path_buf(),
                        mtree_link_path: self.link_path.clone(),
                        path: path.clone(),
                        link_path,
                    });
                }
            }
            Err(source) => {
                errors.push(PathValidationError::ReadLink {
                    path: path.clone(),
                    source,
                });
            }
        }

        let mut common_errors = validate_path_common(
            mtree_path,
            self.time,
            self.uid,
            self.gid,
            &self.mode,
            path.as_path(),
            &metadata,
        );
        errors.append(&mut common_errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Represents the three possible types inside a path type line of an MTREE file.
///
/// While serializing, the type is converted into a `type` field on the inner struct.
/// This means that `Vec<Path>` will be serialized to a list of maps where each map has a `type`
/// entry with the respective name.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum Path {
    #[serde(rename = "dir")]
    Directory(Directory),
    #[serde(rename = "file")]
    File(File),
    #[serde(rename = "link")]
    Link(Link),
}

impl Path {
    /// Checks whether `path` in `base_dir` equals self.
    ///
    /// Depending on type of [`Path`], delegates to [`Directory::equals_path`],
    /// [`File::equals_path`] or [`Link::equals_path`].
    ///
    /// # Errors
    ///
    /// Returns a list of [`PathValidationError`]s if issues have been found during validation of
    /// `path` in `base_dir`.
    pub fn equals_path(
        &self,
        base_dir: impl AsRef<std::path::Path>,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), Vec<PathValidationError>> {
        match self {
            Self::Directory(directory) => directory.equals_path(base_dir, path),
            Self::File(file) => file.equals_path(base_dir, path),
            Self::Link(link) => link.equals_path(base_dir, path),
        }
    }

    /// Returns the [`PathBuf`] of the [`Path`].
    pub fn to_path_buf(&self) -> PathBuf {
        match self {
            Self::Directory(directory) => directory.path.clone(),
            Self::File(file) => file.path.clone(),
            Self::Link(link) => link.path.clone(),
        }
    }

    /// Returns the [`std::path::Path`] of the [`Path`].
    pub fn as_path(&self) -> &std::path::Path {
        match self {
            Self::Directory(directory) => directory.path.as_path(),
            Self::File(file) => file.path.as_path(),
            Self::Link(link) => link.path.as_path(),
        }
    }

    /// Returns the normalized [`std::path::Path`] of the [`Path`].
    ///
    /// Normalization strips the prefix [`MTREE_PATH_PREFIX`].
    ///
    /// # Errors
    ///
    /// Returns an [`alpm_common::Error`] if the prefix can not be stripped.
    pub fn as_normalized_path(&self) -> Result<&std::path::Path, alpm_common::Error> {
        self.as_path()
            .strip_prefix(MTREE_PATH_PREFIX)
            .map_err(|source| alpm_common::Error::PathStripPrefix {
                prefix: PathBuf::from(MTREE_PATH_PREFIX),
                path: self.to_path_buf(),
                source,
            })
    }
}

impl Ord for Path {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let path = match self {
            Path::Directory(dir) => dir.path.as_path(),
            Path::File(file) => file.path.as_path(),
            Path::Link(link) => link.path.as_path(),
        };
        let other_path = match other {
            Path::Directory(dir) => dir.path.as_path(),
            Path::File(file) => file.path.as_path(),
            Path::Link(link) => link.path.as_path(),
        };
        path.cmp(other_path)
    }
}

impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Parse the content of an MTREE v2 file.
///
/// This parser is backwards compatible to `v1`, in the sense that it allows `md5` checksums, but
/// doesn't require them.
///
/// # Example
///
/// ```
/// use alpm_mtree::mtree::v2::parse_mtree_v2;
///
/// # fn main() -> Result<(), alpm_mtree::Error> {
/// let content = r#"
/// /set uid=0 gid=0 mode=644 type=link
/// ./some_link link=/etc time=1706086640.0
/// "#;
/// let paths = parse_mtree_v2(content.to_string())?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// - `Error::ParseError` if a malformed MTREE file is encountered.
/// - `Error::InterpreterError` if there's missing fields or logical error in the parsed contents of
///   the MTREE file.
pub fn parse_mtree_v2(content: String) -> Result<Vec<Path>, Error> {
    let parsed_contents = parser::mtree
        .parse(&content)
        .map_err(|err| Error::ParseError(format!("{err}")))?;

    paths_from_parsed_content(&content, parsed_contents)
}

/// Take unsanitized parsed content and convert it to a list of sorted paths with properties.
///
/// This is effectively the interpreter step for mtree's declaration language.
fn paths_from_parsed_content(
    content: &str,
    parsed_content: Vec<parser::Statement>,
) -> Result<Vec<Path>, Error> {
    let mut paths = Vec::new();
    // Track the current default properties for paths.
    let mut path_defaults = PathDefaults::default();

    for (line_nr, parsed) in parsed_content.into_iter().enumerate() {
        match parsed {
            parser::Statement::Ignored => continue,
            parser::Statement::Path { path, properties } => {
                // Create a [Path] instance from a given path statement.
                // Pass the content and line-nr through.
                // The line nr is incremented by one due to `#mtree` being the first line.
                let path = path_from_parsed(content, line_nr, &path_defaults, path, properties)?;
                paths.push(path);
            }
            parser::Statement::Set(properties) => {
                // Incorporate a new `/set` command into the current set of defaults.
                path_defaults.apply_set(properties);
            }
            parser::Statement::Unset(properties) => {
                // Incorporate a new `/unset` command into the current set of defaults.
                path_defaults.apply_unset(properties);
            }
        }
    }

    // Sort paths to ensure that ALPM-MTREE paths can be compared to file system paths.
    // Paths in a package file, as well as the input to `bsdtar` when creating an ALPM-MTREE file
    // are also sorted.
    // Without this the reproducibility of the data can not be guaranteed.
    paths.sort_unstable();

    Ok(paths)
}

/// Return the nth line of a given file's content.
///
/// # Panics
///
/// Panics if `line_nr` refers to a line, that does not exist in `content`.
/// This is unlikely to ever happen, as the `content` is derived from a parsed file and therefore it
/// is known that the specific line referenced by `line_nr` exists.
fn content_line(content: &str, line_nr: usize) -> String {
    let line = content.lines().nth(line_nr);
    let Some(line) = line else {
        unreachable!(
            "Failed to read {line_nr} while handling an error. This should not happen, please report it as an issue."
        );
    };

    line.to_string()
}

/// Take any given property and ensure that it's set.
///
/// # Errors
///
/// - `Error::InterpreterError` if the expected property is `None`.
fn ensure_property<T>(
    content: &str,
    line_nr: usize,
    property: Option<T>,
    property_name: &str,
) -> Result<T, Error> {
    // Ensure that we know the type of the given path.
    let Some(property) = property else {
        return Err(Error::InterpreterError(
            line_nr,
            content_line(content, line_nr),
            format!("Couldn't find property {property_name} for path."),
        ));
    };

    Ok(property)
}

/// Create the actual final MTREE path representation from the parsed input.
///
/// This is the core of the mtree interpreter logic and does a few critical things:
/// - Incorporate default properties specified by previous `/set` and `/unset` statements.
/// - Ensure all paths have all necessary properties for the given path type.
///
/// The way this works is as follows:
/// Go through all given properties and collect them in local `Option<T>` variables.
/// Afterwards, look at the `path_type` and ensure that all necessary properties for the given
/// path type are set.
/// If all properties are there, initialize the respective [Path] type and return it.
///
/// The original content (`content`), as well as line number (`line_nr`) are passed in as well to
/// provide detailed error messages.
///
/// # Errors
///
/// - `Error::InterpreterError` if expected properties for a given type aren't set.
fn path_from_parsed(
    content: &str,
    line_nr: usize,
    defaults: &PathDefaults,
    path: PathBuf,
    properties: Vec<parser::PathProperty>,
) -> Result<Path, Error> {
    // Copy any possible default values over.
    let mut uid: Option<u32> = defaults.uid;
    let mut gid: Option<u32> = defaults.gid;
    let mut mode: Option<String> = defaults.mode.clone();
    let mut path_type: Option<PathType> = defaults.path_type;

    let mut link: Option<PathBuf> = None;
    let mut size: Option<u64> = None;
    let mut md5_digest: Option<Md5Checksum> = None;
    let mut sha256_digest: Option<Sha256Checksum> = None;
    let mut time: Option<i64> = None;

    // Read all properties and set them accordingly.
    for property in properties {
        match property {
            parser::PathProperty::Uid(inner) => uid = Some(inner),
            parser::PathProperty::Gid(inner) => gid = Some(inner),
            parser::PathProperty::Mode(inner) => mode = Some(inner.to_string()),
            parser::PathProperty::Type(inner) => path_type = Some(inner),
            parser::PathProperty::Size(inner) => size = Some(inner),
            parser::PathProperty::Link(inner) => link = Some(inner),
            parser::PathProperty::Md5Digest(checksum) => md5_digest = Some(checksum),
            parser::PathProperty::Sha256Digest(checksum) => sha256_digest = Some(checksum),
            parser::PathProperty::Time(inner) => time = Some(inner),
        }
    }

    // Ensure that we know the type of the given path.
    let Some(path_type) = path_type else {
        return Err(Error::InterpreterError(
            line_nr,
            content_line(content, line_nr),
            "Found no type for path.".to_string(),
        ));
    };

    // Build the path based on the path type.
    // While doing so, ensure that all required properties are set.
    let path = match path_type {
        PathType::Dir => Path::Directory(Directory {
            path,
            uid: ensure_property(content, line_nr, uid, "uid")?,
            gid: ensure_property(content, line_nr, gid, "gid")?,
            mode: ensure_property(content, line_nr, mode, "mode")?,
            time: ensure_property(content, line_nr, time, "time")?,
        }),
        PathType::File => Path::File(File {
            path,
            uid: ensure_property(content, line_nr, uid, "uid")?,
            gid: ensure_property(content, line_nr, gid, "gid")?,
            mode: ensure_property(content, line_nr, mode, "mode")?,
            size: ensure_property(content, line_nr, size, "size")?,
            time: ensure_property(content, line_nr, time, "time")?,
            md5_digest,
            sha256_digest: ensure_property(content, line_nr, sha256_digest, "sha256_digest")?,
        }),
        PathType::Link => Path::Link(Link {
            path,
            uid: ensure_property(content, line_nr, uid, "uid")?,
            gid: ensure_property(content, line_nr, gid, "gid")?,
            mode: ensure_property(content, line_nr, mode, "mode")?,
            link_path: ensure_property(content, line_nr, link, "link")?,
            time: ensure_property(content, line_nr, time, "time")?,
        }),
    };

    Ok(path)
}
