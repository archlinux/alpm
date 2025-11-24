//! Handling of [alpm-repo-desc] file format versions.
//!
//! [alpm-repo-desc]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html

mod file;
mod parser;
mod schema;
mod v1;
mod v2;

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod commands;

pub use file::RepoDescFile;
pub use parser::{Section, SectionKeyword};
pub use schema::RepoDescSchema;
pub use v1::RepoDescFileV1;
pub use v2::RepoDescFileV2;
