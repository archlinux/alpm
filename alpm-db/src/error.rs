//! Error handling.

use std::path::PathBuf;

use alpm_types::Name;
use fluent_i18n::t;

use crate::{db::DatabaseEntryName, desc::SectionKeyword};

/// The error that can occur when working with the ALPM database desc files.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// An [`alpm_types::Error`].
    #[error("{msg}", msg = t!("error-alpm-types", { "source" => .0.to_string() }))]
    AlpmTypes(#[from] alpm_types::Error),

    /// An [`alpm_common::Error`].
    #[error(transparent)]
    AlpmCommon(#[from] alpm_common::Error),

    /// An [`crate::files::Error`].
    #[error(transparent)]
    AlpmDbFiles(#[from] crate::files::Error),

    /// An [`alpm_mtree::Error`].
    #[error(transparent)]
    AlpmMtree(#[from] alpm_mtree::Error),

    /// IO error.
    #[error("{msg}", msg = t!("error-io", { "context" => context, "source" => source.to_string() }))]
    Io {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O error while ".
        context: String,
        /// The source error.
        source: std::io::Error,
    },

    /// An I/O error occurred at a path.
    #[error("{msg}", msg = t!("error-io-path", {
        "path" => path.display().to_string(),
        "context" => context,
        "source" => source.to_string(),
    }))]
    IoPathError {
        /// The path at which the error occurred.
        path: PathBuf,
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O error at path while ".
        context: String,
        /// The source error.
        source: std::io::Error,
    },

    /// I/O error while reading a buffer.
    #[error("{msg}", msg = t!("error-io-read", { "context" => context, "source" => source.to_string() }))]
    IoReadError {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "Read error while ".
        context: String,
        /// The error source.
        source: std::io::Error,
    },

    /// A winnow parser for a type didn't work and produced an error.
    #[error("{msg}", msg = t!("error-parse", { "error" => .0 }))]
    ParseError(String),

    /// A section is missing in the parsed data.
    #[error("{msg}", msg = t!("error-missing-section", { "section" => .0.to_string() }))]
    MissingSection(SectionKeyword),

    /// A section is duplicated in the parsed data.
    #[error("{msg}", msg = t!("error-duplicate-section", { "section" => .0.to_string() }))]
    DuplicateSection(SectionKeyword),

    /// Invalid file encountered.
    #[error("{msg}", msg = t!("error-invalid-file", { "path" => path.display().to_string(), "context" => context }))]
    InvalidFile {
        /// The path of the invalid file.
        path: PathBuf,
        /// The context in which the error occurred.
        context: String,
    },

    /// Invalid file name encountered.
    #[error("{msg}", msg = t!("error-invalid-file-name", { "path" => path.display().to_string(), "context" => context }))]
    InvalidFileName {
        /// The path of the invalid file.
        path: PathBuf,
        /// The context in which the error occurred.
        context: String,
    },

    /// No input file given.
    #[error("{msg}", msg = t!("error-no-input-file"))]
    NoInputFile,

    #[cfg(feature = "cli")]
    /// JSON error.
    #[error("{msg}", msg = t!("error-json", { "context" => context, "source" => source.to_string() }))]
    Json {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "JSON error while ".
        context: String,
        /// The error source.
        source: serde_json::Error,
    },

    /// Unsupported schema version.
    #[error("{msg}", msg = t!("error-unsupported-schema-version", { "version" => .0 }))]
    UnsupportedSchemaVersion(String),

    /// Failed to parse v1 or v2.
    #[error("{msg}", msg = t!("error-invalid-format"))]
    InvalidFormat,

    /// Database entry already exists.
    #[error("{msg}", msg = t!("error-database-entry-already-exists", { "name" => name.to_string() }))]
    DatabaseEntryAlreadyExists {
        /// The name of the database entry that already exists.
        name: DatabaseEntryName,
    },

    /// Database entry name does not match the desc metadata.
    #[error("{msg}", msg = t!("error-database-entry-name-mismatch", {
        "path" => .0.path,
        "has_path" => .0.path.is_some().to_string(),
        "entry_name" => .0.entry_name.to_string(),
        "desc_name" => .0.desc_name.to_string(),
        "desc_version" => .0.desc_version.to_string(),
    }))]
    DatabaseEntryNameMismatch(Box<DatabaseEntryNameMismatch>),

    /// Duplicate database entry names exist.
    #[error("{msg}", msg = t!("error-database-entry-duplicate-name", {
        "name" => .0.name.to_string(),
        "entries" => .0.entries.iter().map(ToString::to_string).collect::<Vec<_>>().join(", "),
    }))]
    DatabaseEntryDuplicateName(Box<DatabaseEntryDuplicateName>),
}

/// Details for database entry name mismatch errors.
#[derive(Debug)]
pub struct DatabaseEntryNameMismatch {
    /// The optional path of the entry directory.
    pub path: Option<PathBuf>,
    /// The name of the database entry.
    pub entry_name: DatabaseEntryName,
    /// The name from the desc metadata.
    pub desc_name: alpm_types::Name,
    /// The version from the desc metadata.
    pub desc_version: alpm_types::FullVersion,
}

/// Details for duplicate database entry name errors.
#[derive(Debug)]
pub struct DatabaseEntryDuplicateName {
    /// The package name that has duplicate entries.
    pub name: Name,
    /// The duplicate entries for the package name.
    pub entries: Vec<DatabaseEntryName>,
}

impl<'a> From<winnow::error::ParseError<&'a str, winnow::error::ContextError>> for Error {
    /// Converts a [`winnow::error::ParseError`] into an [`Error::ParseError`].
    fn from(value: winnow::error::ParseError<&'a str, winnow::error::ContextError>) -> Self {
        Self::ParseError(value.to_string())
    }
}
