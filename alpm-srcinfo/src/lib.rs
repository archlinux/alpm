#![doc = include_str!("../README.md")]

#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "cli")]
pub mod commands;
pub mod error;
pub mod parser;
pub mod source_info;

pub use error::Error;
pub use source_info::{SourceInfo, SourceInfoResult};
