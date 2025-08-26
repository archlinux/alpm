//! Ensures that SourceInfo data does not contain some sort of architecture-specific fields for an
//! architecture that isn't specified for a given architecture (or pkg base).

use documented::Documented;

use crate::{
    Level,
    internal_prelude::*,
    issue::SourceInfoIssue,
    lint_rules::source_info::source_info_from_resource,
};

/// # What it does
///
/// Ensures that an architecture is set.
///
/// # Why is this bad?
///
/// An architecture must be set as otherwise `any` would be implied.
///
/// # Example
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
        Level::Error
    }

    fn documentation(&self) -> String {
        NoArchitecture::DOCS.into()
    }

    fn help_text(&self) -> String {
        r#"An Architecture must be specified.

Make sure to add an 'arch' field to specify the supported architectures for your package.
"#
        .into()
    }

    fn run(&self, resources: &Resources, issues: &mut Vec<LintIssue>) -> Result<(), Error> {
        // Extract the SourceInfo from the given resources.
        let source_info = source_info_from_resource(resources, self.scoped_name())?;

        // Check if architectures list is empty
        if source_info.base.architectures.is_empty() {
            issues.push(LintIssue::from_rule(
                self,
                SourceInfoIssue::MissingField {
                    field_name: "arch".to_string(),
                }
                .into(),
            ));
        }

        Ok(())
    }
}
