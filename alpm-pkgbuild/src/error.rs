//! All error types that are exposed by this crate.
use std::{path::PathBuf, string::FromUtf8Error};

use thiserror::Error;

use crate::bridge::error::BridgeError;

/// The high-level error that can occur when using this crate.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// ALPM type error
    #[error("ALPM type parse error: {0}")]
    AlpmType(#[from] alpm_types::Error),

    /// UTF-8 parse error when reading the input file.
    #[error(transparent)]
    InvalidUTF8(#[from] FromUtf8Error),

    /// IO error
    #[error("I/O error while {0}:\n{1}")]
    Io(&'static str, std::io::Error),

    /// IO error with additional path info for more context.
    #[error("I/O error at path {0:?} while {1}:\n{2}")]
    IoPath(PathBuf, &'static str, std::io::Error),

    /// Invalid file encountered
    #[error("Encountered invalid file for path {0:?}:\n{1}")]
    InvalidFile(PathBuf, &'static str),

    /// IO error with additional path info for more context.
    #[error("Failed to {0} process to extract PKGBUILD:\nCommand: bash {1:?}\n{2}")]
    ScriptError(&'static str, Vec<String>, std::io::Error),

    /// IO error with additional path info for more context.
    #[error(
        "Error during pkgbuild bridge execution:\nCommand: bash {arguments:?}\nstdout:{stdout}\n\nstderr:{stderr}"
    )]
    ScriptExecutionError {
        arguments: Vec<String>,
        stdout: String,
        stderr: String,
    },

    /// A parsing error that occurred during winnow file parsing.
    #[error(
        "A error happened in the internal bridge output parser. Please report this upstream!:\n{0}"
    )]
    BridgeParseError(String),

    /// A parsing error that occurred during winnow file parsing.
    #[error("{0}")]
    BridgeConversionError(#[from] BridgeError),

    /// A SourceInfo file could not be read.
    #[error("JSON error: {0}")]
    SourceInfoRead(#[from] alpm_srcinfo::error::Error),

    /// JSON error while creating JSON formatted output.
    ///
    /// This error only occurs when running the `commands` functions.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
