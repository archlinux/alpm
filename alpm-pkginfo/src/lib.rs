#![doc = include_str!("../README.md")]

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;
mod error;
pub mod package_info;
mod schema;

pub use error::Error;
pub use package_info::{PackageInfo, v1::PackageInfoV1, v2::PackageInfoV2};
pub use schema::PackageInfoSchema;

fluent_i18n::i18n!("locales");
