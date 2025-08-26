use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use strum::{Display as StrumDisplay, VariantArray};

/// A group lints can belong to.
///
/// A [`LintGroup`] is used to en-/disable groups of lints.
/// Each lint may belong to zero or more groups.
#[derive(
    Clone, Debug, Deserialize, PartialEq, Serialize, StrumDisplay, ValueEnum, VariantArray,
)]
#[strum(serialize_all = "lowercase")]
pub enum LintGroup {
    /// This group contains all lints that are considered fairly pedantic and/or prone to trigger
    /// false-positives.
    ///
    /// When using this group, expect to manually disable some lint rules for your context.
    Pedantic,

    /// Experimental lints that might not be 100% fine-tuned.
    ///
    /// When using this group, the newest and shiniest lints are used.
    Testing,
}
