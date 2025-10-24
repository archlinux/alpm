#![doc = include_str!("../README.md")]

/// Commandline argument handling. This is most likely not interesting for you.
#[cfg(feature = "cli")]
pub mod cli;
mod error;
pub use error::Error;

#[cfg(feature = "creation")]
pub mod file;
#[cfg(feature = "creation")]
pub use file::{
    create::{create_mtree_v1_from_input_dir, create_mtree_v2_from_input_dir},
    error::Error as CreationError,
};

pub mod mtree;
pub use mtree::{Mtree, v2::parse_mtree_v2};

/// Low-level parser for MTREE files. You'll likely want to use [`parse_mtree_v2`] instead.
pub mod parser;
/// MTREE files use a special non-ascii encoding for their paths.
mod path_decoder;

mod utils;
pub(crate) use utils::mtree_buffer_to_string;

mod schema;
pub use schema::MtreeSchema;
