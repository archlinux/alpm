#![doc = include_str!("../README.md")]

/// Commandline argument handling.
#[cfg(feature = "cli")]
pub mod cli;
/// Functions called from the binary.
#[cfg(feature = "cli")]
pub mod commands;
/// All error types that are exposed by this crate.
pub mod error;
/// The parser for SRCINFO data.
///
/// It returns a rather raw line-based, but already typed representation of the contents.
/// The representation is not useful for end-users as it provides data that is not yet validated.
pub mod parser;

pub use error::Error;
