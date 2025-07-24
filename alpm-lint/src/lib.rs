#![doc = include_str!("../README.md")]

use std::fmt::Display;

use clap::ValueEnum;
use serde::Serialize;
use strum::Display as StrumDisplay;

pub mod cli;
mod error;
pub mod lint;
pub mod lint_rules;
pub mod resources;
pub mod scope;

pub use error::Error;
pub use scope::LintScope;

/// Common imports that're required for most linting rule implementations.
pub mod prelude {
    pub use alpm_lint_config::LintRuleConfiguration;

    pub use crate::{Level, LintGroup, lint::LintRule, resources::Resources, scope::LintScope};
}

/// The fully qualified name of a lint rule.
/// This is the scope of the lint rule combined with the name the rule.
/// ```
/// use alpm_lint::{LintScope, ScopedName};
///
/// let name = ScopedName::new(LintScope::SourceRepository, "my_rule");
/// assert_eq!("source_repo::my_rule", name.to_string());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ScopedName {
    scope: LintScope,
    name: &'static str,
}

impl ScopedName {
    /// Create a new instance of [`LintName`]
    pub fn new(scope: LintScope, name: &'static str) -> Self {
        Self { scope, name }
    }
}

impl Display for ScopedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.scope, self.name)
    }
}

/// [`Level`] is used to determine how severe a lint is considered.
///
/// The level of a lint can be overwritten via CLI flags and configuration files.
#[derive(Clone, Debug, PartialEq, StrumDisplay, ValueEnum, Serialize)]
pub enum Level {
    /// Lints with this level are considered to always be bad practices or severe errors.
    Deny,
    /// Lints with this level hint towards mistakes or misconfigurations that are correct with a
    /// high degree of certainty.
    Warn,
    /// Lints with this level suggest best-practices that don't need to be followed.
    Suggest,
}

/// The groups a lint can belong to.
///
/// A lint may belong to zero or multiple groups.
/// [`LintGroup`]s further allow to en-/disable groups of lints.
#[derive(Clone, Debug, PartialEq, StrumDisplay, ValueEnum, Serialize)]
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
