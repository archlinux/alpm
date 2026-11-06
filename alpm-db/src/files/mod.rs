//! The representation of [alpm-db-files] files.
//!
//! [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;
mod error;
mod file;
mod schema;
mod v1;

pub use error::Error;
pub use file::DbFiles;
pub use schema::DbFilesSchema;
pub use v1::{BackupEntry, DbFilesV1};
