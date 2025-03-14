#![doc = include_str!("../README.md")]

pub mod package_info;
pub use package_info::{PackageInfo, v1::PackageInfoV1, v2::PackageInfoV2};

#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "cli")]
pub mod commands;

mod error;
pub use crate::error::Error;

pub mod utils;
pub use crate::utils::RelationOrSoname;

mod schema;
pub use schema::PackageInfoSchema;
