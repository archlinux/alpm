//! Error handling for rootless backends.

/// An error that can occur when using a rootless backend.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An executable that is supposed to be called, is not found.
    #[error("Unable to to find executable: \"{executable}\"")]
    ExecutableNotFound {
        /// The executable that could not be found.
        executable: String,
        /// The source error.
        source: which::Error,
    },
}
