//! The trait definition and behavioral description of a lint rule.

use std::collections::BTreeMap;

use alpm_lint_config::{LintGroup, LintRuleConfigurationOptionName};

use crate::{Error, Level, LintScope, ScopedName, issue::LintIssue, resources::Resources};

/// The trait definition and behavioral description of a lint rule.
///
/// This trait must be implemented by every available lint.
pub trait LintRule {
    /// Returns the name of this lint rule.
    ///
    /// # Note
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

    /// The severity level of this lint rule.
    ///
    /// This is used to determine what lint messages should be shown to the user based on CLI flags
    /// and configuration.
    ///
    /// # Note
    ///
    /// The default level is [`Level::Warn`].
    fn level(&self) -> Level {
        Level::Warn
    }

    /// Returns the static lint groups this lint rule belongs to.
    ///
    /// Lint rules can be in zero or more lint groups.
    /// Each lint rule is considered "enabled" by default, unless it belongs to a [`LintGroup`].
    /// Lint rules that are part of one or more lint groups need to be enabled explicitly.
    fn groups(&self) -> &'static [LintGroup] {
        &[]
    }

    /// Executes the linting logic and appends to list of accumulated issues.
    ///
    /// This method accepts the [`Resources`] enum, that contains all data required to run a
    /// lint.
    ///
    /// The second argument (`issues`) is the list of accumulated issues across all lints.
    /// If your lint rule encounters an issue, add it to that list.
    fn run(&self, resources: &Resources, issues: &mut Vec<LintIssue>) -> Result<(), Error>;

    /// Returns the full documentation for this lint rule.
    ///
    /// This includes:
    ///
    /// - a description of what the rule does,
    /// - the reasoning behind its existence,
    /// - and example usage.
    ///
    /// Typically, you should use the [`documented`] crate to forward the doc
    /// comments from the struct that implements this trait.
    ///
    /// # Examples
    /// ```rust,ignore
    /// fn documentation(&self) -> String {
    ///     DuplicateArchitecture::DOCS.into()
    /// }
    /// ```
    fn documentation(&self) -> String;

    /// Returns the help text for this lint rule.
    ///
    /// The help text explains why this lint is encountered and
    /// how to potentially fix it.
    ///
    /// This information is shown to users when they encounter an issue with this lint.
    fn help_text(&self) -> String;

    /// Returns a map of configuration options used by this lint rule.
    ///
    /// The returned map of [`LintRuleConfigurationOptionName`] represents the options that this
    /// lint rule uses to configure itself.
    /// This is necessary to reference the correct options on the central `lint-config-website`.
    fn configuration_options(&self) -> &[LintRuleConfigurationOptionName] {
        &[]
    }

    /// Returns a map of additional associated links for this lint rule.
    ///
    /// The map provides the links as tuples of URL name and URL (i.e. `BTreeMap<URL-Name, URL>`).
    ///
    /// These links are displayed to users on the central `lint-config-website` as additional
    /// information.
    fn extra_links(&self) -> Option<BTreeMap<String, String>> {
        None
    }
}
