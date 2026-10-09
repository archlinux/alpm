//! Error handling for [alpm-db] creation, access and writing.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::path::PathBuf;

use fluent_i18n::t;

/// The error that can occur when accessing or writing an [alpm-db].
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An [alpm-db] entry is not a directory but a symlink.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error("{msg}", msg = t!("error-database-entry-is-not-a-directory", { "path" => .path }))]
    EntryIsNotADirectory {
        /// The path of the entry, that is not a directory.
        path: PathBuf,
    },

    /// An [alpm-db] entry has no filename.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error("{msg}", msg = t!("error-database-entry-has-no-filename", { "path" => .path }))]
    EntryHasNoFilename {
        /// The path, that has no filename.
        path: PathBuf,
    },

    /// An [alpm-db] entry contains non-UTF-8 characters.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error("{msg}", msg = t!("error-database-entry-has-no-filename", { "path" => .path }))]
    EntryContainsNonUtf8Chars {
        /// The path, that has no filename.
        path: PathBuf,
    },
}
