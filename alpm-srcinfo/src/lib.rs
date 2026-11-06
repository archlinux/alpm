#![doc = include_str!("../README.md")]

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;
pub mod error;
pub mod pkgbuild_bridge;
mod schema;
pub mod source_info;

pub use error::Error;
pub use schema::SourceInfoSchema;
pub use source_info::{
    SourceInfo,
    v1::{SourceInfoV1, merged::MergedPackage},
};

fluent_i18n::i18n!("locales");
