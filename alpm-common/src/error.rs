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

    /// A path is not a directory.
    #[error("The path is not a directory: {path:?}")]
    NotADirectory {
        /// The path that is not a directory.
        path: PathBuf,
    },

    /// One or more paths are not absolute.
    #[error(
        "The following paths are not absolute:\n{}",
        paths.iter().fold(
            String::new(),
            |mut output, path| {
                output.push_str(&format!("{path:?}\n"));
                output
            })
    )]
    NonAbsolutePaths {
        /// The list of non-absolute paths.
        paths: Vec<PathBuf>,
    },

    /// One or more paths are not relative.
    #[error(
        "The following paths are not relative:\n{}",
        paths.iter().fold(
            String::new(),
            |mut output, path| {
                output.push_str(&format!("{path:?}\n"));
                output
            })
    )]
    NonRelativePaths {
        /// The list of non-relative paths.
        paths: Vec<PathBuf>,
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
