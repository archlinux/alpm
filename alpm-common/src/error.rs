use std::path::{PathBuf, StripPrefixError};

/// An error that can occur when dealing with package inputs.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An I/O error occurred at a path.
    #[error("I/O error at path {path} while {context}:\n{source}")]
    IoPath {
        /// The path at which the error occurred.
        path: PathBuf,
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O error at path while ".
        context: &'static str,
        /// The source error.
        source: std::io::Error,
    },

    /// A path's prefix cannot be stripped.
    #[error("Cannot strip prefix {prefix} from path {path}:\n{source}")]
    PathStripPrefix {
        /// The prefix that is supposed to be stripped from `path`.
        prefix: PathBuf,
        /// The path that is supposed to stripped.
        path: PathBuf,
        /// The source error.
        source: StripPrefixError,
    },
}
