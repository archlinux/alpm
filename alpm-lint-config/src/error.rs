use std::path::PathBuf;

use fluent_i18n::t;

/// Errors that can occur when using alpm-lint-config.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// IO error
    #[error("{msg}", msg = t!("error-io", {
        "context" => context,
        "source" => source.to_string()
    }))]
    Io {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O error while ...".
        context: String,
        /// The error source.
        source: std::io::Error,
    },

    /// IO error with additional path info for more context.
    #[error("{msg}", msg = t!("error-io-path", {
        "path" => path.display().to_string(),
        "context" => context,
        "source" => source.to_string()
    }))]
    IoPath {
        /// The path at which the error occurred.
        path: PathBuf,
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O error at path $path while ...".
        context: String,
        /// The error source
        source: std::io::Error,
    },

    /// TOML de/serialization error
    #[error("{msg}", msg = t!("error-toml-deserialization", { "source" => .0.to_string() }))]
    Deserialization(#[from] toml::de::Error),
}
