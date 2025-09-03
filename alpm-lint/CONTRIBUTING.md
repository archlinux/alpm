# Contributing Guide

First off, it's very much encouraged to read the [architectural guide](./ARCHITECTURE.md) so that you're up-to-speed with the glossary and rough outline of the project.

For general best practices refer to the [contribution guidelines](../CONTRIBUTING.md).

# Adding New Lint Rules

## Prerequisites

Before adding a new lint rule, there're a few things that should be clarified first:

1. Define the problem your lint rule should detect and, if applicable, what needs to be done to resolve it.
1. Read the [`LintRule`] trait implementation to get an overview of the API surface.
1. Take a look at a few existing [`LintRule implementations`] and the [`Example`] the to get a rough idea on how to write a lint.

## `LintRule` Implementation

### Determine the Scope

Figure out which scope the new lint will be applied to:

- `LintScope::SourceInfo` - For [`.SRCINFO`] file-specific lints
- `LintScope::PackageBuild` - For [`PKGBUILD`] file-specific lints  
- `LintScope::PackageInfo` - For [`.PKGINFO`] file-specific lints
- `LintScope::BuildInfo` - For [`.BUILDINFO`] file-specific lints
- `LintScope::SourceRepository` - For lints that need both [`PKGBUILD`] and [`.SRCINFO`]
- `LintScope::Package` - For lints that need both [`.PKGINFO`] and [`.BUILDINFO`]

### Create the Lint Rule File

Create a new file in the submodules of the respective scope in `src/lint_rules`. E.g. `src/lint_rules/source_info/my_new_lint.rs`.

It's encouraged to use the [`Example`] template as a starting point.
The example gives you a general structure, example functions and commented out optional functions with an explanation on when/how to use them.

If you notice that the example is outdated, please ping us!

### Register the Lint Rule

Once you're finished writing your lint, manually add the lint to the [`LintStore`]'s `register()` method.
This is fairly straight-forward and only involves a single import and one line.

The following example shows how to add `MyNewLint` to `src/lint_rules/store.rs`:

```rust
fn register(&mut self) {
    self.lint_constructors = vec![
        // ...
        DuplicateArchitecture::new_boxed,
        MyNewLint::new_boxed, // <---
        UnsafeChecksum::new_boxed,
        // ...
    ];
}
```

> [!NOTE]
Don't forget to sort that array after adding new lints to keep things organized.

## Reporting Issues

When your lint rule detects problems, it must report them using the [`LintIssue`] type.
Picking the correct issue type and using good wording is crucial to provide a good user experience.

Inside each [`LintIssue`] is a [`LintIssueType`], which is the type that actually provides detailed information about the issue.
Take a look at the [`SourceInfoIssue`] enum go get an idea of how this looks like.
Be aware that [`SourceInfoIssue`] and equivalents also implement `Into<LintIssueType>` for your convenience.

[`LintIssue`]s and its [`LintIssueType`] are converted to [`LintIssueDisplay`] before being printed, to ensure a consistent formatting across the whole linting framework.
The [`LintIssueDisplay`] documentation explains best how this works, what types of fields there are and how the final layout looks like.

The documentation of [`SourceInfoIssue`] and its equivalent types also provide detailed information on how the various variants' fields are used to create a [`LintIssueDisplay`].

### Creating [`LintIssue`] Instances

In your `Lintrule::run()` method, push [`LintIssue`] instances to the `issues` vector:

```rust
fn run(&self, resources: &Resources, issues: &mut Vec<LintIssue>) -> Result<(), Error> {
    // Your linting logic here
    if problem_detected {
        issues.push(LintIssue::from_rule(
            SourceInfoIssue::BaseField {
                field_name: "pkgname".to_string(),
                value: "invalid-name".to_string(),
                context: "Invalid package name format".to_string(),
                architecture: None,
            }.into(),
        ));
    }
    Ok(())
}
```

In the example above, a new [`LintIssue`] is created from your lint rule's data, in combination with a [`LintIssueType::SourceInfo`] that is created from the `SourceInfoIssue.into()` call.
In this example, the `SourceInfoIssue::BaseField` variant is used to indicate problems on the `pkgname` field of a SourceInfo's `pkgbase` section.

## Write Integration Tests

Every lint rule must have at least two integration test in `tests/lint_rule_tests/{scope}/{rule_name}`.
One for a pass run where the rule is not triggered on correct data, and one fail run where the lint rule properly detects an issue.
If your lint rule covers multiple cases, cover all of these cases via tests.

You can use `rstest` to parameterize tests, but don't go overboard.
If your parameterization results in `if` checks inside the test, consider creating a dedicated test function for that case.

Also, check out the `tests/fixtures.rs` module in case you need stub data for testing.

## Configuration Options

If your lint rule needs to be configurable, you might have to add a new configuration options.
For this, you need to adjust the configuration struct in the `alpm-lint-config` crate.
Simply edit `alpm-lint-config/src/lint_rule.rs` and add your new option to the `create_lint_rule_config!` macro at the bottom of the file:

```rust
linting_config! {
    /// This is a test option
    test_option: String = "This is an option",

    /// Description of your new configuration option
    my_config_option: bool = false,  // Add this line
}
```

Then reference it in your lint rule's `configuration_options()` method as shown in the template.

## Checklist

As a rough guideline, you can follow this checklist:

- [ ] Lint rule file created in correct scope directory.
- [ ] Scope, Group and Level are set correctly.
- [ ] Rule registered in `LintStore::register()`
- [ ] Documentation is correct
- [ ] Eventual links or options are exposed via their respective functions.
- [ ] At least two tests have been written and are passing.
      If possible include real-world examples that inspired this rule when writing tests.

[`.BUILDINFO`]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
[`.PKGINFO`]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
[`.SRCINFO`]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
[`PKGBUILD`]: https://man.archlinux.org/man/PKGBUILD.5
[`Example`]: https://gitlab.archlinux.org/archlinux/alpm/alpm/-/blob/main/alpm-lint/example/my_new_lint.rs
[`Level`]: https://alpm.archlinux.page/rustdoc/alpm_lint/enum.Level.html
[`LintIssue`]: https://alpm.archlinux.page/rustdoc/alpm_lint/issue/struct.LintIssue.html
[`LintIssueType`]: https://alpm.archlinux.page/rustdoc/alpm_lint/issue/enum.LintIssueType.html
[`SourceInfoIssue`]: https://alpm.archlinux.page/rustdoc/alpm_lint/issue/enum.SourceInfoIssue.html
[`LintIssueDisplay`]: https://alpm.archlinux.page/rustdoc/alpm_lint/issue/display/struct.LintIssueDisplay.html
[`LintRule`]: https://alpm.archlinux.page/rustdoc/alpm_lint/lint_rules/trait.LintRule.html
[`LintRule implementations`]: https://alpm.archlinux.page/rustdoc/alpm_lint/lint_rules/index.html
[`LintScope`]: https://alpm.archlinux.page/rustdoc/alpm_lint/scope/enum.LintScope.html
[`LintStore`]: https://alpm.archlinux.page/rustdoc/alpm_lint/lint_rules/store/struct.LintStore.html
