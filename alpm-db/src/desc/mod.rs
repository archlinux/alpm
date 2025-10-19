//! Handling of [alpm-db-desc] file format versions.
//!
//! [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html

mod v1;
pub use v1::DbDescFileV1;

mod v2;
pub use v2::DbDescFileV2;

mod parser;
pub use parser::{Section, SectionKeyword};

mod schema;
pub use schema::DbDescSchema;

mod file;
pub use file::DbDescFile;

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod commands;
