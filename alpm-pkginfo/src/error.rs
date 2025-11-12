use std::{path::PathBuf, string::FromUtf8Error};

/// The Error that can occur when working with PKGINFO files
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// ALPM type error
    #[error("ALPM type parse error: {0}")]
    AlpmType(#[from] alpm_types::Error),

    /// IO error
    #[error("I/O error at path {0:?} while {1}:\n{2}")]
    IoPathError(PathBuf, &'static str, std::io::Error),

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

    /// UTF-8 parse error
    #[error(transparent)]
    InvalidUTF8(#[from] FromUtf8Error),

    /// An [`alpm_parsers::custom_ini::Error`].
    #[error("Failed to deserialize PKGINFO file:\n{0}")]
    DeserializeError(#[from] alpm_parsers::custom_ini::Error),

    /// An extra data field specified without any value.
    #[error("Extra data field is specified without any value")]
    ExtraDataEmpty,

    /// The first extra data field does not specify "pkgtype".
    #[error("The first extra data definition does not specify \"pkgtype\"")]
    FirstExtraDataNotPkgType,

    /// An invalid enum variant
    #[error("Invalid variant ({0})")]
    InvalidVariant(#[from] strum::ParseError),

    /// Unsupported schema version
    #[error("Unsupported schema version: {0}")]
    UnsupportedSchemaVersion(String),
}
