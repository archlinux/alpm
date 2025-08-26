//! Ensures that license expressions in [SRCINFO] data are [SPDX] compliant.
//!
//! [SPDX]: https://spdx.org/licenses/
//! [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html

use std::collections::BTreeMap;

use alpm_lint_config::LintRuleConfigurationOptionName;
use alpm_srcinfo::source_info::v1::package::Override;
use documented::Documented;

use crate::{
    Level,
    internal_prelude::*,
    issue::SourceInfoIssue,
    lint_rules::source_info::source_info_from_resource,
};

/// # What it does
///
/// Ensures that each license in a [SRCINFO] is a valid SPDX license expression.
///
/// # Why is this bad?
///
/// The license attribution for packages clearly defines under what license(s) a package is
/// distributed. When not using valid SPDX license identifiers to describe the license of a package,
/// it may be unclear what license applies for it. Unclear license attribution has implication for
/// the reuse of the package in binary form and whether source code must be made available for it.
/// For this reason, Arch Linux decided to only allow valid SPDX license expressions (see [RFC
/// 0016]).
///
/// # Examples
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
///
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
/// [RFC 0016]: https://rfc.archlinux.page/0016-spdx-license-identifiers/
#[derive(Clone, Debug, Documented)]
pub struct NotSPDX {}

impl NotSPDX {
    /// Create a new, boxed instance of [`NotSPDX`].
    pub fn new_boxed(_: &LintRuleConfiguration) -> Box<dyn LintRule> {
        Box::new(Self {})
    }
}

impl LintRule for NotSPDX {
    fn name(&self) -> &'static str {
        "invalid_spdx_license"
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
        r#"Licenses should use SPDX identifiers as specified in the Arch Linux RFC 0016.

SPDX license identifiers provide a standardized way to identify licenses.
- Apache-2.0 (instead of "Apache")
- GPL-3.0-or-later or GPL-3.0-only (instead of "GPL3")
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

        // Check licenses for all split packages
        for package in &source_info.packages {
            // If we don't have an override, there's nothing to check.
            let Override::Yes { value } = &package.licenses else {
                continue;
            };

            for license in value {
                if !license.is_spdx() {
                    issues.push(LintIssue::from_rule(
                        self,
                        SourceInfoIssue::PackageField {
                            package_name: package.name.to_string(),
                            field_name: "license".to_string(),
                            value: license.to_string(),
                            context: "License is not SPDX compliant".to_string(),
                            architecture: None,
                        }
                        .into(),
                    ));
                }
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
            "Arch Linux RFC 0016".to_string(),
            "https://rfc.archlinux.page/0016-spdx-license-identifiers/".to_string(),
        );
        links.insert(
            "SRCINFO specification".to_string(),
            "https://alpm.archlinux.page/specifications/SRCINFO.5.html".to_string(),
        );

        Some(links)
    }
}
