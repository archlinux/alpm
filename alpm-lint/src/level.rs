use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use strum::{Display as StrumDisplay, VariantArray};

/// [`Level`] is used to determine how severe a lint is considered.
///
/// The level of a lint can be overwritten via CLI flags and configuration files.
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    PartialEq,
    PartialOrd,
    Serialize,
    StrumDisplay,
    ValueEnum,
    VariantArray,
)]
#[strum(serialize_all = "lowercase")]
pub enum Level {
    /// Error type lints are always forbidden and should be used when broken or invalid data is
    /// encountered.
    Error = 1,
    /// Lints with this level are considered to always be bad practices or severe errors.
    Deny = 2,
    /// Lints with this level hint towards mistakes or misconfigurations that are correct with a
    /// high degree of certainty.
    Warn = 3,
    /// Lints with this level suggest best-practices that don't need to be followed.
    Suggest = 4,
}
