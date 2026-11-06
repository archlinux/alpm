#![doc = include_str!("../README.md")]
#![cfg_attr(test, expect(clippy::indexing_slicing, clippy::panic))]

pub mod db;
pub mod desc;
mod error;
pub mod files;

/// Re-exports for types from the [`alpm_types`] crate which are part of the public API.
pub mod alpm_types {
    pub use alpm_types::{
        Architecture,
        BuildDate,
        EntryName,
        Error,
        ExtraData,
        ExtraDataEntry,
        FullVersion,
        Group,
        InstalledPackage,
        InstalledSize,
        License,
        Md5Checksum,
        Name,
        OptionalDependency,
        PackageBaseName,
        PackageDescription,
        PackageInstallReason,
        PackageRelation,
        PackageValidation,
        Packager,
        RelationOrSoname,
        RelativeFilePath,
        RelativePath,
        SchemaVersion,
        Url,
        semver_version,
    };
}
/// Re-exports for types and traits from the [`alpm_common`] crate which are part of the public API.
pub mod alpm_common {
    pub(crate) use alpm_common::relative_files;
    pub use alpm_common::{Error, FileFormatSchema, InputPaths, MetadataFile};
}
/// Re-exports for types from the [`alpm_mtree`] crate which are part of the public API.
pub mod alpm_mtree {
    pub use alpm_mtree::{Error, Mtree, MtreeTransfer};
}

pub use error::Error;

// Initialize i18n support.
fluent_i18n::i18n!("locales");
