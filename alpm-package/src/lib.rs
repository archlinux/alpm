#![doc = include_str!("../README.md")]

pub mod compression;
pub mod config;
pub mod error;
pub mod input;
pub mod package;
mod scriptlet;

pub use compression::{
    Bzip2CompressionLevel,
    CompressionAlgorithm,
    CompressionDecoder,
    CompressionEncoder,
    CompressionSettings,
    GzipCompressionLevel,
    XzCompressionLevel,
    ZstdCompressionLevel,
    ZstdThreads,
};
pub use config::{OutputDir, PackageCreationConfig};
pub use error::Error;
pub use input::{InputDir, PackageInput};
pub use package::{ExistingAbsoluteDir, Package};
