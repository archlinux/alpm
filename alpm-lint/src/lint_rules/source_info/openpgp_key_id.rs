//! Checks that architectures are not specified twice.

use documented::Documented;

use crate::{internal_prelude::*, lint_rules::source_info::source_info_from_resource};

/// ### What it does
///
/// Ensure that no OpenPGP Key IDs are used to verify the integrity of upstream artifacts.
///
/// ### Why is this bad?
///
/// OpenPGP Key IDs are highly discouraged, as their length doesn't guarantee uniqueness.
/// It could allow somebody else to craft a different key with an identical key ID.
///
/// Upstream artifacts are checked with these Key IDs. Being able to craft a new key with the same
/// ID would allow attackers to craft malicious artifacts that pass this validity check.
///
/// Attackers could then to swap existing artifacts with these malicious artifacts. Any packages
/// that're then re-build on those artifacts would be compromised, without any indicator that
/// something changed.
///
/// ### Example
///
/// ```ini,ignore
/// pkgbase = test
///     pkgver = 1.0.0
///     pkgrel = 1
///     arch = x86_64
///     validpgpkeys = 2F2670AC164DB36F
/// ```
///
/// Use instead:
///
/// ```ini,ignore
/// pkgbase = test
///     pkgver = 1.0.0
///     pkgrel = 1
///     arch = x86_64
///     validpgpkeys = 4A0C4DFFC02E1A7ED969ED231C2358A25A10D94E
/// ```
#[derive(Clone, Debug, Documented)]
pub struct OpenPGPKeyId {}

impl OpenPGPKeyId {
    /// Create a new, boxed instance of [`OpenPGPKeyId`].
    ///
    /// This is used to register the lint on the `LintStore`.
    pub fn new_boxed(_: &LintRuleConfiguration) -> Box<dyn LintRule> {
        Box::new(OpenPGPKeyId {})
    }
}

impl LintRule for OpenPGPKeyId {
    fn name(&self) -> &'static str {
        "open_pgpg_key_id"
    }

    fn scope(&self) -> LintScope {
        LintScope::SourceInfo
    }

    fn documentation(&self) -> String {
        OpenPGPKeyId::DOCS.into()
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
}
