//! The file verification for some source in a .SRCINFO file uses an unsafe hash algorithm.

use alpm_lint_config::LintRuleConfigurationOptionName;
use documented::Documented;

use crate::{Level, internal_prelude::*, lint_rules::source_info::source_info_from_resource};

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
        r#"
"#
        .into()
    }

    fn run(&self, resources: &Resources, _issues: &mut Vec<LintIssue>) -> Result<(), Error> {
        // Extract the SourceInfo from the given resources.
        let _source_info = source_info_from_resource(resources, self.scoped_name())?;

        // TODO: Write linting logic
        Ok(())
    }

    fn configuration_options(&self) -> &[LintRuleConfigurationOptionName] {
        &[LintRuleConfigurationOptionName::test_option]
    }
}
