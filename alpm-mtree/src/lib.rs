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

/// Re-exports for types from the [`alpm_types`] crate which are part of the public API.
pub mod alpm_types {
    #[cfg(feature = "creation")]
    pub(crate) use alpm_types::MetadataFileName;
    pub(crate) use alpm_types::{Checksum, Digest};
    pub use alpm_types::{Md5Checksum, SchemaVersion, Sha256Checksum, semver_version};
}
/// Re-exports for types and traits from the [`alpm_common`] crate which are part of the public API.
pub mod alpm_common {
    #[cfg(feature = "creation")]
    pub(crate) use alpm_common::relative_files;
    pub use alpm_common::{Error, FileFormatSchema, InputPath, InputPaths, MetadataFile};
}

pub use error::Error;
#[cfg(feature = "creation")]
pub use file::{
    create::{create_mtree_v1_from_input_dir, create_mtree_v2_from_input_dir},
    error::Error as CreationError,
};
pub use mtree::{Mtree, MtreeTransfer, v2::parse_mtree_v2};
pub use schema::MtreeSchema;
pub(crate) use utils::{data_is_gzip, mtree_buffer_to_string};

fluent_i18n::i18n!("locales");
