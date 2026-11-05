#![doc = include_str!("../README.md")]

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;
mod error;
pub mod package_info;
mod schema;

/// Re-exports for types from the [`alpm_types`] crate which are part of the public API.
pub mod alpm_types {
    pub use alpm_types::{
        Architecture,
        Backup,
        BuildDate,
        Error,
        ExtraData,
        ExtraDataEntry,
        FullVersion,
        Group,
        InstalledSize,
        License,
        Name,
        OptionalDependency,
        PackageDescription,
        PackageRelation,
        Packager,
        RelationOrSoname,
        SchemaVersion,
        Url,
        semver_version,
    };
}

pub use error::Error;
pub use package_info::{PackageInfo, v1::PackageInfoV1, v2::PackageInfoV2};
pub use schema::PackageInfoSchema;

fluent_i18n::i18n!("locales");
