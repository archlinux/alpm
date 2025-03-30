//! Error handling for [ALPM-MTREE] creation.
//!
//! [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html

use std::process::ExitStatus;

/// The Error that can occur when creating [ALPM-MTREE] files.
///
/// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An alpm-common error.
    #[error("ALPM common error:\n{0}")]
    AlpmCommon(#[from] alpm_common::Error),

    /// Unable to attach to stdin of a command.
    #[error("Unable to attach to stdin of command {command:?}")]
    CommandAttachToStdin {
        /// The command for which attaching to stdin failed.
        command: String,
    },

    /// A command could not be started in the background.
    #[error("The command {command:?} could not be started in the background:\n{source}")]
    CommandBackground {
        /// The command that could not be started in the background.
        command: String,
        /// The source error.
        source: std::io::Error,
    },

    /// A command could not be executed.
    #[error("The command {command:?} could not be executed:\n{source}")]
    CommandExec {
        /// The command that could not be executed.
        command: String,
        /// The source error.
        source: std::io::Error,
    },

    /// A command exited unsuccessfully.
    #[error(
        "The command {command:?} exited with non-zero status code {exit_status:?}:\nstderr:\n{stderr}"
    )]
    CommandNonZero {
        /// The command that exited with a non-zero exit code.
        command: String,
        /// The exit status of `command`.
        exit_status: ExitStatus,
        /// The stderr of `command`.
        stderr: String,
    },

    /// A command exited unsuccessfully.
    #[error("The command {command:?} could not be found:\n{source}")]
    CommandNotFound {
        /// The command that exited with a non-zero exit code.
        command: &'static str,
        /// The source error.
        source: which::Error,
    },

    /// Unable to write to stdin of a command.
    #[error("Unable to write to stdin of command {command:?}")]
    CommandWriteToStdin {
        /// The command for which writing to stdin failed.
        command: String,
        /// The source error.
        source: std::io::Error,
    },
}
