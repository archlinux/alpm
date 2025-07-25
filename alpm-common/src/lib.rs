#![doc = include_str!("../README.md")]

mod error;
pub mod locale;
mod package;
mod traits;
pub use error::Error;
pub use fluent_templates;
pub use locale::{get_locale, set_locale};
pub use package::input::{InputPath, InputPaths, relative_data_files, relative_files};
pub use traits::{metadata_file::MetadataFile, schema::FileFormatSchema};

#[cfg(not(test))]
i18n!("locales", fallback = "en-US");

#[cfg(test)]
i18n!("tests/locales");
