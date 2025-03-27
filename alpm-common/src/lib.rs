#![doc = include_str!("../README.md")]

mod package;
mod traits;
pub use package::{INSTALL_SCRIPTLET_FILENAME, MetadataFileName};
pub use traits::{metadata_file::MetadataFile, schema::FileFormatSchema};
