//! Error handling for the `dev-scripts` executable.

use std::path::PathBuf;

use colored::Colorize;
use log::SetLoggerError;

/// The error that can occur when using the `dev-scripts` executable.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An `alpm_buildinfo::Error` occurred.
    #[error(transparent)]
    AlpmBuildinfo(#[from] alpm_buildinfo::Error),

    /// An `alpm_pkginfo::Error` occurred.
    #[error(transparent)]
    AlpmPackageInfo(#[from] alpm_pkginfo::Error),

    /// An `alpm_db::Error` occurred.
    #[error(transparent)]
    AlpmDb(#[from] alpm_db::Error),

    /// An `alpm_mtree::Error` occurred.
    #[error(transparent)]
    AlpmMtree(#[from] alpm_mtree::Error),

    /// An `alpm_srcinfo::Error` occurred.
    #[error(transparent)]
    AlpmSourceInfo(#[from] alpm_srcinfo::Error),

    /// An `alpm_types::Error` occurred.
    #[error(transparent)]
    AlpmTypes(#[from] alpm_types::Error),

    /// The logger cannot be setup.
    #[error("Failed to setup the logger:\n{0}")]
    SetupLogger(#[from] SetLoggerError),

    /// It is not possible to get the cache directory for the current user.
    #[error("Failed to determine the current user's cache directory")]
    CannotGetCacheDir,

    /// A command failed.
    #[error("A command failed:{message}\nstdout:\n{stdout}\nstderr:\n{stderr}")]
    CommandFailed {
        /// A message explaining what command failed.
        message: String,
        /// The stdout of the failed command.
        stdout: String,
        /// The stderr of the failed command.
        stderr: String,
    },

    #[error("An HTTP query failed while {context}:\n{source}")]
    HttpQueryFailed {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "An HTTP query failed while {context}".
        context: String,
        /// The source error.
        source: reqwest::Error,
    },

    /// An I/O error occurred.
    #[error("I/O error while {context}:\n{source}")]
    Io {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O error while ".
        context: String,
        /// The source error.
        source: std::io::Error,
    },

    /// An I/O error occurred at a path.
    #[error("I/O error at path {path} while {context}:\n{source}")]
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

    /// A JSON error occurred.
    #[error("JSON error while {context}:\n{source}")]
    Json {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "JSON error while ".
        context: String,
        /// The source error.
        source: serde_json::Error,
    },

    /// A winnow parser for a type didn't work and produced an error.
    #[error("Parser error:\n{0}")]
    Parser(String),

    #[error("Rsync report error:\n{message}")]
    RsyncReport { message: String },

    /// A test run failed.
    #[error(
        "The test run failed\n{}",
        failures
            .iter()
            .map(|(index, path, message)| {
                let index = format!("[{index}]").bold().red();
                format!("{index} {} failed with error:\n{message}", path.to_string_lossy().bold())})
            .collect::<Vec<_>>()
            .join("\n")
    )]
    TestFailed {
        /// The failed items as tuples of index, paths and messages.
        failures: Vec<(usize, PathBuf, String)>,
    },

    /// A `voa::Error` occurred.
    #[error(transparent)]
    Voa(#[from] voa::Error),

    #[error("Verifying the file {file:?} with signature {signature:?} failed:\n{context}")]
    VoaVerificationFailed {
        /// The path of the data file that failed verification.
        file: PathBuf,
        /// The path of the signature file that failed verification.
        signature: PathBuf,
        /// Additional context.
        context: String,
    },
}

impl<'a> From<winnow::error::ParseError<&'a str, winnow::error::ContextError>>
    for crate::error::Error
{
    /// Converts a [`winnow::error::ParseError`] into an [`Error::Parser`].
    fn from(value: winnow::error::ParseError<&'a str, winnow::error::ContextError>) -> Self {
        Self::Parser(value.to_string())
    }
}
