#![allow(dead_code)]

use std::path::PathBuf;

use alpm_types::{Md5Checksum, Sha256Checksum};
use winnow::Parser;

pub use crate::parser::PathType;
use crate::{
    error::Error,
    parser::{self, SetProperty, UnsetProperty},
};

// Represents a `/set` line in an mtree file.
//
// This struct also internally serves as the representation of default values
// that're passed to all following path type lines.
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

#[derive(Debug, Clone)]
pub struct Directory {
    path: PathBuf,
    uid: usize,
    gid: usize,
    mode: String,
    time: usize,
}

/// A link that's contained inside the
#[derive(Debug, Clone)]
pub struct File {
    path: PathBuf,
    uid: usize,
    gid: usize,
    mode: String,
    size: usize,
    time: usize,
    md5_digest: Option<Md5Checksum>,
    sha256_digest: Sha256Checksum,
}

/// A link that points to a file somewhere on the system.
#[derive(Debug, Clone)]
pub struct Link {
    path: PathBuf,
    uid: usize,
    gid: usize,
    mode: String,
    time: usize,
    link_path: PathBuf,
}

/// Represents the three possible types inside a path type line of an mtree file.
#[derive(Debug, Clone)]
pub enum Path {
    #[serde(rename = "dir")]
    Directory(Directory),
    #[serde(rename = "file")]
    File(File),
    #[serde(rename = "link")]
    Link(Link),
}

pub fn parse_mtree_v2(content: String) -> Result<Vec<Path>, Error> {
    let parsed_contents = parser::mtree
        .parse(&content)
        .map_err(|err| Error::ParseError(format!("{err}")))?;

    paths_from_parsed_content(&content, parsed_contents)
}

/// Go through the unsanitized parsed content and convert it into a list of paths with their
/// respective properties.
///
/// This is effectively the interpreter step for mtree's declaration language.
fn paths_from_parsed_content(
    content: &str,
    parsed_content: Vec<parser::Statement>,
) -> Result<Vec<Path>, Error> {
    let mut paths = Vec::new();
    // Use an instance `Set` to track the current default properties for paths.
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
                // Incorporate a new set instruction into the current set of defaults.
                path_defaults.apply_set(properties);
            }
            parser::Statement::Unset(properties) => {
                // Incorporate a new unset instruction into the current set of defaults.
                path_defaults.apply_unset(properties);
            }
        }
    }

    Ok(paths)
}

/// Return the nth line of a given file's content.
///
/// This should not be able to error, as we already parsed the file and thereby know that the
/// expected number of lines exists. However, just to be sure, we do some error handling anyway.
fn content_line(content: &str, line_nr: usize) -> String {
    let line = content.lines().nth(line_nr);
    let Some(line) = line else {
        unreachable!("Failed to read {line_nr} while handling error. This bug should be unreachable, please report.");
    };

    line.to_string()
}

/// Take any given property and ensure that it's set.
///
/// If it isn't set, return a error message with the failing line as context.
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

/// Create the actual final mtree path representation from the parsed input.
///
/// This is the core of the mtree interpreter logic and does a few critical things:
/// - Incorporate default properties specified by previous `/set` and `/unset` statements.
/// - Ensure all paths have all necessary properties for the given path type.
///   - In case some property is missing, return a meaningful error message.
///
/// The way this works is as follows:
/// Go through all given properties and collect them in local `Option<T>` variables.
/// Afterwards, look at the `path_type` and ensure that all necessary properties for the given
/// path type are set. If any property is missing, throw an error.
/// If all properties are there, initialize the respective [Path] type and return it.
fn path_from_parsed(
    // The original content, passed through to create proper error messages.
    content: &str,
    // Passed in to create proper error messages.
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
