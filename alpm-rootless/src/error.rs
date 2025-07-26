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
}
