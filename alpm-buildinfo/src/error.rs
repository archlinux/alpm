use std::{path::PathBuf, string::FromUtf8Error};

use alpm_types::SchemaVersion;
use fluent_i18n::t;

/// The Error that can occur when working with BUILDINFO files.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// ALPM type error.
    #[error("{msg}", msg = t!("error-alpm-type", { "source" => .0.to_string() }))]
    AlpmType(#[from] alpm_types::Error),

    /// IO error.
    #[error("{msg}", msg = t!("error-io-path", {
        "path" => path.display().to_string(),
        "context" => context,
        "source" => source.to_string()
    }))]
    IoPath {
        /// The path where the error occurred.
        path: PathBuf,
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O error at path ".
        context: String,
        /// The error source.
        source: std::io::Error,
    },

    /// I/O error while reading a buffer.
    #[error("{msg}", msg = t!("error-io-read", {
        "context" => context,
        "source" => source.to_string()
    }))]
    IoRead {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "Read error while ".
        context: String,
        /// The error source.
        source: std::io::Error,
    },

    /// UTF-8 parse error.
    #[error(transparent)]
    InvalidUTF8(#[from] FromUtf8Error),

    /// An [`alpm_parsers::custom_ini::Error`].
    #[error("{msg}", msg = t!("error-deserialize-buildinfo", { "source" => .0.to_string() }))]
    Deserialization(#[from] alpm_parsers::custom_ini::Error),

    /// Unsupported schema version
    #[error("Unsupported schema version: {0}")]
    UnsupportedSchemaVersion(String),

    /// A SchemaVersion with the wrong version is used.
    #[error("{msg}", msg = t!("error-wrong-schema-version", { "version" => .0.to_string() }))]
    WrongSchemaVersion(SchemaVersion),

    /// BuildInfo file is missing the format field.
    #[error("{msg}", msg = t!("error-missing-format-field"))]
    MissingFormatField,
}
