#![doc = include_str!("../README.md")]

mod error;
mod fakeroot;
mod traits;
mod utils;

pub use error::Error;
pub use fakeroot::{FakerootBackend, FakerootOptions};
pub use traits::{RootlessBackend, RootlessOptions};
pub use utils::{
    ConfidentialVirtualizationTechnology,
    SystemdDetectVirtContainer,
    SystemdDetectVirtOutput,
    SystemdDetectVirtVm,
    detect_virt,
    get_command,
};
