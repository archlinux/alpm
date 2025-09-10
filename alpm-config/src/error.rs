use std::path::PathBuf;

/// Errors that can occur when using alpm-lint-config.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// IO error
    #[error("I/O error while {context}:\n{source}")]
    Io {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O error while ...".
        context: &'static str,
        /// The error source.
        source: std::io::Error,
    },

    /// IO error with additional path info for more context.
    #[error("I/O error at path {path:?} while {context}:\n{source}")]
    IoPath {
        /// The path at which the error occurred.
        path: PathBuf,
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O error at path $path while ...".
        context: &'static str,
        /// The error source
        source: std::io::Error,
    },

    /// TOML de/serialization error
    #[error("Failed to deserialize configuration: {0}")]
    Deserialization(#[from] toml::de::Error),
}
