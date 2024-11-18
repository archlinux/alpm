#![doc = include_str!("../README.md")]

mod buildinfo_v1;
pub use crate::buildinfo_v1::BuildInfoV1;

mod buildinfo_v2;
pub use crate::buildinfo_v2::BuildInfoV2;

pub mod cli;

mod error;
pub mod schema;
pub use crate::error::Error;
