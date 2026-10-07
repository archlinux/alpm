#![doc = include_str!("../README.md")]
#![cfg_attr(
    test,
    expect(clippy::arithmetic_side_effects, clippy::expect_used, clippy::panic)
)]

mod error;

pub mod compression;
pub mod decompression;
pub mod tarball;

pub use error::Error;

fluent_i18n::i18n!("locales");
