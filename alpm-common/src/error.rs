use std::path::{PathBuf, StripPrefixError};

use rust_i18n::t;

/// An error that can occur when dealing with package inputs.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An I/O error occurred at a path.
    #[error("{msg}: {path}:\n{source}", msg = t!("error.io_path", context = context))]
    IoPath {
        /// The path at which the error occurred.
        path: PathBuf,
        /// The context in which the error occurred.
        context: String,
        /// The source error.
        source: std::io::Error,
    },

    /// A path is not a directory.
    #[error("{msg}: {path:?}", msg = t!("error.not_a_directory"))]
    NotADirectory {
        /// The path that is not a directory.
        path: PathBuf,
    },

    /// One or more paths are not absolute.
    #[error(
        "{msg}:\n{paths}",
        msg = t!("error.non_absolute_paths"),
        paths = paths.iter().fold(
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
        "{msg}:\n{paths}",
        msg = t!("error.non_relative_paths"),
        paths = paths.iter().fold(
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
    #[error(
        "{msg}: {prefix} -> {path}\n{source}",
        msg = t!("error.strip_prefix")
    )]
    PathStripPrefix {
        /// The prefix that is supposed to be stripped from `path`.
        prefix: PathBuf,
        /// The path that is supposed to stripped.
        path: PathBuf,
        /// The source error.
        source: StripPrefixError,
    },
}
