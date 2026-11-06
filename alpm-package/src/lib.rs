#![doc = include_str!("../README.md")]
#![cfg_attr(test, expect(clippy::panic))]

pub mod config;
pub mod error;
pub mod input;
pub mod package;
mod scriptlet;

/// Re-exports for types from the [`alpm_types`] crate which are part of the public API.
pub mod alpm_types {
    pub(crate) use alpm_types::INSTALL_SCRIPTLET_FILE_NAME;
    pub use alpm_types::{
        Architecture,
        Error,
        FullVersion,
        MetadataFileName,
        Name,
        PackageError,
        PackageFileName,
        Packager,
        Sha256Checksum,
    };
}

pub use config::{OutputDir, PackageCreationConfig};
pub use error::Error;
pub use input::{InputDir, PackageInput};
pub use package::{ExistingAbsoluteDir, MetadataEntry, Package, PackageEntry, PackageReader};

fluent_i18n::i18n!("locales");
