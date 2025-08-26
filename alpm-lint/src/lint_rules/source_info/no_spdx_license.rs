//! The file verification for some source in a .SRCINFO file uses an unsafe hash algorithm.

use std::collections::BTreeMap;

use alpm_lint_config::LintRuleConfigurationOptionName;
use documented::Documented;

use crate::{
    Level,
    internal_prelude::*,
    issue::SourceInfoIssue,
    lint_rules::source_info::source_info_from_resource,
};

/// ### What it does
///
/// Checks that the licenses are formatted in the SPDX format.
///
/// ### Why is this bad?
///
/// Arch Linux decided to use SPDX license identifiers going forward.
/// See the official [RFC](https://rfc.archlinux.page/0016-spdx-license-identifiers/#unresolved-questions) for more information.
///
/// ### Example
///
/// ```ini,ignore
/// pkgbase = test
///     pkgver = 1.0.0
///     pkgrel = 1
///     arch = x86_64
///     license = Apache
/// ```
///
/// Use instead:
///
/// ```ini,ignore
/// pkgbase = test
///     pkgver = 1.0.0
///     pkgrel = 1
///     arch = x86_64
///     license = Apache-2.0
/// ```
#[derive(Clone, Debug, Documented)]
pub struct NotSPDX {}

impl NotSPDX {
    /// Create a new, boxed instance of [`NotSPDX`].
    ///
    /// This is used to register the lint on the `LintStore`.
    pub fn new_boxed(_: &LintRuleConfiguration) -> Box<dyn LintRule> {
        Box::new(Self {})
    }
}

impl LintRule for NotSPDX {
    fn name(&self) -> &'static str {
        "no_spdx_license"
    }

    fn scope(&self) -> LintScope {
        LintScope::SourceInfo
    }

    fn level(&self) -> Level {
        Level::Deny
    }

    fn documentation(&self) -> String {
        NotSPDX::DOCS.into()
    }

    fn help_text(&self) -> String {
        r#"Licenses should use SPDX identifiers as specified in the Arch Linux RFC.

SPDX license identifiers provide a standardized way to identify licenses.
- Apache-2.0 (instead of "Apache")
- GPL-3.0-or-later (instead of "GPL")
- MIT (instead of "MIT License")


"#
        .into()
    }

    fn run(&self, resources: &Resources, issues: &mut Vec<LintIssue>) -> Result<(), Error> {
        // Extract the SourceInfo from the given resources.
        let source_info = source_info_from_resource(resources, self.scoped_name())?;

        // Check licenses in the base package
        for license in &source_info.base.licenses {
            if !license.is_spdx() {
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

    fn configuration_options(&self) -> &[LintRuleConfigurationOptionName] {
        &[LintRuleConfigurationOptionName::test_option]
    }

    /// Return the associated links for this lint rule.
    fn extra_links(&self) -> Option<BTreeMap<String, String>> {
        let mut links = BTreeMap::new();
        links.insert(
            "RFC".to_string(),
            "https://rfc.archlinux.page/0016-spdx-license-identifiers/".to_string(),
        );
        Some(links)
    }
}
