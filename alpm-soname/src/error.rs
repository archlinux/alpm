//! Error handling.

use std::path::PathBuf;

/// The error that can occur when working with the library.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// IO path error
    #[error("I/O error at path {path} while {context}:\n{source}")]
    IoPathError {
        /// The path at which the error occurred.
        path: PathBuf,

        /// The context in which the error occurred at `path`.
        ///
        /// This is meant to complete the sentence "I/O error at path {path} while ".
        context: &'static str,

        /// The source of the error.
        source: std::io::Error,
    },

    /// I/O error while writing.
    #[error("I/O write error while {context}:\n{source}")]
    IoWriteError {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O write error while ".
        context: &'static str,

        /// The source of the error.
        source: std::io::Error,
    },

    /// I/O error while reading.
    #[error("I/O read error while {context}:\n{source}")]
    IoReadError {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O read error while ".
        context: &'static str,

        /// The source of the error.
        source: std::io::Error,
    },

    /// ALPM PKGINFO error
    #[error(transparent)]
    AlpmPkginfo(#[from] alpm_pkginfo::Error),

    /// ALPM types error
    #[error(transparent)]
    AlpmType(#[from] alpm_types::Error),

    /// ALPM package error
    #[error(transparent)]
    AlpmPackage(#[from] alpm_package::Error),

    /// ELF format handling error
    #[error("ELF format error while {context}:\n{source}")]
    ElfError {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "ELF format error while ".
        context: &'static str,

        /// The source of the error.
        source: goblin::error::Error,
    },

    /// Input directory not supported
    #[error("Using input directories is not supported: {path}")]
    InputDirectoryNotSupported {
        /// The path of the input directory.
        path: PathBuf,
    },

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
