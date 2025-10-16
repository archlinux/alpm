//! Error handling for alpm-files.

use std::path::PathBuf;

use fluent_i18n::t;

/// The error that can occur when working with the [alpm-files] format.
///
/// [alpm-files]: https://alpm.archlinux.page/specifications/alpm-files.5.html
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An [`alpm_common::Error`] occurred.
    #[error(transparent)]
    AlpmCommon(#[from] alpm_common::Error),

    /// One or more invalid paths for a [`Files`][`crate::Files`] are encountered.
    #[error("{msg}", msg = t!("error-invalid-files-paths", { "message" => message }))]
    InvalidFilesPaths {
        /// An error message that explains which paths are invalid and why.
        message: String,
    },

    /// An I/O error occurred.
    #[error("{msg}", msg = t!("error-io", { "context" => context, "source" => source.to_string() }))]
    Io {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O error while ".
        /// See the fluent-i18n token "error-io" for details.
        context: String,
        /// The source error.
        source: std::io::Error,
    },

    /// An I/O error occurred at a path.
    #[error(
        "{msg}",
        msg = t!(
            "error-io",
            {
                "path" => path.display().to_string(),
                "context" => context,
                "source" => source.to_string(),
            }
        )
    )]
    IoPath {
        /// The path at which the error occurred.
        path: PathBuf,
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O error at path while ".
        /// See the fluent-i18n token "error-io-path" for details.
        context: String,
        /// The source error.
        source: std::io::Error,
    },

    /// A winnow parser for a type didn't work and produced an error.
    #[error("{msg}", msg = t!("error-parse", { "error" => .0 }))]
    ParseError(String),

    /// No schema version can be derived from [alpm-files] data.
    ///
    /// [alpm-files]: https://alpm.archlinux.page/specifications/alpm-files.5.html
    #[error("{msg}", msg = t!("error-version-is-unknown"))]
    SchemaVersionIsUnknown,
}

impl<'a> From<winnow::error::ParseError<&'a str, winnow::error::ContextError>>
    for crate::error::Error
{
    /// Converts a [`winnow::error::ParseError`] into an [`Error::ParseError`].
    fn from(value: winnow::error::ParseError<&'a str, winnow::error::ContextError>) -> Self {
        Self::ParseError(value.to_string())
    }
}
