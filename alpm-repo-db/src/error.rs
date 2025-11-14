//! Error handling.

use std::path::PathBuf;

use crate::desc::SectionKeyword;

/// The error that can occur when working with the [`alpm-repo-desc`] files.
///
/// [`alpm-repo-desc`]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// IO error.
    #[error("I/O error while {0}:\n{1}")]
    Io(&'static str, std::io::Error),

    /// An I/O error occurred at a path.
    #[error("I/O error at {path} while {context}:\n{source}")]
    IoPathError {
        /// The path at which the error occurred.
        path: PathBuf,
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O error at path while ".
        context: &'static str,
        /// The source error.
        source: std::io::Error,
    },

    /// I/O error while reading a buffer.
    #[error("Read error while {context}:\n{source}")]
    IoReadError {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "Read error while ".
        context: &'static str,
        /// The error source.
        source: std::io::Error,
    },

    /// A winnow parser for a type didn't work and produced an error.
    #[error("Parser failed with the following error:\n{0}")]
    ParseError(String),

    /// A section is missing in the parsed data.
    #[error("Missing section: %{0}%")]
    MissingSection(SectionKeyword),

    /// A section is duplicated in the parsed data.
    #[error("Duplicate section: %{0}%")]
    DuplicateSection(SectionKeyword),

    /// A section is invalid for the given schema version.
    #[error("Section %{section}% is invalid for the schema version {version}.")]
    InvalidSectionForVersion {
        /// The section keyword.
        section: SectionKeyword,
        /// The schema version.
        version: u8,
    },

    /// Found an empty section that should either contain value(s) or be omitted.
    #[error("Unexpected empty section: %{0}%")]
    EmptySection(SectionKeyword),

    /// No input file given.
    #[error("No input file given.")]
    NoInputFile,

    #[cfg(feature = "cli")]
    /// JSON error.
    #[error("JSON error while {context}:\n{source}")]
    Json {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "JSON error while ".
        context: &'static str,
        /// The error source.
        source: serde_json::Error,
    },
    
    /// Unsupported schema version.
    #[error("Unsupported schema version: {0}")]
    UnsupportedSchemaVersion(String),

    /// Failed to parse v1 or v2.
    #[error("Failed to parse v1 or v2 format")]
    InvalidFormat,
}

impl<'a> From<winnow::error::ParseError<&'a str, winnow::error::ContextError>> for Error {
    /// Converts a [`winnow::error::ParseError`] into an [`Error::ParseError`].
    fn from(value: winnow::error::ParseError<&'a str, winnow::error::ContextError>) -> Self {
        Self::ParseError(value.to_string())
    }
}
