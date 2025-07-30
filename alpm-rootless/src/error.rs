//! Error handling for rootless backends.

/// An error that can occur when using a rootless backend.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An executable that is supposed to be called, is not found.
    #[error("Unable to to find executable \"{command}\"")]
    ExecutableNotFound {
        /// The executable that could not be found.
        command: String,
        /// The source error.
        source: which::Error,
    },

    /// A command could not be executed.
    #[error("The command \"{command}\" could not be executed:\n{source}")]
    CommandExec {
        /// The command that could not be executed.
        command: String,
        /// The source error.
        source: std::io::Error,
    },

    /// Unknown output of [systemd-detect-virt] detected.
    ///
    /// [systemd-detect-virt]: https://man.archlinux.org/man/systemd-detect-virt.1
    #[error("Unknown output of \"systemd-detect-virt\": \"{output}\"")]
    UnknownSystemdDetectVirtOutput {
        /// The unknown output for [systemd-detect-virt].
        output: String,
    },
}
