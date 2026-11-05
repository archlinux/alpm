#![doc = include_str!("../README.md")]

mod error;
mod package;
mod traits;

/// Re-exports for types from the [`alpm_types`] crate which are part of the public API.
pub mod alpm_types {
    pub(crate) use alpm_types::{INSTALL_SCRIPTLET_FILE_NAME, MetadataFileName};
    pub use alpm_types::{SchemaVersion, semver_version};
}

pub use error::Error;
pub use package::input::{InputPath, InputPaths, relative_data_files, relative_files};
pub use traits::{metadata_file::MetadataFile, schema::FileFormatSchema};

fluent_i18n::i18n!("locales");
