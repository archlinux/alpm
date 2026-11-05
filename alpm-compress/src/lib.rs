#![doc = include_str!("../README.md")]
#![cfg_attr(
    test,
    expect(clippy::arithmetic_side_effects, clippy::expect_used, clippy::panic)
)]

mod error;

pub mod compression;
pub mod decompression;
pub mod tarball;

/// Re-exports for types from the [`alpm_types`] crate which are part of the public API.
pub mod alpm_types {
    pub use alpm_types::{CompressionAlgorithmFileExtension, Error};
}

pub use error::Error;

fluent_i18n::i18n!("locales");
