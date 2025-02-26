#![doc = include_str!("../README.md")]

pub mod compression;
pub mod error;
pub mod input;
pub mod package;
pub mod pipeline;
mod scriptlet;

pub use compression::{CompressionEncoder, CompressionSettings};
pub use error::Error;
pub use input::{InputDir, PackageInput};
pub use package::{ExistingAbsoluteDir, Package};
pub use pipeline::{OutputDir, PackageCreationPipeline};
