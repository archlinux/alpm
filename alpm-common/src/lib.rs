#![doc = include_str!("../README.md")]

mod error;
mod package;
mod traits;
pub use error::Error;
pub use package::input::{InputPath, InputPaths, relative_data_files, relative_files};
pub use traits::{
    build::BuildRelationLookupData,
    metadata_file::MetadataFile,
    package_data::RuntimeRelationLookupData,
    schema::FileFormatSchema,
};

fluent_i18n::i18n!("locales");
