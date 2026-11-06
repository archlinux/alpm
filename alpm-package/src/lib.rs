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
/// Re-exports for types and traits from the [`alpm_common`] crate which are part of the public API.
pub mod alpm_common {
    pub use alpm_common::{Error, InputPaths};
    pub(crate) use alpm_common::{MetadataFile, relative_files};
}
/// Re-exports for types from the [`alpm_mtree`] crate which are part of the public API.
pub mod alpm_mtree {
    pub use alpm_mtree::*;
}
/// Re-exports for types from the [`alpm_pkginfo`] crate which are part of the public API.
pub mod alpm_pkginfo {
    pub use alpm_pkginfo::*;
}
/// Re-exports for types from the [`alpm_buildinfo`] crate which are part of the public API.
pub mod alpm_buildinfo {
    pub use alpm_buildinfo::*;
}

pub use config::{OutputDir, PackageCreationConfig};
pub use error::Error;
pub use input::{InputDir, PackageInput};
pub use package::{ExistingAbsoluteDir, MetadataEntry, Package, PackageEntry, PackageReader};

fluent_i18n::i18n!("locales");
