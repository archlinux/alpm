#![doc = include_str!("../README.md")]

mod build_info;
#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;
mod error;
mod schema;

pub use build_info::{BuildInfo, v1::BuildInfoV1, v2::BuildInfoV2};
pub use error::Error;
pub use schema::BuildInfoSchema;

fluent_i18n::i18n!("locales");
