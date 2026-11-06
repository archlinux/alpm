#![doc = include_str!("../README.md")]

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;
mod error;
mod lookup;

pub use error::Error;
pub use lookup::{ElfSonames, extract_elf_sonames, find_dependencies, find_provisions};

fluent_i18n::i18n!("locales");
