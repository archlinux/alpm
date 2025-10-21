#![doc = include_str!("../README.md")]

pub mod bridge;
pub mod error;

pub use error::Error;

fluent_i18n::i18n!("locales");
