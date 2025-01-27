#![doc = include_str!("../README.md")]

mod pkginfo_v1;
pub use crate::pkginfo_v1::PkgInfoV1;

mod pkginfo_v2;
pub use crate::pkginfo_v2::PkgInfoV2;

#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "cli")]
pub mod commands;

mod error;
pub use crate::error::Error;
