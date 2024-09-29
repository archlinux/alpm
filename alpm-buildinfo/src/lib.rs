#![doc = include_str!("../README.md")]

mod buildinfo_v1;
pub use crate::buildinfo_v1::BuildInfoV1;

pub mod cli;

mod common;

mod error;
pub use crate::error::Error;
