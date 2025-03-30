#![doc = include_str!("../README.md")]

mod error;
mod package;
mod package_input;
mod traits;
pub use error::Error;
pub use package::{INSTALL_SCRIPTLET_FILENAME, MetadataFileName};
pub use package_input::{relative_data_files, relative_files};
pub use traits::{metadata_file::MetadataFile, schema::FileFormatSchema};
