//! Handling of [alpm-db-desc] file format versions.
//!
//! [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;
#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod commands;
mod file;
mod parser;
mod schema;
mod v1;
mod v2;

pub use file::DbDescFile;
pub use parser::{Section, SectionKeyword};
pub use schema::DbDescSchema;
pub use v1::DbDescFileV1;
pub use v2::DbDescFileV2;
