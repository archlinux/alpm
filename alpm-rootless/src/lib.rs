#![doc = include_str!("../README.md")]

mod error;
mod fakeroot;
mod traits;
mod utils;

pub use error::Error;
pub use fakeroot::{FakerootBackend, FakerootOptions};
pub use traits::{RootlessBackend, RootlessOptions};
pub use utils::get_command;
