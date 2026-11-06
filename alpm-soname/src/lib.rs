#![doc = include_str!("../README.md")]

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;
mod error;
mod lookup;

/// Re-exports for types from the [`alpm_types`] crate which are part of the public API.
pub mod alpm_types {
    #[cfg(feature = "cli")]
    pub use alpm_types::SharedLibraryPrefix;
    pub use alpm_types::{Error, RelationOrSoname, Soname, SonameLookupDirectory, SonameV2};
}

pub use error::Error;
pub use lookup::{ElfSonames, extract_elf_sonames, find_dependencies, find_provisions};

fluent_i18n::i18n!("locales");
