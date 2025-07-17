#![doc = include_str!("../README.md")]
mod lint_rule;

use clap::ValueEnum;
pub use lint_rule::{
    LintRuleConfiguration,
    LintRuleConfigurationOption,
    LintRuleConfigurationOptionName,
};
use serde::{Deserialize, Serialize};
use strum::Display as StrumDisplay;

/// The groups a lint can belong to.
///
/// A lint may belong to zero or multiple groups.
/// [`LintGroup`]s further allow to en-/disable groups of lints.
// Developer Note:
// We implement ValueEnum to allow good CLI handling in `alpm-lint`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, StrumDisplay, ValueEnum)]
pub enum LintGroup {
    /// This group contains all lints that're fairly pedantic and/or prone for false-positives.
    ///
    /// Expect to manually disable some rules for your package when enabling this group.
    Pedantic,

    /// Experimental lints that might not be 100% fine-tuned.
    ///
    /// Enabling these is much appreciated and will give you the newest and shiniest lints.
    Testing,
}

/// All configuration options to en-/disable indidual lint rules or groups and categories of lint
/// rules, as well as configuration of lint rule behavior.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LintConfiguration {
    /// All options that can be used to configure various lint rules.
    pub options: LintRuleConfiguration,
    /// All non-default groups that are additionally enabled.
    pub groups: Vec<LintGroup>,
    /// A list of lint rules that are explicitly disabled.
    pub disabled_rules: Vec<String>,
    /// A list of lint rules that are explicitly enabled.
    pub enabled_rules: Vec<String>,
}
