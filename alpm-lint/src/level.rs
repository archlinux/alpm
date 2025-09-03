use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use strum::{Display as StrumDisplay, VariantArray};

/// Represents the severity level of a lint.
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
    /// Lint rules leading to errors.
    ///
    /// Lint rules with this severity level are used when encountering broken or invalid data.
    Error = 1,
    /// Lint rules leading to denials.
    ///
    /// Lint rules with this severity level always represent bad practices or severe errors.
    Deny = 2,
    /// Lint rules leading to warnings.
    ///
    /// Lint rules with this severity level indicate a mistake or misconfiguration and are
    /// considered to be detectable with a high degree of certainty.
    Warn = 3,
    /// Lint rules leading to suggestions.
    ///
    /// Lint rules with this severity level are used to suggest best practices, which when not
    /// followed do not lead to functional issues.
    Suggest = 4,
}
