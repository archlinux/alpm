//! Ensures that SourceInfo data does not contain some sort of architecture-specific fields for an
//! architecture that isn't specified for a given architecture (or pkg base).

use documented::Documented;

use crate::{internal_prelude::*, lint_rules::source_info::source_info_from_resource};

/// ### What it does
///
/// Ensures that SourceInfo data does not contain some sort of architecture-specific fields for an
/// architecture that isn't specified for a given architecture (or pkg base).
///
/// ### Why is this bad?
///
/// Fields for unspecified architectures are not used and should be considered dead code.
/// Such fields are likely remnants of architecture removals in the respective PKGBUILD that
/// haven't been fully cleaned up.
///
/// ### Example
///
/// ```ini,ignore
/// pkgbase = test
///     pkgver = 1.0.0
///     pkgrel = 1
///     arch = x86_64
///     # A source property for the aarch64 architecture which isn't specified above.
///     source_aarch64 = https://domain.tld/testing/aarch_64_test.tar.gz
///     source_x86_64 = https://domain.tld/testing/x86_64_test.tar.gz
/// ```
///
/// Use instead:
///
/// ```ini,ignore
/// pkgbase = test
///     pkgver = 1.0.0
///     pkgrel = 1
///     arch = x86_64
///     source_x86_64 = https://domain.tld/testing/x86_64_test.tar.gz
/// ```
#[derive(Clone, Debug, Documented)]
pub struct UndefinedArchitecture {}

impl UndefinedArchitecture {
    /// Create a new, boxed instance of [`UndefinedArchitecture`].
    ///
    /// This is used to register the lint on the `LintStore`.
    pub fn new_boxed(_: &LintRuleConfiguration) -> Box<dyn LintRule> {
        Box::new(Self {})
    }
}

impl LintRule for UndefinedArchitecture {
    fn name(&self) -> &'static str {
        "undefined_architecture"
    }

    fn scope(&self) -> LintScope {
        LintScope::SourceInfo
    }

    fn level(&self) -> Level {
        crate::Level::Warn
    }

    fn documentation(&self) -> String {
        UndefinedArchitecture::DOCS.into()
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
