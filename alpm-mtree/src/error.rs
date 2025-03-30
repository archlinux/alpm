use std::{path::PathBuf, process::ExitStatus, string::FromUtf8Error};

/// The Error that can occur when working with ALPM-MTREE
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// An alpm-common error.
    #[error("ALPM common error:\n{0}")]
    AlpmCommon(#[from] alpm_common::Error),

    /// Unable to attach to stdin of a command.
    #[error("Unable to attach to stdin of command \"{command}\"")]
    CommandAttachToStdin {
        /// The command for which attaching to stdin failed.
        command: String,
    },

    /// A command could not be started in the background.
    #[error("The command \"{command}\" could not be started in the background:\n{source}")]
    CommandBackground {
        /// The command that could not be started in the background.
        command: String,
        /// The source error.
        source: std::io::Error,
    },

    /// A command could not be executed.
    #[error("The command \"{command}\" could not be executed:\n{source}")]
    CommandExec {
        /// The command that could not be executed.
        command: String,
        /// The source error.
        source: std::io::Error,
    },

    /// A command exited unsuccessfully.
    #[error(
        "The command \"{command}\" exited with non-zero status code \"{exit_status}\":\nstderr:\n{stderr}"
    )]
    CommandNonZero {
        /// The command that exited with a non-zero exit code.
        command: String,
        /// The exit status of `command`.
        exit_status: ExitStatus,
        /// The stderr of `command`.
        stderr: String,
    },

    /// Unable to write to stdin of a command.
    #[error("Unable to write to stdin of command \"{command}\"")]
    CommandWriteToStdin {
        /// The command for which writing to stdin failed.
        command: String,
        /// The source error.
        source: std::io::Error,
    },

    /// IO error
    #[error("I/O error while {0}:\n{1}")]
    Io(&'static str, std::io::Error),

    /// IO error with additional path info for more context.
    #[error("I/O error at path {0:?} while {1}:\n{2}")]
    IoPath(PathBuf, &'static str, std::io::Error),

    /// UTF-8 parse error
    #[error(transparent)]
    InvalidUTF8(#[from] FromUtf8Error),

    /// No input file given
    #[error("No input file given.")]
    NoInputFile,

    /// An error occurred while unpacking a gzip file.
    #[error("Error while unpacking gzip file:\n{0}")]
    InvalidGzip(std::io::Error),

    /// A Parsing error that occurred during the winnow file parsing.
    #[error("File parsing error:\n{0}")]
    ParseError(String),

    /// An error occurred during the interpretation phase of the language.
    #[error("Error while interpreting file in line {0}:\nAffected line:\n{1}\n\nReason:\n{2}")]
    InterpreterError(usize, String, String),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Joining a thread returned an error.
    #[error("Thread error while {context}")]
    Thread {
        /// The context in which the failed thread ran.
        ///
        /// Should complete the sentence "Thread error while ".
        context: String,
    },

    /// Unsupported schema version
    #[error("Unsupported schema version: {0}")]
    UnsupportedSchemaVersion(String),
}
