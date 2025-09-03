//! Example implementation of a new lint rule.
//!
//! This serves as a template for creating new lint rules in the alpm-lint framework.
//!
//! When creating an actual lint rule, you would place it in src/lint_rules/{scope}/
//! and use `crate::` imports instead of `alpm_lint::`, specifically `crate::internal_prelude::*`.
//! Also, register the LintRule in the LintStore for the framework to know about it.

// Allow dead code for this example, as it contains stub/example fields that aren't used.
#![allow(dead_code)]

use std::{collections::BTreeMap, str::FromStr};

// Instead of this lengthy import, you would simply use `crate::internal_prelude::*`.
use alpm_lint::{
    Error,
    Level,
    LintScope,
    issue::{LintIssue, SourceInfoIssue},
    resources::Resources,
    rule::LintRule,
};
use alpm_lint_config::{LintGroup, LintRuleConfiguration, LintRuleConfigurationOptionName};
use alpm_srcinfo::{SourceInfo, source_info::v1::package_base::PackageBase};
use alpm_types::{Architecture, FullVersion, Name};
use documented::Documented;
use testresult::TestResult;

/// ### What it does
///
/// This is an example lint rule that demonstrates the basic structure of a lint rule.
/// It checks if the architecture is x86_64 and reports anything else as an issue.
///
/// ### Why is this bad?
///
/// This is just an example - in practice, x86_64 is a perfectly valid architecture.
/// This rule exists purely for demonstration purposes.
///
/// ### Example
///
/// ```ini,ignore
/// pkgbase = test
/// pkgver = 1.0.0
/// pkgrel = 1
/// arch = x86_64
/// ```
///
/// Use instead
///
/// ```ini,ignore
/// pkgbase = test
/// pkgver = 1.0.0
/// pkgrel = 1
/// arch = any_other_architecture
/// ```
#[derive(Clone, Debug, Documented)]
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
    pub fn new_boxed(_config: &LintRuleConfiguration) -> Box<dyn LintRule> {
        Box::new(Self {
            // For this example, we'll just use a default value since the option doesn't actually
            // exist. In practice, you'd extract any options from `config.options`.
            my_option: false,
        })
    }
}

impl LintRule for MyNewLint {
    fn name(&self) -> &'static str {
        "my_new_lint" // Must be unique, always use snake_case.
    }

    fn scope(&self) -> LintScope {
        LintScope::SourceInfo // Choose appropriate scope
    }

    /// Use this to set your severity [`Level`].
    ///
    /// Otherwise, this can be omitted if the level is [`Level::Warn`].
    fn level(&self) -> Level {
        // Choose: Error, Deny, Warn, or Suggest
        Level::Warn
    }

    /// If your lint should not be enabled by default and is to be added to one or more groups, use
    /// this function.
    ///
    /// Otherwise, this function can be omitted.
    /// The default implementation returns an empty slice.
    fn groups(&self) -> &'static [LintGroup] {
        // Most rules belong to no groups, which implies that they're enabled by default.
        // See [LintGroup] for what groups exist.
        &[]
    }

    // We use the `documented` crate to expose the struct's doc string as a constant variable.
    // That way we can use the normal rust documentation while also exposing it externally via our
    // API.
    fn documentation(&self) -> String {
        MyNewLint::DOCS.into()
    }

    fn help_text(&self) -> String {
        r#"This example lint rule detected x86_64 architecture.

In a real lint rule, you would explain why this is problematic and how users can fix the issue.
"#
        .into()
    }

    // The following is an example for a lint that lints `SourceInfo` data.
    //
    // It extracts the `SourceInfo` data from the `Resources` and runs some lint logic on it.
    fn run(&self, resources: &Resources, issues: &mut Vec<LintIssue>) -> Result<(), Error> {
        // Extract the SourceInfo from the given resources.
        // In an actual implementation, you'd use the source_info_from_resource helper from the
        // module `lint_rules/source_info` module.
        let source_info = match resources {
            Resources::SourceRepository {
                source_info: SourceInfo::V1(source_info),
                ..
            }
            | Resources::SourceInfo(SourceInfo::V1(source_info)) => source_info,
            _ => {
                return Err(Error::InvalidResources {
                    scope: resources.scope(),
                    lint_rule: self.scoped_name(),
                    expected: LintScope::SourceInfo,
                });
            }
        };

        // Implement your linting logic here.
        // The logic below simply throws an error whenever an `x86_64` architecture is encountered.
        for architecture in &source_info.base.architectures {
            if architecture.to_string() == "x86_64" {
                // When an issue is encountered, add it to the issues vector.
                issues.push(LintIssue {
                    lint_rule: self.scoped_name(),
                    level: self.level(),
                    help_text: self.help_text(),
                    scope: self.scope(),
                    issue_type: SourceInfoIssue::BaseField {
                        field_name: "arch".to_string(),
                        value: architecture.to_string(),
                        context: "Found bad architecture".to_string(),
                        architecture: None,
                    }
                    .into(),
                    links: std::collections::BTreeMap::new(),
                });
            }
        }

        Ok(())
    }

    /// If your lint rule uses some configuration fields, use this function to publicly declare the
    /// usage of those options. This is used to establish a backwards relationship of lint rules
    /// and options for documentation purposes.
    ///
    /// Otherwise, this function can be omitted.
    /// The default implementation returns an empty slice.
    fn configuration_options(&self) -> &[LintRuleConfigurationOptionName] {
        // Return references to configuration options your rule uses.
        //&[LintRuleConfigurationOptionName::my_option]
        &[]
    }

    /// If your lint rule documentation points to some external links, use this function
    /// to expose those links in a well-formed manner.
    ///
    /// Otherwise, this function can be omitted.
    /// The default implementation returns `None`.
    fn extra_links(&self) -> Option<BTreeMap<String, String>> {
        let mut map = BTreeMap::new();
        map.insert(
            "specification".to_string(),
            "https://archlinux.org/link_to_some_spec".to_string(),
        );
        Some(map)
    }
}

fn main() -> TestResult {
    println!("This is an example lint rule structure for the alpm-lint framework.");
    println!("See MyNewLint in ./alpm-lint/examples/my_new_lint.rs for more details.");

    // Create a minimal SourceInfoV1 struct with a PackageBase that has the X86_64 architecture.
    let mut base = PackageBase::new_with_defaults(
        Name::from_str("test-package")?,
        FullVersion::from_str("1:1.0.0-1")?,
    );
    base.architectures = vec![Architecture::X86_64];
    let source_info = alpm_srcinfo::SourceInfoV1 {
        base,
        packages: Vec::new(),
    };

    // Initialize the Resources and the config. This is normally all done automatically.
    let resources = Resources::SourceInfo(SourceInfo::V1(source_info.clone()));
    let config = LintRuleConfiguration::default();

    // Run the lint.
    let mut issues = Vec::new();
    let my_lint = MyNewLint::new_boxed(&config);
    my_lint.run(&resources, &mut issues)?;

    for issue in issues {
        println!("{issue}");
    }

    Ok(())
}
