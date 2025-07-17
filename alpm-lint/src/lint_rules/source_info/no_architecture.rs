//! Ensures that SourceInfo data does not contain some sort of architecture-specific fields for an
//! architecture that isn't specified for a given architecture (or pkg base).

use documented::Documented;

use crate::{Level, internal_prelude::*, lint_rules::source_info::source_info_from_resource};

/// ### What it does
///
/// Ensures that an architecture is set.
///
/// ### Why is this bad?
///
/// An architecture should be explicitly set, as otherwise `any` is simply assumed.
///
/// ### Example
///
/// ```ini,ignore
/// pkgbase = test
///     pkgver = 1.0.0
///     pkgrel = 1
/// ```
///
/// Use instead:
///
/// ```ini,ignore
/// pkgbase = test
///     pkgver = 1.0.0
///     pkgrel = 1
///     arch = x86_64
/// ```
#[derive(Clone, Debug, Documented)]
pub struct NoArchitecture {}

impl NoArchitecture {
    /// Create a new, boxed instance of [`NoArchitecture`].
    ///
    /// This is used to register the lint on the `LintStore`.
    pub fn new_boxed(_: &LintRuleConfiguration) -> Box<dyn LintRule> {
        Box::new(Self {})
    }
}

impl LintRule for NoArchitecture {
    fn name(&self) -> &'static str {
        "no_architecture"
    }

    fn scope(&self) -> LintScope {
        LintScope::SourceInfo
    }

    fn level(&self) -> Level {
        Level::Deny
    }

    fn documentation(&self) -> String {
        NoArchitecture::DOCS.into()
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
