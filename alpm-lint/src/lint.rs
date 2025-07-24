//! The trait definition and behavioral description of a lint.

use alpm_lint_config::LintRuleConfigurationOptionName;

use crate::{Level, LintGroup, LintScope, ScopedName, resources::Resources};

/// The main trait that must be implemented by every available lint.
pub trait LintRule {
    /// Return the name of this linting rule.
    ///
    /// This must be a static and unique identifier.
    ///
    /// Each lint should have a `const pub fn const_name` function, which is then called by this
    /// function. `const fn` functions cannot be part of traits yet, hence this workaround.
    //
    // Example for such an implementation:
    // ```rs
    // pub struct MyLint {}
    //
    // impl Lint for MyLint {
    // //...
    //     fn name() -> &'static str {
    //         MyLint::const_name()
    //     }
    // //...
    // }
    //
    // impl MyLint {
    //     pub const fn const_name() -> &'static str {
    //         "Test"
    //     }
    // }
    // ```
    fn name(&self) -> &'static str;

    /// The full name of this lint.
    // **Don't re-implement this. The default implementation should cover all cases.
    fn scoped_name(&self) -> String {
        ScopedName::new(self.scope(), self.name()).to_string()
    }

    /// Return the scope of this lint rule.
    ///
    /// This is used to select groups of lints based on the performed linting operation.
    /// Linting scopes can also be fully dis-/enabled via configuration files.
    fn scope(&self) -> LintScope;

    /// The severity level of this linting rule.
    ///
    /// This is used to determine what lint messages should be shown based on CLI flags and
    /// configuration.
    ///
    /// The default level is to [`Level::Warn`] the user about the rule.
    fn level(&self) -> Level {
        Level::Warn
    }

    /// Return the static list groups this lint rule belongs to.
    fn groups(&self) -> &'static [LintGroup] {
        &[]
    }

    /// Execute the linting logic.
    ///
    /// This gets passed in the [`Resources`] enum, which provides the resources.
    fn run(&self, resource: Resources);

    /// Return the help text for this lint rule, which explains what a lint does and the rationale
    /// behind it.
    fn help_text(&self) -> &'static str;

    /// Return a list of [`LintRuleConfigurationOptionName`]s this lint rule uses to configure
    /// itself.
    fn configuration_options(&self) -> &[LintRuleConfigurationOptionName] {
        &[]
    }
}
