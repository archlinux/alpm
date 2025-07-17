# Architecture Guide for alpm-lint

The `alpm-lint` framework is a comprehensive, extensible linting system for Arch Linux Package Management (ALPM) files.
This document explains how the framework is architected, how its components interact, and provides a detailed walkthrough of the linting pipeline.

## Overview

The framework follows a fairly modular design that tries to separate concerns as well as possible:
Tasks done by this framework include:

1. Scope Detection - Automatically determines what type of files/data to lint based on a given path.
1. Resource Gathering - Parsing and loading of relevant data files.
1. Lint Rule Selection - Filters available rules based on scope, CLI options and configuration.
1. Execution - Runs selected lints and collects issues
1. Output - Formats and displays results

Additionally, `alpm-lint` provides first class support for external tool integration for multiple output formats.
This is done by exposing all data, such as lints rule and configuration option specifications as structured data, as well as a structured data representation for issues found during linting.

## Core Components

### Lint Scopes

The [`LintScope`] enum defines the context in which lint rules operate. It has a hierarchical structure:
Check the official documentation to see the different scope variants and their usecases.

The `LintScope::detect` function takes care of determining the current scope, based on a given path and the files that are found at this path.

### Resources

The [`Resources`] enum provides typed access to parsed file data for each scope.

Since it contains data for a respective scope, it has the same variants like [`LintScope`], but each variant is structural and contains the respective data required for linting that scope.

The `Resources::gather` function is responsible for reading files from disk for a given scope from a given path.
This function calls all `alpm-*` parsers, as well as.

### `LintRule` Trait

All lint rules implement the [`LintRule`] trait. This allows the [`LintStore`] to handle all lint rules generically via that trait interface.

Here's a rough overview of the [`LintRule`] trait. Look at the official documentation to read the more detailed explanations.

```rust
pub trait LintRule {
    // Unique identifier inside of the current scope.
    fn name(&self) -> &'static str;
    // To which scope this rule applies to
    fn scope(&self) -> LintScope;
    // Severity (Error/Deny/Warn/Suggest)
    fn level(&self) -> Level;
    // Any groups that must be enabled if this lint rule isn't enabled by default.
    fn groups(&self) -> &'static [LintGroup];
    // Text that's shown to users when this lint is encountered
    fn help_text(&self) -> &'static str;
    fn run(&self, resources: &Resources, issues: &mut Vec<LintIssue>) -> Result<(), Error>;
    // Back-reference to any config options used by this lint
    fn configuration_options(&self) -> &[LintRuleConfigurationOptionName];
}
```

To see how a specific [`LintRule`] implementation would look like, check out the [CONTRIBUTING.md](../CONTRIBUTING.md) document.

### Lint Store

The [`LintStore`] acts as the registry and factory for all lint rules:

```rust
impl LintStore {
    // Initialize the store and all rules with the provided configuration.
    pub fn new(config: LintConfiguration) -> Self
    // Get all applicable rules for the provided configuration and scope.
    pub fn filtered_lint_rules(&self, scope: &LintScope) -> FilteredLintRules 
    // Helper function to retrieve a specific lint rule by it's global scoped name.
    pub fn lint_rule_by_name(&self, name: &ScopedName) -> Option<&Box<dyn LintRule>>
}
```

Note that new lint rules need to be manually added to the `LintStore::register` function, otherwise they won't show up.
The returned [`FilteredLintRules`] iterator is the struct that applies the configuration and scope-based filtering of the rules.

### Configuration System

The configuration structs and enums are contained in the [`alpm-lint-config`] crate.

The whole configuration system is split across two main types, [`LintConfiguration`] and [`LintRuleConfiguration`].

- [`LintConfiguration`] is used to en/disable lints or groups of lints.
- [`LintRuleConfiguration`] is used to change how certain lint rules behave.

The [`LintRuleConfiguration`] is a bit special, as it's created via the [`linting_config`] macro.
This macro performs allows us to consistently several things while maintaining consistency between the generated code:

- The actual [`LintRuleConfiguration`] struct with all its fields and a `Default` implementation.
- Introspection-like output of all options via `LintRuleConfiguration::configuration_options` for documentation generation.

#### Lint Groups

[`LintGroup`] enum defines categories like `Pedantic` and `Testing`:

- Some lints aren't enabled by default, such as pedantic lints that are prone for false-positives
- These are assigned to groups, which can then be included by users

**[`Level`]**:

- The [`Level`] enum specifies severity: `Error`, `Deny`, `Warn`, `Suggest`  
- Used to differentiate between best-practices, suggestions, potentially critical things and straight up errors

## Linting Pipeline Walkthrough

The `check` function in `alpm-lint/src/commands.rs` is a good example of how a complete linting pipeline would look like:

1. Get either the provided path or fallback to the current working directory.
    ```rust
    let path = match path {
        Some(path) => path,
        None => current_dir()?,  // Use CWD if no path provided
    };
    ```
1. Determine the scope based on that path. Fails if it cannot find any expected files.
    ```rust
    let scope = match scope {
        Some(scope) => scope,
        None => LintScope::detect(&path)?,
    };
    ```
1. Load the actual data from disk, based on the path and scope.
    ```rust
    let resources = Resources::gather(&path, scope)?;
    ```

1. Initialize and configure all lints via the [`LintStore`]. Then get all lints that match the [`LintConfiguration`] and the current scope.
    ```rust
    let store = LintStore::new(config);
    let lint_rules = store.filtered_lint_rules(&scope);
    ```
1. Run the filtered lint rules one by one and aggregate any detected issues in a vector.
    ```rust
    let mut issues = Vec::new();
    for (name, rule) in lint_rules {
        rule.run(&resources, &mut issues)?;
    }
    ```

## Mission Goal

The goal of this crate is to enable ArchLinux developers to easily and ergonomically build highly encapsulated and configurable lint rules.
The whole project is structured in a rather modular way to allow quick additions of new scopes, options, groups and such.

Users of the linter should get well-formatted and expressive error messages.
At best even with explicit proposals on how to fix a specific issue.

[`alpm-lint-config`]: file:///home/nuke/work/repos/alpm-rs/target/doc/alpm_lint_config/index.html
[`linting_config`]: file:///home/nuke/work/repos/alpm-rs/target/doc/alpm_lint_config/macro.linting_config.html
[`FilteredLintRules`]: file:///home/nuke/work/repos/alpm-rs/target/doc/alpm_lint/lint_rules/store/struct.FilteredLintRules.html
[`Level`]: file:///home/nuke/work/repos/alpm-rs/target/doc/alpm_lint/enum.Level.html
[`LintConfiguration`]: file:///home/nuke/work/repos/alpm-rs/target/doc/alpm_lint_config/struct.LintConfiguration.html
[`LintGroup`]: file:///home/nuke/work/repos/alpm-rs/target/doc/alpm_lint_config/enum.LintGroup.html
[`LintIssue`]: file:///home/nuke/work/repos/alpm-rs/target/doc/alpm_lint/issue/struct.LintIssue.html
[`LintRule`]: file:///home/nuke/work/repos/alpm-rs/target/doc/alpm_lint/lint_rules/trait.LintRule.html
[`LintRuleConfiguration`]: file:///home/nuke/work/repos/alpm-rs/target/doc/alpm_lint_config/lint_rule/struct.LintRuleConfiguration.html
[`LintScope`]: file:///home/nuke/work/repos/alpm-rs/target/doc/alpm_lint/scope/enum.LintScope.html
[`LintStore`]: file:///home/nuke/work/repos/alpm-rs/target/doc/alpm_lint/lint_rules/store/struct.LintStore.html
[`Resources`]: file:///home/nuke/work/repos/alpm-rs/target/doc/alpm_lint/resources/enum.Resources.html
