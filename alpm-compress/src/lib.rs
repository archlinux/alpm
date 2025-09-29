#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod compression;
pub mod decompression;
pub mod error;

pub use error::Error;
