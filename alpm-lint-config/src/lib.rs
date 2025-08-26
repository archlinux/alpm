#![doc = include_str!("../README.md")]

mod error;
mod group;
mod lint_config;
mod lint_rule_config;

pub use error::Error;
pub use group::LintGroup;
pub use lint_config::LintConfiguration;
pub use lint_rule_config::{
    LintRuleConfiguration,
    LintRuleConfigurationOption,
    LintRuleConfigurationOptionName,
};
