#![doc = include_str!("../README.md")]
#![cfg_attr(test, expect(clippy::panic))]

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;
mod error;
pub mod issue;
mod level;
pub mod lint_rules;
mod resources;
mod rule;
mod scope;
mod utils;

/// Re-exports for types from the [`alpm_types`] crate which are part of the public API.
pub mod alpm_types {
    pub use alpm_types::SystemArchitecture;
    pub(crate) use alpm_types::{
        Architecture,
        Architectures,
        ChecksumAlgorithm,
        MetadataFileName,
        OpenPGPIdentifier,
        PKGBUILD_FILE_NAME,
        SRCINFO_FILE_NAME,
    };
}

/// Common imports that're required for most linting rule implementations.
///
/// This is a convenience prelude module as pretty much all of these imports are used in every
/// single lint rule.
#[allow(unused_imports)]
mod internal_prelude {
    pub use alpm_lint_config::{LintGroup, LintRuleConfiguration};

    pub use crate::{
        Error,
        issue::LintIssue,
        level::Level,
        resources::Resources,
        rule::LintRule,
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

pub use crate::{
    error::Error,
    level::Level,
    lint_rules::store::LintStore,
    resources::Resources,
    rule::LintRule,
    scope::{LintScope, ScopedName},
};
