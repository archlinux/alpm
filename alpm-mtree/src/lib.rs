#![doc = include_str!("../README.md")]

/// Commandline argument handling. This is most likely not interesting for you.
#[cfg(feature = "cli")]
pub mod cli;
/// Commandline functions, that're called by the `alpm-mtree` executable.
#[cfg(feature = "cli")]
pub mod commands;
mod error;
pub use error::Error;

pub mod mtree;
pub use mtree::v2::parse_mtree_v2;

/// Low-level parser for MTREE files. You'll likely want to use [`parse_mtree_v2`] instead.
pub mod parser;
/// MTREE files use a special non-ascii encoding for their paths.
mod path_decoder;

mod utils;
pub(crate) use utils::mtree_buffer_to_string;

mod schema;
pub use schema::MtreeSchema;
