#![doc = include_str!("../README.md")]

pub(crate) mod bridge;
#[cfg(feature = "cli")]
pub mod cli;
pub(crate) mod error;

pub use bridge::{error::BridgeError, source_info::source_info_v1_from_pkgbuild};
#[cfg(feature = "bridge_output")]
pub use bridge::{
    parser::{BridgeOutput, ClearableValue, Keyword, RawPackageName, Value},
    run_bridge_script,
};
pub use error::Error;
