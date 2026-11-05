#![doc = include_str!("../README.md")]
#![cfg_attr(test, expect(clippy::panic))]

pub mod desc;
mod error;
pub mod files;

/// Re-exports for types from the [`alpm_types`] crate which are part of the public API.
pub mod alpm_types {
    pub use alpm_types::{
        Architecture,
        Base64OpenPGPSignature,
        BuildDate,
        CompressedSize,
        Error,
        FullVersion,
        Group,
        InstalledSize,
        License,
        Md5Checksum,
        Name,
        OptionalDependency,
        PackageBaseName,
        PackageDescription,
        PackageFileName,
        PackageRelation,
        Packager,
        RelationOrSoname,
        RelativePath,
        SchemaVersion,
        Sha256Checksum,
        Url,
        semver_version,
    };
}
/// Re-exports for types and traits from the [`alpm_common`] crate which are part of the public API.
pub mod alpm_common {
    pub(crate) use alpm_common::relative_files;
    pub use alpm_common::{Error, FileFormatSchema, MetadataFile};
}

pub use error::Error;

// Initialize i18n support.
fluent_i18n::i18n!("locales");
