//! Access and filtering to all registered lints.
//!
//! # Note
//!
//! All lints need to be registered in the private `LintStore::register` function when adding a new
//! lint rule!

use std::{
    collections::{BTreeMap, btree_map},
    fmt,
};

use alpm_lint_config::{LintConfiguration, LintRuleConfiguration, LintRuleConfigurationOptionName};
use serde::Serialize;

use crate::{
    ScopedName,
    internal_prelude::{Level, LintGroup, LintRule, LintScope},
    lint_rules::source_info::{
        duplicate_architecture::DuplicateArchitecture,
        invalid_spdx_license::NotSPDX,
        no_architecture::NoArchitecture,
        openpgp_key_id::OpenPGPKeyId,
        undefined_architecture::UndefinedArchitecture,
        unknown_architecture::UnknownArchitecture,
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
    scoped_name: String,
    scope: LintScope,
    level: Level,
    groups: Vec<LintGroup>,
    documentation: String,
    option_names: Vec<String>,
}

/// The constructor function type that is used by each implementation of [`LintRule`].
///
/// E.g. [`DuplicateArchitecture::new_boxed`]. These constructors are saved in the [`LintStore`].
type LintConstructor = fn(&LintRuleConfiguration) -> Box<dyn LintRule>;

/// A map of lint rule name and generic [`LintRule`] implementations.
///
/// Used in [`LintStore`] to describe tuples of lint rule names and [`LintRule`] implementations.
type LintMap = BTreeMap<String, Box<dyn LintRule>>;

/// The [`LintStore`], which contains all available and known lint rules.
///
/// It can be used to further filter and select lints based on various criteria.
pub struct LintStore {
    config: LintConfiguration,
    lint_constructors: Vec<LintConstructor>,
    initialized_lints: LintMap,
}

impl fmt::Debug for LintStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LintStore")
            .field("config", &self.config)
            .field("lint_constructors", &"Vec<LintConstructor>")
            .field("initialized_lints", &"LintMap")
            .finish()
    }
}

impl LintStore {
    /// Creates a new [`LintStore`].
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

    /// Registers all lints that are made available in the store.
    ///
    /// # Note
    ///
    /// New lints must be specified in this function!
    fn register(&mut self) {
        // **IMPORTANT** NOTE: ⚠️⚠️⚠️⚠️⚠️⚠️⚠️⚠️
        // When you edit this, please sort the array while at it :)
        // Much appreciated!
        self.lint_constructors = vec![
            DuplicateArchitecture::new_boxed,
            NoArchitecture::new_boxed,
            NotSPDX::new_boxed,
            OpenPGPKeyId::new_boxed,
            UndefinedArchitecture::new_boxed,
            UnknownArchitecture::new_boxed,
            UnsafeChecksum::new_boxed,
        ];
    }

    /// Initializes and configures all linting rules.
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

    /// Returns a reference to the map of all available and configured lint rules.
    pub fn lint_rules(&self) -> &LintMap {
        &self.initialized_lints
    }

    /// Returns a specific lint rule by its scoped name.
    ///
    /// Returns [`None`] if no lint rule with a matching `name` exists.
    // False positive lint warning on the return type.
    #[allow(clippy::borrowed_box)]
    pub fn lint_rule_by_name(&self, name: &ScopedName) -> Option<&Box<dyn LintRule>> {
        self.initialized_lints.get(&name.to_string())
    }

    /// Returns a map of all available and configured lint rules as [`SerializableLintRule`].
    pub fn serializable_lint_rules(&self) -> BTreeMap<String, SerializableLintRule> {
        let mut map = BTreeMap::new();
        for (scoped_name, lint) in &self.initialized_lints {
            // Make sure that there's no duplicate key.
            // We explicitly choose a `panic` as this is considered a hard consistency error.
            //
            // This is also covered by a test case, so it should really never happen in a release.
            if map.contains_key(scoped_name) {
                panic!("Encountered duplicate lint with name: {scoped_name}");
            }

            map.insert(
                scoped_name.clone(),
                SerializableLintRule {
                    name: lint.name().to_string(),
                    scoped_name: scoped_name.clone(),
                    scope: lint.scope(),
                    level: lint.level(),
                    groups: lint.groups().to_vec(),
                    documentation: lint.documentation().to_string(),
                    option_names: lint
                        .configuration_options()
                        .iter()
                        .map(LintRuleConfigurationOptionName::to_string)
                        .collect(),
                },
            );
        }

        map
    }

    /// Returns lint rules that match a filter consisting of [`LintScope`] and [`Level`].
    ///
    /// This function filters out all lint rules that are not explicitly included **and**
    /// - assigned to a deactivated group,
    /// - **or** have a level above the max_level,
    /// - **or** are explicitly ignored.
    pub fn filtered_lint_rules<'a>(
        &'a self,
        scope: &LintScope,
        max_level: Level,
    ) -> FilteredLintRules<'a> {
        FilteredLintRules::new(
            &self.config,
            self.initialized_lints.iter(),
            *scope,
            max_level,
        )
    }
}

/// The iterator that is returned by `LintConfiguration.initialized_lints.iter()`.
type BTreeMapRuleIter<'a> = btree_map::Iter<'a, String, Box<dyn LintRule>>;

/// An Iterator that allows iterating over lint rules filtered by a specific configuration file.
///
/// # Examples
///
/// ```
/// use alpm_lint::{Level, LintScope, LintStore, config::LintConfiguration};
///
/// // Build a default config and use it to filter all lints from the store.
/// let config = LintConfiguration::default();
/// let store = LintStore::new(config);
/// let mut iterator = store.filtered_lint_rules(&LintScope::SourceInfo, Level::Suggest);
///
/// // We get a lint
/// assert!(iterator.next().is_some())
/// ```
pub struct FilteredLintRules<'a> {
    /// The configuration used for filtering lint rules.
    config: &'a LintConfiguration,
    /// The unfiltered iterator over all lint rules.
    rules_iter: BTreeMapRuleIter<'a>,
    /// The scope in which lint rules should be.
    scope: LintScope,
    /// The lowest [`Level`] from which lint rules are considered.
    min_level: Level,
}

impl<'a> FilteredLintRules<'a> {
    /// Creates a new [`FilteredLintRules`].
    pub fn new(
        config: &'a LintConfiguration,
        rules_iter: BTreeMapRuleIter<'a>,
        scope: LintScope,
        min_level: Level,
    ) -> Self {
        Self {
            config,
            rules_iter,
            scope,
            min_level,
        }
    }
}

impl std::fmt::Debug for FilteredLintRules<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilteredLintRules")
            .field("config", &self.config)
            .field("scope", &self.scope)
            .field("min_level", &self.min_level)
            .finish()
    }
}

impl<'a> Iterator for FilteredLintRules<'a> {
    type Item = (&'a String, &'a Box<dyn LintRule>);

    // Allow while_let on an iterator. This pattern is required to give us more control
    // over `self.rules_iter`.
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

            // Skip any lint rules that're below the specified severity level threshold.
            // The higher the number, the less important the Level.
            // (e.g. Error=1, Suggest=4).
            if rule.level() as isize > self.min_level as isize {
                continue;
            }

            // If the groups are not empty, check whether all lint groups are enabled in the
            // configuration file. If so, the lint will be returned, otherwise skip it.
            let groups = rule.groups();
            if !groups.is_empty() {
                // As there are very few groups, an `n * m` lookup is reasonable.
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
    use super::*;

    /// Unit tests for the LintStore itself
    mod lint_store {
        use std::collections::HashSet;

        use alpm_lint_config::{LintConfiguration, LintRuleConfiguration};
        use testresult::TestResult;

        use super::LintStore;

        /// Ensures that no two lint rules have the same scoped name.
        ///
        /// This is extremely important as to prevent naming conflicts and to ensure that each lint
        /// rule has a unique identifier.
        #[test]
        fn no_duplicate_scoped_names() {
            let store = LintStore::new(LintConfiguration::default());
            let config = LintRuleConfiguration::default();

            // Test the raw constructors for duplicate scoped names
            let constructors = store.lint_constructors;
            let mut scoped_names = HashSet::<String>::new();

            for constructor in constructors {
                let lint_rule = constructor(&config);
                let scoped_name = lint_rule.scoped_name();

                if scoped_names.contains(&scoped_name) {
                    panic!("Found duplicate scoped lint rule name: {scoped_name}");
                }
                scoped_names.insert(scoped_name);
            }
        }

        /// Ensures that all lint rule names only consist of lower-case alphanumerics or
        /// underscores.
        #[test]
        fn lowercase_alphanum_underscore_names() -> TestResult {
            let store = LintStore::new(LintConfiguration::default());
            let config = LintRuleConfiguration::default();

            for constructor in store.lint_constructors {
                let lint_rule = constructor(&config);
                let name = lint_rule.name();

                let is_valid = name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');

                if !is_valid {
                    let scoped_name = lint_rule.scoped_name();
                    panic!(
                        "Found lint rule name with invalid character: '{scoped_name}'
Lint rule names are only allowed to consist of lowercase alphanumeric characters and underscores."
                    );
                }
            }

            Ok(())
        }
    }

    /// Tests for the the FilteredLintRules iterator
    mod filtered_lint_rules {
        use std::collections::BTreeMap;

        use alpm_lint_config::{LintConfiguration, LintGroup};

        use super::FilteredLintRules;
        use crate::internal_prelude::*;
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
                self.level
            }

            fn groups(&self) -> &'static [LintGroup] {
                self.groups
            }

            fn run(
                &self,
                _resources: &Resources,
                _issues: &mut Vec<LintIssue>,
            ) -> Result<(), Error> {
                Ok(())
            }

            fn documentation(&self) -> String {
                format!("Documentation for {}", self.name)
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

            /// Creates a mock lint rule with a specific level.
            fn with_level(name: &'static str, scope: LintScope, level: Level) -> Box<dyn LintRule> {
                Box::new(Self {
                    name,
                    scope,
                    level,
                    groups: &[],
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
            let rule4 = MockLintRule::with_groups(
                "testing_rule",
                LintScope::SourceInfo,
                &[LintGroup::Testing],
            );
            // Pedantic **and** Testing groups SourceInfo Rule
            let rule5 = MockLintRule::with_groups(
                "multi_group_rule",
                LintScope::SourceInfo,
                &[LintGroup::Pedantic, LintGroup::Testing],
            );
            let rule6 = MockLintRule::with_level("with_error", LintScope::SourceInfo, Level::Error);

            rules.insert(rule1.scoped_name(), rule1);
            rules.insert(rule2.scoped_name(), rule2);
            rules.insert(rule3.scoped_name(), rule3);
            rules.insert(rule4.scoped_name(), rule4);
            rules.insert(rule5.scoped_name(), rule5);
            rules.insert(rule6.scoped_name(), rule6);

            rules
        }

        /// Ensures that filtering respects scope boundaries.
        #[test]
        fn filters_by_scope() {
            let config = LintConfiguration::default();
            let rules = create_mock_rules();
            let mut filtered = FilteredLintRules::new(
                &config,
                rules.iter(),
                LintScope::SourceInfo,
                Level::Suggest,
            );

            // Should include only ungrouped SourceInfo scope rules
            // test_rule_1 is the only rule that's by default enabled for the SourceInfo scope.
            next_is(&mut filtered, "source_info::test_rule_1");
            next_is(&mut filtered, "source_info::with_error");
            next_is_none(&mut filtered);
        }

        /// Ensures that explicitly disabled rules are excluded.
        #[test]
        fn respects_disabled_rules() {
            let config = LintConfiguration {
                disabled_rules: vec![
                    "source_info::test_rule_1".to_string(),
                    "source_info::with_error".to_string(),
                ],
                ..Default::default()
            };
            let rules = create_mock_rules();
            let mut filtered = FilteredLintRules::new(
                &config,
                rules.iter(),
                LintScope::SourceInfo,
                Level::Suggest,
            );

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
            let mut filtered = FilteredLintRules::new(
                &config,
                rules.iter(),
                LintScope::SourceInfo,
                Level::Suggest,
            );

            // Should include the explicitly enabled pedantic rule even with no groups
            next_is(&mut filtered, "source_info::pedantic_rule");
            next_is(&mut filtered, "source_info::test_rule_1");
            next_is(&mut filtered, "source_info::with_error");
            next_is_none(&mut filtered);
        }

        /// Ensures that disabling rules takes precedence over enabling rules.
        #[test]
        fn disabled_rules_take_precedence() {
            let config = LintConfiguration {
                disabled_rules: vec![
                    "source_info::test_rule_1".to_string(),
                    "source_info::with_error".to_string(),
                ],
                enabled_rules: vec!["source_info::test_rule_1".to_string()],
                ..Default::default()
            };
            let rules = create_mock_rules();
            let mut filtered = FilteredLintRules::new(
                &config,
                rules.iter(),
                LintScope::SourceInfo,
                Level::Suggest,
            );

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
            let mut filtered = FilteredLintRules::new(
                &config,
                rules.iter(),
                LintScope::SourceInfo,
                Level::Suggest,
            );

            // Should get pedantic_rule and test_rule_1, but not multi_group_rule
            next_is(&mut filtered, "source_info::pedantic_rule");
            next_is(&mut filtered, "source_info::test_rule_1");
            next_is(&mut filtered, "source_info::with_error");
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
            let mut filtered = FilteredLintRules::new(
                &config,
                rules.iter(),
                LintScope::SourceInfo,
                Level::Suggest,
            );

            // Should get all SourceInfo rules: multi_group_rule, pedantic_rule, test_rule_1,
            // testing_rule
            next_is(&mut filtered, "source_info::multi_group_rule");
            next_is(&mut filtered, "source_info::pedantic_rule");
            next_is(&mut filtered, "source_info::test_rule_1");
            next_is(&mut filtered, "source_info::testing_rule");
            next_is(&mut filtered, "source_info::with_error");
            next_is_none(&mut filtered);
        }

        /// Ensures that the scope hierarchy is respected in filtering.
        #[test]
        fn source_repository_scope() {
            let config = LintConfiguration::default();
            let rules = create_mock_rules();
            let mut filtered = FilteredLintRules::new(
                &config,
                rules.iter(),
                LintScope::SourceRepository,
                Level::Suggest,
            );

            // SourceRepository scope should include both SourceInfo and PackageBuild rules
            // Both test_rule_1 and test_rule_2 are ungrouped and match the scope
            next_is(&mut filtered, "package_build::test_rule_2");
            next_is(&mut filtered, "source_info::test_rule_1");
            next_is(&mut filtered, "source_info::with_error");
            next_is_none(&mut filtered);
        }

        /// Ensures that rules are filtered by minimum level threshold.
        #[test]
        fn filters_by_level() {
            let config = LintConfiguration::default();
            let rules = create_mock_rules();

            // Test with Error level threshold
            let mut filtered =
                FilteredLintRules::new(&config, rules.iter(), LintScope::SourceInfo, Level::Error);
            next_is(&mut filtered, "source_info::with_error");
            next_is_none(&mut filtered);
        }
    }
}
