#![doc = include_str!("../README.md")]
#![cfg_attr(test, expect(clippy::panic))]

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;
mod error;
#[cfg(feature = "creation")]
pub mod file;
pub mod mtree;
pub mod parser;
mod path_decoder;
mod schema;
mod utils;

pub use error::Error;
#[cfg(feature = "creation")]
pub use file::{
    create::{create_mtree_v1_from_input_dir, create_mtree_v2_from_input_dir},
    error::Error as CreationError,
};
pub use mtree::{Mtree, v2::parse_mtree_v2};
pub use schema::MtreeSchema;
pub(crate) use utils::{data_is_gzip, mtree_buffer_to_string};

fluent_i18n::i18n!("locales");
