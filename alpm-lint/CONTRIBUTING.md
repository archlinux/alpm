# Contributing Guide

First of, it's very much encouraged to read the [architectural guide](./ARCHITECTURE.md) so that you're up-to-speed with the glossary and rough outline of the project.

If you're familiar with that document, feel free to continue!

For general best practices refer to the projects global [CONTRIBUTING.md](../CONTRIBUTING.md).

# Adding New Lint Rules

## Prerequisites

Before adding a new lint rule, there're a few things that should be clarified first:

1. Clarify the problem your lint rule will detect and if applicable what needs to be done to solve it.
1. Read the [`LintRule`] trait implementation to get an overview of the API surface.
1. Take a look at a few existing [`LintRule implementations`] to get a rough idea on how to write a lint.

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

You can use this template as a starting point:

```rust
//! Brief description of what this lint checks.

use crate::{internal_prelude::*, issue::LintIssueType};

/// Proper description of the lint rule and its purpose.
#[derive(Clone, Debug)]
pub struct MyNewLint {
    // Add any configuration options you might want to extract from the [`LintRuleConfiguration`]
    //
    // You can keep the struct empty if it doesn't need any configuration.
    my_option: bool,
}

impl MyNewLint {
    /// Create a new, boxed instance of [`MyNewLint`].
    ///
    /// This is used to register the lint on the `LintStore`.
    pub fn new_boxed(config: &LintRuleConfiguration) -> Box<dyn LintRule> {
        Box::new(Self {
            my_option: config.options.my_option,
        })
    }
}

impl LintRule for MyNewLint {
    fn name(&self) -> &'static str {
        "my_new_lint"  // Must be unique, use snake_case
    }

    fn scope(&self) -> LintScope {
        LintScope::SourceInfo  // Choose appropriate scope
    }

    // // The default implementation returns `Level::Warn`.
    // // Unless another level is picked, this can be omitted.
    // fn level(&self) -> Level {
    //     // Choose: Error, Deny, Warn, or Suggest
    //     Level::Warn
    // }

    // // The default implementation returns an empty slice.
    // // Unless the lint is to be added to one or more groups, this function can be omitted.
    // fn groups(&self) -> &'static [LintGroup] {
    //     // Most rules belong to no groups, which implies that they're enabled by default.
    //     // See [LintGroup] for what groups exist.
    //     &[]
    // }

    fn help_text(&self) -> &'static str {
        r#"Explain why this lint rule has been triggered.

Explain what users can do fix the issue or how they can disable the lint.
"#
    }

    // The following logic is an example for a lint that lints `SourceInfo` data.
    //
    // It extracts the `SourceInfo` data from the `Resources` and runs some lint logic on it.
    fn run(&self, resources: &Resources, issues: &mut Vec<LintIssue>) -> Result<(), Error> {
        // Extract the required resources for your scope
        let source_info = match resources {
            // Extract SourceInfo from the expected resource types
            Resources::SourceRepository {
                source_info: SourceInfo::V1(source_info),
                ..
            }
            | Resources::SourceInfo(SourceInfo::V1(source_info)) => source_info,
            // Handle unexpected resource types
            _ => {
                return Err(Error::InvalidResources {
                    scope: resources.scope(),
                    lint_rule: self.scoped_name(),
                    expected: LintScope::SourceInfo,
                });
            }
        };

        // Implement your linting logic here.
        // This logic throws an error whenever an `x86_64` architecture is encountered.
        if source_info.base.architecture == alpm_types::Architecture::X86_64 {
            // When an issue is encountered, add it to the passed issues vector.
            issues.push(LintIssue {
                lint_rule: self.scoped_name(),
                level: self.level(),
                help_text: self.help_text().to_string(),
                issue_type: LintIssueType::Field {
                    scope: self.scope(),
                },
            });
        }

        Ok(())
    }

    // // The default implementation returns an empty slice.
    // // Unless your lint rule uses some configuration fields, this function can be omitted.
    // fn configuration_options(&self) -> &[LintRuleConfigurationOptionName] {
    //     // Return references to configuration options your rule uses.
    //     &[LintRuleConfigurationOptionName::my_option]
    // }
}
```

### Register the Lint Rule

Once you're finished writing your lint, manually add the lint to the [`LintStore`]'s `register()` method.
This is fairly straight-forward and only involves a single import and one line.

The following example shows how to add `MyNewLint` to `src/lint_rules/store.rs`:

```rust
fn register(&mut self) {
    self.lint_constructors = vec![
        // ...
        DuplicateArchitecture::new_boxed,
        UnsafeChecksum::new_boxed,
        MyNewLint::new_boxed, // <---
        // ...
    ];
}
```

## Configuration Options

If your lint rule needs to be configurable, you might have to add a new configuration options.
For this, you need to adjust the configuration struct in the `alpm-lint-config` crate.
Simply edit `alpm-lint-config/src/lint_rule.rs` and add your new option to the `linting_config!` macro at the bottom of the file:

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

As a rough guideline, you can follow this checklist :)

- [ ] Lint rule file created in correct scope directory.
- [ ] Scope, Group and Level are set correctly.
- [ ] Rule registered in `LintStore::register()`
- [ ] Tests written and passing
      If possible include real-world examples that inspired this rule in tests.

[`.BUILDINFO`]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
[`.PKGINFO`]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
[`.SRCINFO`]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
[`PKGBUILD`]: https://man.archlinux.org/man/PKGBUILD.5
[`Level`]: https://alpm.archlinux.page/rustdoc/alpm_lint/enum.Level.html
[`LintRule`]: https://alpm.archlinux.page/rustdoc/alpm_lint/lint_rules/trait.LintRule.html
[`LintRule implementations`]: https://alpm.archlinux.page/rustdoc/alpm_lint/lint_rules/index.html
[`LintScope`]: https://alpm.archlinux.page/rustdoc/alpm_lint/scope/enum.LintScope.html
[`LintStore`]: https://alpm.archlinux.page/rustdoc/alpm_lint/lint_rules/store/struct.LintStore.html
