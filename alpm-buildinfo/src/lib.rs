#![doc = include_str!("../README.md")]

mod build_info;
pub use crate::build_info::{BuildInfo, v1::BuildInfoV1, v2::BuildInfoV2};

/// Commandline argument handling. This is most likely not interesting for you.
#[cfg(feature = "cli")]
pub mod cli;
/// Commandline functions, that're called by the `alpm-buildinfo` executable.
#[cfg(feature = "cli")]
pub mod commands;

mod error;
pub use error::Error;

mod schema;
pub use schema::BuildInfoSchema;

fluent_i18n::i18n!("locales");
