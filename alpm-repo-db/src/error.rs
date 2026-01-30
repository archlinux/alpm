//! Error handling.

use std::path::PathBuf;

use fluent_i18n::t;

use crate::desc::SectionKeyword;

/// The error that can occur when working with the [`alpm-repo-desc`] files.
///
/// [`alpm-repo-desc`]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
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
    IoPath {
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
    IoRead {
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

    /// A section is invalid for the given schema version.
    #[error("{msg}", msg = t!("error-invalid-section-for-version", { "section" => section.to_string(), "version" => version.to_string() }))]
    InvalidSectionForVersion {
        /// The section keyword.
        section: SectionKeyword,
        /// The schema version.
        version: u8,
    },

    /// Found an empty section that should either contain value(s) or be omitted.
    #[error("{msg}", msg = t!("error-empty-section", { "section" => .0.to_string() }))]
    EmptySection(SectionKeyword),

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
}

impl<'a> From<winnow::error::ParseError<&'a str, winnow::error::ContextError>> for Error {
    /// Converts a [`winnow::error::ParseError`] into an [`Error::ParseError`].
    fn from(value: winnow::error::ParseError<&'a str, winnow::error::ContextError>) -> Self {
        Self::ParseError(value.to_string())
    }
}
