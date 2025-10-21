#![doc = include_str!("../README.md")]

pub mod config;
pub mod error;
pub mod input;
pub mod package;
mod scriptlet;

pub use config::{OutputDir, PackageCreationConfig};
pub use error::Error;
pub use input::{InputDir, PackageInput};
pub use package::{ExistingAbsoluteDir, MetadataEntry, Package, PackageEntry, PackageReader};

fluent_i18n::i18n!("locales");
