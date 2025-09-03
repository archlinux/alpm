//! The trait definition and behavioral description of a lint rule.

use std::collections::BTreeMap;

use alpm_lint_config::{LintGroup, LintRuleConfigurationOptionName};

use crate::{Error, Level, LintScope, ScopedName, issue::LintIssue, resources::Resources};

/// The trait definition and behavioral description of a lint rule.
///
/// This trait that must be implemented by every available lint.
pub trait LintRule {
    /// Return the name of this linting rule.
    ///
    /// This must be a static and unique identifier.
    fn name(&self) -> &'static str;

    /// Returns the full name of this lint by combining [`LintRule::scope`] and [`LintRule::name`]
    /// as `{scope}::{name}`.
    ///
    /// # Warning
    ///
    /// Do not re-implement this. The default implementation should cover all cases.
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
    ///
    /// Lint rules are enabled by default, unless they belong to a [`LintGroup`], in which case
    /// they need to be explicitly enabled.
    fn groups(&self) -> &'static [LintGroup] {
        &[]
    }

    /// Execute the linting logic.
    ///
    /// This function receives the [`Resources`] enum, that contains all data required to run a
    /// lint.
    ///
    /// The second argument (`issues`) is the list of accumulated issues across all lints.
    /// If your lint rule encounters an issue, add it to that list.
    fn run(&self, resources: &Resources, issues: &mut Vec<LintIssue>) -> Result<(), Error>;

    /// Returns the full documentation for this lint rule, including:
    /// - A description of what the rule does
    /// - The reasoning behind its existence
    /// - An example usage
    ///
    /// Typically, you should use the [`documented`] crate to forward the doc
    /// comments from the struct that implements this trait.
    ///
    /// # Example
    /// ```rust,ignore
    /// fn documentation(&self) -> String {
    ///     DuplicateArchitecture::DOCS.into()
    /// }
    /// ```
    fn documentation(&self) -> String;

    /// Return the help text for this lint rule, which explains why they encountered this lint and
    /// how to potentially fix it.
    ///
    /// This is shown to users when they encounter this lint issue.
    fn help_text(&self) -> String;

    /// Return a list of [`LintRuleConfigurationOptionName`]s this lint rule uses to configure
    /// itself. This is necessary to reference the correct options on our `lint-config-website`.
    fn configuration_options(&self) -> &[LintRuleConfigurationOptionName] {
        &[]
    }

    /// Return extra associated links for this lint rule.
    ///
    /// The map has the structure of `BTreeMap<URL-Name, URL>`.
    ///
    /// These links will be displayed in the
    fn extra_links(&self) -> Option<BTreeMap<String, String>> {
        None
    }
}
