#![doc = include_str!("../README.md")]

mod package_info_v1;
pub use crate::package_info_v1::PackageInfoV1;

mod package_info_v2;
pub use crate::package_info_v2::PackageInfoV2;

pub mod cli;
pub mod commands;

mod error;
pub use crate::error::Error;
