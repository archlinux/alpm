#![doc = include_str!("../README.md")]

mod error;
mod fakeroot;
mod rootlesskit;
mod traits;
mod utils;

pub use error::Error;
pub use fakeroot::{FakerootBackend, FakerootOptions};
pub use rootlesskit::{
    CopyUpMode,
    Net,
    PortDriver,
    Propagation,
    ReaperAutoOption,
    RootlesskitBackend,
    RootlesskitOptions,
    Slirp4netnsAutoOption,
    SubIdSource,
};
pub use traits::{RootlessBackend, RootlessOptions};
pub use utils::{
    ConfidentialVirtualizationTechnology,
    SystemdDetectVirtContainer,
    SystemdDetectVirtOutput,
    SystemdDetectVirtVm,
    detect_virt,
    get_command,
};
