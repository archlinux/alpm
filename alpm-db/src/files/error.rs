//! Error handling for alpm-db-files.

use fluent_i18n::t;

/// The error that can occur when working with the [alpm-db-files] format.
///
/// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An [`alpm_common::Error`] occurred.
    #[error(transparent)]
    AlpmCommon(#[from] alpm_common::Error),

    /// One or more invalid paths for a [`DbFiles`][`crate::files::DbFiles`] are encountered.
    #[error("{msg}", msg = t!("error-invalid-files-paths", { "message" => message }))]
    InvalidFilesPaths {
        /// An error message that explains which paths are invalid and why.
        message: String,
    },

    /// One or more invalid backup entries are encountered.
    #[error(
        "{msg}",
        msg = t!(
            "error-invalid-backup-entries",
            { "message" => message }
        )
    )]
    InvalidBackupEntries {
        /// An error message that explains which backup entries are invalid and why.
        message: String,
    },

    /// No schema version can be derived from [alpm-db-files] data.
    ///
    /// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
    #[error("{msg}", msg = t!("error-version-is-unknown"))]
    UnknownSchemaVersion,
}
