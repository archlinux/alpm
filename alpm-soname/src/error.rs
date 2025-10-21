//! Error handling.

use std::path::PathBuf;

use fluent_i18n::t;

/// The error that can occur when working with the library.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// I/O path error
    #[error("{msg}", msg = t!("error-io-path-error", {
        "path" => path,
        "context" => context,
        "source" => source.to_string()
    }))]
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
    #[error("{msg}", msg = t!("error-io-write-error", {
        "context" => context,
        "source" => source.to_string()
    }))]
    IoWriteError {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O write error while ".
        context: &'static str,

        /// The source of the error.
        source: std::io::Error,
    },

    /// I/O error while reading.
    #[error("{msg}", msg = t!("error-io-read-error", {
        "context" => context,
        "source" => source.to_string()
    }))]
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
    #[error("{msg}", msg = t!("error-elf-error", {
        "context" => context,
        "source" => source.to_string()
    }))]
    ElfError {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "ELF format error while ".
        context: &'static str,

        /// The source of the error.
        source: goblin::error::Error,
    },

    /// Input directory not supported
    #[error("{msg}", msg = t!("error-input-dir-not-supported", { "path" => path }))]
    InputDirectoryNotSupported {
        /// The path of the input directory.
        path: PathBuf,
    },

    /// JSON error
    #[error("{msg}", msg = t!("error-json", { "source" => .0.to_string() }))]
    Json(#[from] serde_json::Error),
}
