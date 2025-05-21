//! All error types that are exposed by this crate.

use std::{path::PathBuf, string::FromUtf8Error};

#[cfg(doc)]
use alpm_srcinfo::SourceInfo;
use thiserror::Error;

use crate::bridge::error::BridgeError;

/// The high-level error that can occur when using this crate.
#[derive(Debug, Error)]
pub enum Error {
    /// ALPM types error.
    #[error(transparent)]
    AlpmType(#[from] alpm_types::Error),

    /// UTF-8 parse error.
    #[error(transparent)]
    InvalidUTF8(#[from] FromUtf8Error),

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

    /// Invalid file encountered
    #[error("Encountered invalid file for path {path:?}:\n{context}")]
    InvalidFile {
        /// The path of the file that's invalid
        path: PathBuf,
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "Encountered invalid file for path ...".
        context: &'static str,
    },

    /// The alpm-pkgbuild-bridge script could not be found in `$PATH`.
    #[error("Could not find '{script_name}' script in $PATH:\n{source}")]
    ScriptNotFound {
        /// The name of the script that couldn't be found.
        script_name: String,
        /// The error source
        source: which::Error,
    },

    /// The pkgbuild bridge script failed to be started.
    #[error(
        "Failed to {context} process to extract PKGBUILD:\nCommand: alpm-pkgbuild-bridge {parameters:?}\n{source}"
    )]
    ScriptError {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "Failed to ...".
        context: &'static str,
        /// The parameters that were supplied to the script.
        parameters: Vec<String>,
        /// The error source
        source: std::io::Error,
    },

    /// The pkgbuild bridge script errored with some log output.
    #[error(
        "Error during pkgbuild bridge execution:\nCommand: alpm-pkgbuild-bridge {parameters:?}\nstdout:{stdout}\n\nstderr:{stderr}"
    )]
    ScriptExecutionError {
        /// The parameters that were supplied to the script.
        parameters: Vec<String>,
        /// The stdout of the failed command.
        stdout: String,
        /// The stderr of the failed command.
        stderr: String,
    },

    /// A parsing error that occurred during winnow file parsing.
    #[error(
        "An unexpected error occurred in the output parser for the 'alpm-pkgbuild-bridge' script:\n{0}\n\nPlease report this as a bug at https://gitlab.archlinux.org/archlinux/alpm/alpm/-/issues"
    )]
    BridgeParseError(String),

    /// A logical error occurred when transforming `alpm-pkgbuild-bridge` script output to a
    /// [`SourceInfo`] struct.
    ///
    /// See [`BridgeError`] for further details.
    #[error(transparent)]
    BridgeConversionError(#[from] BridgeError),

    /// A SourceInfo file could not be read.
    #[error("Error while reading SRCINFO file: {0}")]
    SourceInfoRead(#[from] alpm_srcinfo::error::Error),

    /// JSON error while creating JSON formatted output.
    ///
    /// This error only occurs when running the `commands` functions.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
