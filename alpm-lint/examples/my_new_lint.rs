//! Example implementation of a new lint rule.
//!
//! This serves as a template for creating new lint rules in the alpm-lint framework.
//!
//! To create a new lint rule:
//! - Place it under `src/lint_rules/{scope}/`.
//! - Use `crate::internal_prelude::*` instead of `alpm_lint::*`.
//! - Register it in the `LintStore` so the framework knows about it.

// Allow dead code for this example, as it contains stub/example fields that aren't used.
#![allow(dead_code)]

use std::{collections::BTreeMap, str::FromStr};

// Instead of this lengthy import, you would simply use `crate::internal_prelude::*`.
use alpm_lint::{
    Error,
    Level,
    LintRule,
    LintScope,
    Resources,
    issue::{LintIssue, SourceInfoIssue},
};
use alpm_lint_config::{LintGroup, LintRuleConfiguration, LintRuleConfigurationOptionName};
use alpm_srcinfo::{SourceInfo, source_info::v1::package_base::PackageBase};
use alpm_types::{Architecture, FullVersion, Name};
use documented::Documented;
use testresult::TestResult;

/// # What it does
///
/// This is an example lint rule that demonstrates the basic structure of a lint rule.
/// It ensures if the package base license is a non-SPDX license.
///
/// # Why is this bad?
///
/// Arch Linux decided to use SPDX conform license expressions.
///
/// # Example
///
/// ```ini,ignore
/// pkgbase = test
/// pkgver = 1.0.0
/// pkgrel = 1
/// licenses = GPLv3 or later
/// ```
///
/// Use instead:
///
/// ```ini,ignore
/// pkgbase = test
/// pkgver = 1.0.0
/// pkgrel = 1
/// licenses = GPL-3.0-or-later
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
        // Must be unique, always use snake_case.
        "my_new_lint"
    }

    fn scope(&self) -> LintScope {
        // Choose appropriate scope.
        LintScope::SourceInfo
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

    // We use the [`documented`] crate to expose the struct's doc string as a constant variable.
    // That way we can use the normal rust documentation while also exposing it externally via our
    // API.
    //
    // [`documented`]: https://crates.io/crates/documented
    fn documentation(&self) -> String {
        MyNewLint::DOCS.into()
    }

    fn help_text(&self) -> String {
        r#"This example lint rule detected a non-spdx compliant license expression.

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
        // The logic below simply throws an error whenever an non-SPDX license is encountered.
        for license in &source_info.base.licenses {
            if !license.is_spdx() {
                // When an issue is encountered, add it to the issues vector.
                issues.push(LintIssue::from_rule(
                    self,
                    SourceInfoIssue::BaseField {
                        field_name: "license".to_string(),
                        value: license.to_string(),
                        context: "License is not SPDX compliant".to_string(),
                        architecture: None,
                    }
                    .into(),
                ));
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
    /// Return the associated links for this lint rule.
    fn extra_links(&self) -> Option<BTreeMap<String, String>> {
        let mut links = BTreeMap::new();
        links.insert(
            "RFC".to_string(),
            "https://rfc.archlinux.page/0016-SPDX-license-identifiers/".to_string(),
        );
        Some(links)
    }
}

fn main() -> TestResult {
    println!("This is an example lint rule structure for the alpm-lint framework.");
    println!("See MyNewLint in ./alpm-lint/examples/my_new_lint.rs for more details.");

    // Create a minimal SourceInfoV1 struct with a PackageBase that has a non-SPDX license
    let mut base = PackageBase::new_with_defaults(
        Name::from_str("test-package")?,
        FullVersion::from_str("1:1.0.0-1")?,
    );
    base.architectures = vec![Architecture::X86_64];

    // Add the non-SPDX license
    base.licenses
        .push(alpm_types::License::Unknown("Unknown License".to_string()));

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
