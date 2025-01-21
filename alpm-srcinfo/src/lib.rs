#![doc = include_str!("../README.md")]

/// Commandline argument handling.
#[cfg(feature = "cli")]
pub mod cli;
/// Functions called from the binary.
#[cfg(feature = "cli")]
pub mod commands;
/// All error types that are exposed by this crate.
pub mod error;
/// Provides fully resolved package metadata derived from SRCINFO data.
pub mod merged;
/// The parser for SRCINFO data.
///
/// It returns a rather raw line-based, but already typed representation of the contents.
/// The representation is not useful for end-users as it provides data that is not yet validated.
pub mod parser;
/// Contains the second parsing and linting pass.
///
/// The raw representation from the [`parser`] module is brought into a proper struct-based
/// representation that fully represents the SRCINFO data (apart from comments and empty lines).
pub mod source_info;

pub use error::Error;
pub use merged::MergedPackage;
pub use source_info::{SourceInfo, SourceInfoResult};
