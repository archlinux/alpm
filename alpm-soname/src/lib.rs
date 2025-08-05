#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "cli")]
pub mod commands;

mod autodeps;
pub use autodeps::{Autodeps, AutodepsOptions};

mod lookup;
pub use lookup::{find_dependencies, find_provisions};

mod error;
pub use error::Error;
