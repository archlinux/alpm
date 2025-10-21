//! All error types that are exposed by this crate.

use std::{path::PathBuf, string::FromUtf8Error};

use fluent_i18n::t;
use thiserror::Error;

/// The high-level error that can occur when using this crate.
#[derive(Debug, Error)]
pub enum Error {
    /// ALPM types error.
    #[error(transparent)]
    AlpmType(#[from] alpm_types::Error),

    /// UTF-8 parse error.
    #[error(transparent)]
    InvalidUTF8(#[from] FromUtf8Error),

    /// IO error.
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
        "path" => path,
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
        /// The error source.
        source: std::io::Error,
    },

    /// Invalid file encountered.
    #[error("{msg}", msg = t!("error-invalid-file", {
        "path" => path,
        "context" => context
    }))]
    InvalidFile {
        /// The path of the invalid file.
        path: PathBuf,
        /// The context in which the error occurred.
        context: String,
    },

    /// The alpm-pkgbuild-bridge script could not be found in `$PATH`.
    #[error("{msg}", msg = t!("error-script-not-found", {
        "script_name" => script_name,
        "source" => source.to_string()
    }))]
    ScriptNotFound {
        /// The name of the script that couldn't be found.
        script_name: String,
        /// The error source.
        source: which::Error,
    },

    /// The pkgbuild bridge script failed to be started.
    #[error("{msg}", msg = t!("error-script-failed-start", {
        "context" => context,
        "parameters" => format!("{parameters:?}"),
        "source" => source.to_string()
    }))]
    ScriptError {
        /// The context in which the error occurred.
        context: String,
        /// The parameters supplied to the script.
        parameters: Vec<String>,
        /// The error source.
        source: std::io::Error,
    },

    /// The pkgbuild bridge script errored with some log output.
    #[error("{msg}", msg = t!("error-script-execution", {
        "parameters" => format!("{parameters:?}"),
        "stdout" => stdout,
        "stderr" => stderr
    }))]
    ScriptExecutionError {
        /// The parameters supplied to the script.
        parameters: Vec<String>,
        /// The stdout of the failed command.
        stdout: String,
        /// The stderr of the failed command.
        stderr: String,
    },

    /// A parsing error that occurred during winnow file parsing.
    #[error("{msg}", msg = t!("error-bridge-parse", { "error" => .0 }))]
    BridgeParseError(String),

    /// JSON error while creating JSON formatted output.
    #[error("{msg}", msg = t!("error-json", { "source" => .0.to_string() }))]
    Json(#[from] serde_json::Error),
}
