use std::{path::PathBuf, string::FromUtf8Error};

use fluent_i18n::t;

/// The Error that can occur when working with PKGINFO files.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// ALPM type error
    #[error(transparent)]
    AlpmType(#[from] alpm_types::Error),

    /// IO path error
    #[error("{msg}", msg = t!("error-io-path", {
        "path" => path,
        "context" => context,
        "source" => source.to_string()
    }))]
    IoPathError {
        /// The path where the error occurred.
        ///
        /// This is meant to complete the sentence "I/O error while  ".
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
    #[error("{msg}", msg = t!("error-deserialize", { "source" => source.to_string() }))]
    DeserializeError {
        /// The deserialization error source.
        #[from]
        source: alpm_parsers::custom_ini::Error,
    },

    /// An extra data field specified without any value.
    #[error("{msg}", msg = t!("error-extra-data-empty"))]
    ExtraDataEmpty,

    /// The first extra data field does not specify "pkgtype".
    #[error("{msg}", msg = t!("error-first-extra-data-not-pkgtype"))]
    FirstExtraDataNotPkgType,

    /// An invalid enum variant
    #[error("{msg}", msg = t!("error-invalid-variant", { "error" => 0.to_string() }))]
    InvalidVariant(#[from] strum::ParseError),

    /// Unsupported schema version.
    #[error("{msg}", msg = t!("error-unsupported-schema", { "version" => .0 }))]
    UnsupportedSchemaVersion(String),
}
