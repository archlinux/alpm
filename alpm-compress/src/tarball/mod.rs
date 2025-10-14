//! Reading and writing optionally compressed tarballs.

mod builder;
mod reader;

pub use builder::TarballBuilder;
pub use reader::{TarballEntries, TarballEntry, TarballReader};
