#![doc = include_str!("../README.md")]

#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "cli")]
pub mod commands;
pub mod error;
pub mod merged;
pub mod parser;
pub mod source_info;

pub use error::Error;
pub use merged::MergedPackage;
pub use source_info::{SourceInfo, SourceInfoResult, relation::RelationOrSoname};

mod schema;
pub use schema::SourceInfoSchema;
