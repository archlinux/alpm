#![doc = include_str!("../README.md")]

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;
pub mod error;
pub mod pkgbuild_bridge;
pub mod source_info;

pub use error::Error;
pub use source_info::{
    SourceInfo,
    v1::{SourceInfoV1, merged::MergedPackage},
};

mod schema;
pub use schema::SourceInfoSchema;

fluent_i18n::i18n!("locales");
