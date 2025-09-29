#![doc = include_str!("../README.md")]

pub mod config;
pub mod error;
pub mod input;
pub mod package;
mod scriptlet;

pub use config::{OutputDir, PackageCreationConfig};
pub use error::Error;
pub use input::{InputDir, PackageInput};
pub use package::{
    DataEntry,
    ExistingAbsoluteDir,
    MetadataEntry,
    Package,
    PackageEntry,
    PackageReader,
};
