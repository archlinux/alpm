#![doc = include_str!("../README.md")]

/// Commandline argument handling.
#[cfg(feature = "cli")]
pub mod cli;
/// Functions called from the binary.
#[cfg(feature = "cli")]
pub mod commands;
/// All error types that are exposed by this crate.
pub mod error;
/// The first parsing pass for SRCINFO data.
///
/// It returns a rather raw line-based, but already typed representation of the contents.
/// The representation is not useful for end-users as it provides data that is not yet validated.
pub mod parser;
/// This module contains the second parsing and linting pass.
/// The raw representation from the [`parser`] module is brought into a proper struct-based
/// representation that fully represents the SRCINFO data (apart from comments and empty lines).
///
/// The [`source_info::SourceInfo::from_parsed`] function returns an error array instead of a
/// `Result` as those errors may only be linting errors that can be ignored.
/// The consumer has to handle those errors and decide what to do based on their type.
pub mod source_info;

pub use error::Error;
pub use source_info::{SourceInfo, SourceInfoResult};
