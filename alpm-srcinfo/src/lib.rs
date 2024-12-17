#![doc = include_str!("../README.md")]

#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "cli")]
pub mod commands;
pub mod error;
pub mod parser;

pub use error::Error;
