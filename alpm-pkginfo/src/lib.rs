#![doc = include_str!("../README.md")]

pub mod package_info;
pub use package_info::{PackageInfo, v1::PackageInfoV1, v2::PackageInfoV2};

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;

mod error;
pub use crate::error::Error;

pub mod utils;
pub use crate::utils::RelationOrSoname;

mod schema;
pub use schema::PackageInfoSchema;

fluent_i18n::i18n!("locales");
