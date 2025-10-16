#![doc = include_str!("../README.md")]

mod error;
mod files;
mod schema;

pub use error::Error;
pub use files::{Files, FilesStyle, FilesStyleToString, v1::FilesV1};
pub use schema::FilesSchema;

// Initialize i18n support.
fluent_i18n::i18n!("locales");
