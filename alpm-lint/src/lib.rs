#![doc = include_str!("../README.md")]

use std::fmt::Display;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use strum::Display as StrumDisplay;

pub mod cli;
mod error;
pub mod issue;
pub mod lint_rules;
pub mod resources;
pub mod scope;

pub use crate::{
    error::Error,
    issue::LintIssue,
    lint_rules::{LintRule, store::LintStore},
    resources::Resources,
    scope::LintScope,
};

/// Common imports that're required for most linting rule implementations.
///
/// This is a convenience prelude module as all of these imports are used in every single lint
/// rule.
mod internal_prelude {
    pub use alpm_lint_config::{LintGroup, LintRuleConfiguration};

    pub use crate::{
        Error,
        Level,
        issue::LintIssue,
        lint_rules::LintRule,
        resources::Resources,
        scope::LintScope,
    };
}
/// Convenience re-export of [`alpm_lint_config`] types.
pub mod config {
    pub use alpm_lint_config::{
        LintConfiguration,
        LintGroup,
        LintRuleConfiguration,
        LintRuleConfigurationOption,
    };
}

/// The fully qualified name of a lint rule.
/// This is the scope of the lint rule combined with the name the rule.
/// ```
/// use alpm_lint::{ScopedName, scope::LintScope};
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
    /// Create a new instance of [`ScopedName`]
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
#[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize, StrumDisplay, ValueEnum)]
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
