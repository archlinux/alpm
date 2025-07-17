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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use alpm_lint_config::{LintConfiguration, LintGroup};

    use super::FilteredLintRules;
    use crate::internal_prelude::*;

    // ---- All tests below this line test the FilteredLintRules iterator ----
    //
    // The iterator is explicitly tested without the store as the store will always contain all
    // lints, meaning that the list of tested lints might change over time.
    //
    // To isolate things a bit and to make testing deterministic, we create some "MockLintRule"s
    // on which we perform the filtering.

    /// Test implementation of [`LintRule`] for unit testing.
    struct MockLintRule {
        name: &'static str,
        scope: LintScope,
        level: Level,
        groups: &'static [LintGroup],
    }

    impl LintRule for MockLintRule {
        fn name(&self) -> &'static str {
            self.name
        }

        fn scope(&self) -> LintScope {
            self.scope
        }

        fn level(&self) -> Level {
            self.level.clone()
        }

        fn groups(&self) -> &'static [LintGroup] {
            self.groups
        }

        fn run(&self, _resources: &Resources, _issues: &mut Vec<LintIssue>) -> Result<(), Error> {
            Ok(())
        }

        fn help_text(&self) -> String {
            format!("Help for {}", self.name)
        }
    }

    impl MockLintRule {
        /// Creates a mock lint rules.
        fn new_boxed(name: &'static str, scope: LintScope) -> Box<dyn LintRule> {
            Box::new(Self {
                name,
                scope,
                level: Level::Warn,
                groups: &[],
            })
        }

        /// Creates a mock lint rule with specified groups.
        fn with_groups(
            name: &'static str,
            scope: LintScope,
            groups: &'static [LintGroup],
        ) -> Box<dyn LintRule> {
            Box::new(Self {
                name,
                scope,
                level: Level::Warn,
                groups,
            })
        }
    }

    /// Helper function to assert the next rule name from a filtered iterator.
    fn next_is(filtered: &mut FilteredLintRules, expected_name: &str) {
        let (name, _) = filtered
            .next()
            .unwrap_or_else(|| panic!("Should have {expected_name}"));
        assert_eq!(name, expected_name);
    }

    /// Helper function to assert that the filtered iterator has no more rules.
    fn next_is_none(filtered: &mut FilteredLintRules) {
        assert!(filtered.next().is_none(), "Should have no more rules");
    }

    /// Creates a set of mock lint rules for testing with differing properties.
    fn create_mock_rules() -> BTreeMap<String, Box<dyn LintRule>> {
        let mut rules = BTreeMap::new();

        // Always enabled for SourceInfo
        let rule1 = MockLintRule::new_boxed("test_rule_1", LintScope::SourceInfo);
        // Always enabled for PackageBuild
        let rule2 = MockLintRule::new_boxed("test_rule_2", LintScope::PackageBuild);
        // Pedantic SourceInfo Rule
        let rule3 = MockLintRule::with_groups(
            "pedantic_rule",
            LintScope::SourceInfo,
            &[LintGroup::Pedantic],
        );
        // Testing Group SourceInfo Rule
        let rule4 =
            MockLintRule::with_groups("testing_rule", LintScope::SourceInfo, &[LintGroup::Testing]);
        // Pedantic **and** Testing groups SourceInfo Rule
        let rule5 = MockLintRule::with_groups(
            "multi_group_rule",
            LintScope::SourceInfo,
            &[LintGroup::Pedantic, LintGroup::Testing],
        );

        rules.insert(rule1.scoped_name(), rule1);
        rules.insert(rule2.scoped_name(), rule2);
        rules.insert(rule3.scoped_name(), rule3);
        rules.insert(rule4.scoped_name(), rule4);
        rules.insert(rule5.scoped_name(), rule5);

        rules
    }

    /// Ensures that filtering respects scope boundaries.
    #[test]
    fn filters_by_scope() {
        let config = LintConfiguration::default();
        let rules = create_mock_rules();
        let mut filtered = FilteredLintRules::new(&config, rules.iter(), LintScope::SourceInfo);

        // Should include only ungrouped SourceInfo scope rules
        // test_rule_1 is the only rule that's by default enabled for the SourceInfo scope.
        next_is(&mut filtered, "source_info::test_rule_1");
        next_is_none(&mut filtered);
    }

    /// Ensures that explicitly disabled rules are excluded.
    #[test]
    fn respects_disabled_rules() {
        let config = LintConfiguration {
            disabled_rules: vec!["source_info::test_rule_1".to_string()],
            ..Default::default()
        };
        let rules = create_mock_rules();
        let mut filtered = FilteredLintRules::new(&config, rules.iter(), LintScope::SourceInfo);

        // Should exclude the disabled rule.
        next_is_none(&mut filtered);
    }

    /// Ensures that explicitly enabled rules bypass group filtering.
    #[test]
    fn includes_explicitly_enabled_rules() {
        let config = LintConfiguration {
            enabled_rules: vec!["source_info::pedantic_rule".to_string()],
            groups: vec![], // No groups enabled
            ..Default::default()
        };
        let rules = create_mock_rules();
        let mut filtered = FilteredLintRules::new(&config, rules.iter(), LintScope::SourceInfo);

        // Should include the explicitly enabled pedantic rule even with no groups
        next_is(&mut filtered, "source_info::pedantic_rule");
        next_is(&mut filtered, "source_info::test_rule_1");
        next_is_none(&mut filtered);
    }

    /// Ensures that disabling rules takes precedence over enabling rules.
    #[test]
    fn disabled_rules_take_precedence() {
        let config = LintConfiguration {
            disabled_rules: vec!["source_info::test_rule_1".to_string()],
            enabled_rules: vec!["source_info::test_rule_1".to_string()],
            ..Default::default()
        };
        let rules = create_mock_rules();
        let mut filtered = FilteredLintRules::new(&config, rules.iter(), LintScope::SourceInfo);

        // Disabled rules are checked first and take precedence
        next_is_none(&mut filtered);
    }

    /// Ensures that rules with multiple groups require *ALL* groups to be enabled.
    #[test]
    fn multi_group_requires_all_groups() {
        let config = LintConfiguration {
            groups: vec![LintGroup::Pedantic], // Only one group enabled
            ..Default::default()
        };
        let rules = create_mock_rules();
        let mut filtered = FilteredLintRules::new(&config, rules.iter(), LintScope::SourceInfo);

        // Should get pedantic_rule and test_rule_1, but not multi_group_rule
        next_is(&mut filtered, "source_info::pedantic_rule");
        next_is(&mut filtered, "source_info::test_rule_1");
        next_is_none(&mut filtered);
    }

    /// Ensures that multi-group lint rules are included when all their groups are enabled.
    #[test]
    fn multi_group_included() {
        let config = LintConfiguration {
            groups: vec![LintGroup::Pedantic, LintGroup::Testing],
            ..Default::default()
        };
        let rules = create_mock_rules();
        let mut filtered = FilteredLintRules::new(&config, rules.iter(), LintScope::SourceInfo);

        // Should get all SourceInfo rules: multi_group_rule, pedantic_rule, test_rule_1,
        // testing_rule
        next_is(&mut filtered, "source_info::multi_group_rule");
        next_is(&mut filtered, "source_info::pedantic_rule");
        next_is(&mut filtered, "source_info::test_rule_1");
        next_is(&mut filtered, "source_info::testing_rule");
        next_is_none(&mut filtered);
    }

    /// Ensures that the scope hierarchy is respected in filtering.
    #[test]
    fn source_repository_scope() {
        let config = LintConfiguration::default();
        let rules = create_mock_rules();
        let mut filtered =
            FilteredLintRules::new(&config, rules.iter(), LintScope::SourceRepository);

        // SourceRepository scope should include both SourceInfo and PackageBuild rules
        // Both test_rule_1 and test_rule_2 are ungrouped and match the scope
        next_is(&mut filtered, "package_build::test_rule_2");
        next_is(&mut filtered, "source_info::test_rule_1");
        next_is_none(&mut filtered);
    }
}
