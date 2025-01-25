use std::{io::Read, path::PathBuf};

use alpm_types::{Checksum, Digest, Md5Checksum, Sha256Checksum};
use flate2::read::GzDecoder;
use serde::{ser::Error as SerdeError, Serialize, Serializer}; // codespell:ignore ser
use winnow::Parser;

pub use crate::parser::PathType;
use crate::{
    parser::{self, SetProperty, UnsetProperty},
    Error,
};

/// Represents a `/set` line in an MTREE file.
///
/// This struct also internally serves as the representation of default values
/// that're passed to all following path type lines.
#[derive(Debug, Clone, Default)]
pub struct PathDefaults {
    uid: Option<usize>,
    gid: Option<usize>,
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

/// A directory type path statement in an mtree file.
#[derive(Debug, Clone, Serialize)]
pub struct Directory {
    path: PathBuf,
    uid: usize,
    gid: usize,
    mode: String,
    time: usize,
}

/// A file type path statement in an mtree file.
///
/// The md5_digest is accepted for backwards compatibility reasons in v2 as well.
#[derive(Debug, Clone, Serialize)]
pub struct File {
    path: PathBuf,
    uid: usize,
    gid: usize,
    mode: String,
    size: usize,
    time: usize,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_optional_checksum_as_hex"
    )]
    md5_digest: Option<Md5Checksum>,
    #[serde(serialize_with = "serialize_checksum_as_hex")]
    sha256_digest: Sha256Checksum,
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
#[derive(Debug, Clone, Serialize)]
pub struct Link {
    path: PathBuf,
    uid: usize,
    gid: usize,
    mode: String,
    time: usize,
    link_path: PathBuf,
}

/// Represents the three possible types inside a path type line of an MTREE file.
///
/// While serializing, the type is converted into a `type` field on the inner struct.
/// This means that `Vec<Path>` will be serialized to a list of maps where each map has a `type`
/// entry with the respective name.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Path {
    #[serde(rename = "dir")]
    Directory(Directory),
    #[serde(rename = "file")]
    File(File),
    #[serde(rename = "link")]
    Link(Link),
}

/// Two magic bytes that occur at the beginning of gzip files and can be used to detect whether a
/// file is gzip compressed.
const GZIP_MAGIC_NUMBER: [u8; 2] = [0x1f, 0x8b];

/// Parse the raw byte content of an MTREE v2 file.
///
/// This is a thin wrapper around [`parse_mtree_v2`] that also takes care about potential GZIP
/// compression if the file has been compressed.
pub fn parse_raw_mtree_v2(buffer: Vec<u8>) -> Result<Vec<Path>, Error> {
    // Check if the file starts with `0x1f8b`, which is the magic number that marks files
    // as gzip compressed. If that's the case, decompress the content first.
    let contents = if buffer.len() >= 2 && [buffer[0], buffer[1]] == GZIP_MAGIC_NUMBER {
        let mut decoder = GzDecoder::new(buffer.as_slice());

        let mut content = String::new();
        decoder
            .read_to_string(&mut content)
            .map_err(Error::InvalidGzip)?;
        content
    } else {
        String::from_utf8(buffer)?.to_string()
    };

    parse_mtree_v2(contents)
}

/// Parse the content of an MTREE v2 file.
///
/// This parser is backwards compatible to `v1`, in the sense that it allows `md5` checksums, but
/// doesn't require them.
///
/// # Example
///
/// ```
/// use alpm_mtree::mtree_v2::parse_mtree_v2;
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

/// Take unsanitized parsed content and convert it to a list of paths with properties.
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
        unreachable!("Failed to read {line_nr} while handling an error. This should not happen, please report it as an issue.");
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
    let mut uid: Option<usize> = defaults.uid;
    let mut gid: Option<usize> = defaults.gid;
    let mut mode: Option<String> = defaults.mode.clone();
    let mut path_type: Option<PathType> = defaults.path_type;

    let mut link: Option<PathBuf> = None;
    let mut size: Option<usize> = None;
    let mut md5_digest: Option<Md5Checksum> = None;
    let mut sha256_digest: Option<Sha256Checksum> = None;
    let mut time: Option<usize> = None;

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
