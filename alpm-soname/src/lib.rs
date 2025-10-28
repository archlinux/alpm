#![doc = include_str!("../README.md")]

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;

mod lookup;
pub use lookup::{ElfSonames, extract_elf_sonames, find_dependencies, find_provisions};

mod error;
pub use error::Error;
