//! Handling of [alpm-repo-desc] file format versions.
//!
//! [alpm-repo-desc]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html

mod v1;
pub use v1::RepoDescFileV1;

mod v2;
pub use v2::RepoDescFileV2;

mod parser;
pub use parser::{Section, SectionKeyword};

mod schema;
pub use schema::RepoDescSchema;

mod file;
pub use file::RepoDescFile;

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod commands;
