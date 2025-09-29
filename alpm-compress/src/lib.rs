#![doc = include_str!("../README.md")]

mod error;

pub mod compression;
pub mod decompression;

pub use error::Error;
