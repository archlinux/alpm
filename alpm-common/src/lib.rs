#![doc = include_str!("../README.md")]

mod error;
mod package;
mod traits;
pub use error::Error;
pub use package::input::{relative_data_files, relative_files};
pub use traits::{metadata_file::MetadataFile, schema::FileFormatSchema};
