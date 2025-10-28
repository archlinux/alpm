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
    IoPathError {
        /// The path where the error occurred.
        path: PathBuf,
        /// The context in which the error occurred.
        context: String,
        /// The error source.
        source: std::io::Error,
    },

    /// I/O error while reading a buffer.
    #[error("{msg}", msg = t!("error-io-read", {
        "context" => context,
        "source" => source.to_string()
    }))]
    IoReadError {
        /// The context in which the error occurred.
        context: String,
        /// The error source.
        source: std::io::Error,
    },

    /// UTF-8 parse error.
    #[error(transparent)]
    InvalidUTF8(#[from] FromUtf8Error),

    /// An [`alpm_parsers::custom_ini::Error`].
    #[error("{msg}", msg = t!("error-deserialize-buildinfo", { "source" => .0.to_string() }))]
    DeserializeError(#[from] alpm_parsers::custom_ini::Error),

    /// No input file given.
    #[error("{msg}", msg = t!("error-no-input-file"))]
    NoInputFile,

    /// Unsupported schema version.
    #[error("{msg}", msg = t!("error-unsupported-schema", { "version" => .0 }))]
    UnsupportedSchemaVersion(String),

    /// A SchemaVersion with the wrong version is used.
    #[error("{msg}", msg = t!("error-wrong-schema-version", { "version" => .0.to_string() }))]
    WrongSchemaVersion(SchemaVersion),

    /// BuildInfo file is missing the format field.
    #[error("{msg}", msg = t!("error-missing-format-field"))]
    MissingFormatField,

    /// JSON error.
    #[error("{msg}", msg = t!("error-json", { "source" => .0.to_string() }))]
    Json(#[from] serde_json::Error),
}
