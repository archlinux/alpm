#![doc = include_str!("../README.md")]
#![cfg_attr(test, expect(clippy::panic))]

pub mod desc;
mod error;
pub mod files;

pub use error::Error;

// Initialize i18n support.
fluent_i18n::i18n!("locales");
