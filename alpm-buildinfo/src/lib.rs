#![doc = include_str!("../README.md")]

mod build_info;
#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;
mod error;
mod schema;

/// Re-exports for types from the [`alpm_types`] crate which are part of the public API.
pub mod alpm_types {
    pub use alpm_types::{
        Architecture,
        BuildDate,
        BuildDirectory,
        BuildEnvironmentOption,
        BuildTool,
        BuildToolVersion,
        Checksum,
        Error,
        FullVersion,
        InstalledPackage,
        Name,
        PackageOption,
        Packager,
        SchemaVersion,
        Sha256Checksum,
        StartDirectory,
        digests,
        semver_version,
    };
}

pub use build_info::{BuildInfo, v1::BuildInfoV1, v2::BuildInfoV2};
pub use error::Error;
pub use schema::BuildInfoSchema;

fluent_i18n::i18n!("locales");
