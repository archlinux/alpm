#![doc = include_str!("../README.md")]
#![cfg_attr(test, expect(clippy::indexing_slicing, clippy::panic))]

mod error;
pub use error::Error;

pub mod db;

pub mod desc;

pub mod files;

// Initialize i18n support.
fluent_i18n::i18n!("locales");
