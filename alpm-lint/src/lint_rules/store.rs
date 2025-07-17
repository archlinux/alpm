//! The store is used to access and filter all registered lints.
//!
//! Lints need to be registered manually in the `LintStore::register` function.

use std::collections::{BTreeMap, btree_map};

use alpm_lint_config::{LintConfiguration, LintRuleConfiguration};
use serde::Serialize;

use crate::{
    ScopedName,
    internal_prelude::{Level, LintGroup, LintRule, LintScope},
    lint_rules::source_info::{
        duplicate_architecture::DuplicateArchitecture,
        unsafe_checksum::UnsafeChecksum,
    },
};

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

type LintConstructor = fn(&LintRuleConfiguration) -> Box<dyn LintRule>;
type LintMap = BTreeMap<String, Box<dyn LintRule>>;

/// The [`LintStore`], which contains all available and known lint rules.
///
/// It can be further used to filter and select lints based on various criteria.
#[allow(missing_debug_implementations)]
pub struct LintStore {
    config: LintConfiguration,
    lint_constructors: Vec<LintConstructor>,
    initialized_lints: LintMap,
}

impl LintStore {
    /// Create a new [`LintStore`].
    ///
    /// This adds all known lint rules to the store.
    pub fn new(config: LintConfiguration) -> Self {
        let mut store = Self {
            config,
            lint_constructors: Vec::new(),
            initialized_lints: BTreeMap::new(),
        };
        store.register();
        store.initialize_lint_rules();

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
    fn initialize_lint_rules(&mut self) {
        // Early return if the lints are already initialized.
        if !self.initialized_lints.is_empty() {
            return;
        }

        for lint in &self.lint_constructors {
            let initialized = lint(&self.config.options);

            self.initialized_lints
                .insert(initialized.scoped_name(), initialized);
        }
    }

    /// Get the list of all available and configured lints.
    pub fn lint_rules(&self) -> &LintMap {
        &self.initialized_lints
    }

    /// Try to get a specific lint by it's scoped name.
    ///
    /// Returns `None` if no such lint exists.
    #[allow(clippy::borrowed_box)]
    pub fn lint_rule_by_name(&self, name: &ScopedName) -> Option<&Box<dyn LintRule>> {
        self.initialized_lints.get(&name.to_string())
    }

    /// Get the list of all available and configured lints.
    pub fn serializable_lint_rules(&self) -> BTreeMap<String, SerializableLintRule> {
        let mut map = BTreeMap::new();
        for (name, lint) in &self.initialized_lints {
            // Make sure that there's no duplicate key.
            // We explicitly choose a `panic` as this is considered a hard consistency error.
            if map.contains_key(name) {
                panic!("Encountered duplicate lint with name: {name}");
            }

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

    /// Get all lints given a certain lint configuration.
    ///
    /// This function filters out all lint rules that are:
    /// - not explicitly included **and**
    /// - assigned an deactivated group **or**
    /// - explicitly ignored
    pub fn filtered_lint_rules<'a>(&'a self, scope: &LintScope) -> FilteredLintRules<'a> {
        FilteredLintRules::new(&self.config, self.initialized_lints.iter(), *scope)
    }
}

/// The iterator that is returned by `LintConfiguration.initialized_lints.iter()`.
type BTreeMapRuleIter<'a> = btree_map::Iter<'a, String, Box<dyn LintRule>>;

/// Iterator that allows iterating over lints filtered by a given configuration file.
///
/// ```
/// use alpm_lint::{LintScope, LintStore, config::LintConfiguration};
///
/// // Build a default config and use it to filter all lints from the store.
/// let config = LintConfiguration::default();
/// let store = LintStore::new(config);
/// let mut iterator = store.filtered_lint_rules(&LintScope::SourceInfo);
///
/// // We get a lint
/// assert!(iterator.next().is_some())
/// ```
#[allow(missing_debug_implementations)]
pub struct FilteredLintRules<'a> {
    /// The configuration that's used to filter out lints.
    config: &'a LintConfiguration,
    /// The unfiltered iterator over all lints.
    rules_iter: BTreeMapRuleIter<'a>,
    /// The scope for which lints should be executed.
    scope: LintScope,
}

impl<'a> FilteredLintRules<'a> {
    /// Create a new `FilteredLintRules`.
    pub fn new(
        config: &'a LintConfiguration,
        rules_iter: BTreeMapRuleIter<'a>,
        scope: LintScope,
    ) -> Self {
        Self {
            config,
            rules_iter,
            scope,
        }
    }
}

impl<'a> Iterator for FilteredLintRules<'a> {
    type Item = (&'a String, &'a Box<dyn LintRule>);

    // Allow while_let on an iterator. This pattern is required to give us more control
    // `self.rules_iter`.
    #[allow(clippy::while_let_on_iterator)]
    fn next(&mut self) -> Option<Self::Item> {
        'outer: while let Some((name, rule)) = self.rules_iter.next() {
            // Check whether this rule is explicitly disabled.
            // If so immediately skip it.
            if self.config.disabled_rules.contains(name) {
                continue;
            }

            // Check whether this rule is explicitly enabled.
            // If so immediately return it.
            if self.config.enabled_rules.contains(name) {
                return Some((name, rule));
            }

            // If the groups are not empty, check whether all lint groups are enabled in the
            // configuration file. If so, the lint will be returned, otherwise skip it.
            let groups = rule.groups();
            if !groups.is_empty() {
                // Since there're very few groups, a nxm lookup is reasonable.
                for group in groups {
                    if !self.config.groups.contains(group) {
                        // A group isn't enabled, skip the rule.
                        continue 'outer;
                    }
                }
            }

            // Make sure that the selected scope includes this specific lint rule.
            let lint_rule_scope = rule.scope();
            if !self.scope.contains(&lint_rule_scope) {
                continue;
            }

            return Some((name, rule));
        }

        None
    }
}
