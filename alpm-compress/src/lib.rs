#![doc = include_str!("../README.md")]

mod error;

pub mod compression;
pub mod decompression;
pub mod tarball;

pub use error::Error;

fluent_i18n::i18n!("locales");
