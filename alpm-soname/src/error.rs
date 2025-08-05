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
    #[error("IO write error while {context}:\n{source}")]
    IoWriteError {
        /// The context in which the error occurred.
        context: &'static str,

        /// The source of the error.
        source: std::io::Error,
    },

    /// I/O error while reading.
    #[error("IO read error while {context}:\n{source}")]
    IoReadError {
        /// The context in which the error occurred.
        context: &'static str,

        /// The source of the error.
        source: std::io::Error,
    },

    /// Error while running a command
    #[error("Command error while {context}: failed to run '{command}':\n{source}")]
    CommandError {
        /// The context in which the error occurred.
        context: &'static str,

        /// The command that was executed
        command: String,

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
        context: &'static str,

        /// The source of the error.
        source: goblin::error::Error,
    },

    /// Dependency analyzer error
    #[error("Failed to find dependencies for {path}:\n{source}")]
    LibraryDependenciesError {
        /// The path of the library
        path: PathBuf,

        /// The source of the error.
        source: lddtree::Error,
    },
}
