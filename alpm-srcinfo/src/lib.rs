#![doc = include_str!("../README.md")]

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;
pub mod error;
pub mod pkgbuild_bridge;
mod schema;
pub mod source_info;

/// Re-exports for types from the [`alpm_types`] crate which are part of the public API.
pub mod alpm_types {
    pub use alpm_types::{
        Architecture,
        Architectures,
        Backup,
        Changelog,
        Epoch,
        Error,
        FullVersion,
        Group,
        Install,
        License,
        MakepkgOption,
        Name,
        OpenPGPIdentifier,
        OptionalDependency,
        PackageDescription,
        PackageRelation,
        PackageRelease,
        PackageVersion,
        RelationOrSoname,
        RelativeFilePath,
        SchemaVersion,
        SkippableChecksum,
        Source,
        SystemArchitecture,
        Url,
        digests,
        semver_version,
    };
}

pub use error::Error;
pub use schema::SourceInfoSchema;
pub use source_info::{
    SourceInfo,
    v1::{SourceInfoV1, merged::MergedPackage},
};

fluent_i18n::i18n!("locales");
