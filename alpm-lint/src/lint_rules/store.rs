//! The store is used to access and filter all registered lints.
//!
//! Lints need to be registered manually in the [`LintStore::register`] function.

use std::collections::BTreeMap;

use alpm_lint_config::LintRuleConfiguration;
use serde::Serialize;

use crate::{
    Level,
    LintGroup,
    LintScope,
    lint::LintRule,
    lint_rules::source_info::{
        duplicate_architecture::DuplicateArchitecture,
        unsafe_checksum::UnsafeChecksum,
    },
};

type LintConstructor = fn(&LintRuleConfiguration) -> Box<dyn LintRule>;
type LintMap = BTreeMap<String, Box<dyn LintRule>>;

/// The [`LintStore`], which contains all available and known lint rules.
///
/// It can be further used to filter and select lints based on various criteria.
#[allow(missing_debug_implementations)]
pub struct LintStore {
    lint_constructors: Vec<LintConstructor>,
    initialized_lints: LintMap,
}

impl LintStore {
    /// Create a new [`LintStore`].
    ///
    /// This adds all known lint rules to the store.
    pub fn new(config: &LintRuleConfiguration) -> Self {
        let mut store = Self {
            lint_constructors: Vec::new(),
            initialized_lints: BTreeMap::new(),
        };
        store.register();
        store.initialize_lint_rules(config);

        store
    }

    /// Register all lints that exist in the store.
    ///
    /// New lints must be manually added to this function!
    fn register(&mut self) {
        self.lint_constructors = vec![DuplicateArchitecture::new_boxed, UnsafeChecksum::new_boxed];
    }

    /// Initialize and configure all linting rules.
    ///
    /// This function instantly returns if the lints have already been initialized.
    fn initialize_lint_rules(&mut self, config: &LintRuleConfiguration) {
        // Early return if the lints are already initialialized.
        if !self.initialized_lints.is_empty() {
            return;
        }

        for lint in &self.lint_constructors {
            let initialized = lint(config);
            self.initialized_lints
                .insert(initialized.scoped_name(), initialized);
        }
    }

    /// Get the list of all available and configured lints.
    pub fn lint_rules(&self) -> &LintMap {
        &self.initialized_lints
    }

    /// Get the list of all available and configured lints.
    pub fn serializable_lint_rules(&self) -> BTreeMap<String, SerializableLintRule> {
        let mut map = BTreeMap::new();
        for (name, lint) in &self.initialized_lints {
            map.insert(
                name.clone(),
                SerializableLintRule {
                    name: name.clone(),
                    scope: lint.scope(),
                    level: lint.level(),
                    groups: lint.groups().to_vec(),
                    doc: lint.help_text().to_string(),
                },
            );
        }

        map
    }
}

/// The data representation of a singular lint rule.
///
/// This is used to expose lints via the CLI so that the lints can be used in website generation or
/// for development integration.
#[derive(Clone, Debug, Serialize)]
pub struct SerializableLintRule {
    name: String,
    scope: LintScope,
    level: Level,
    groups: Vec<LintGroup>,
    doc: String,
}
