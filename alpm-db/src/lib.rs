#![doc = include_str!("../README.md")]
#![cfg_attr(test, expect(clippy::indexing_slicing, clippy::panic))]

pub mod db;
pub mod desc;
mod error;
pub mod files;

pub use error::Error;

// Initialize i18n support.
fluent_i18n::i18n!("locales");
