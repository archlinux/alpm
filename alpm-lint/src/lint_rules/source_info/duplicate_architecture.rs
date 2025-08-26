//! Checks that architectures are not specified twice.

use std::collections::HashSet;

use documented::Documented;

use crate::{
    internal_prelude::*,
    issue::SourceInfoIssue,
    lint_rules::source_info::source_info_from_resource,
};

/// # What it does
///
/// Checks that architectures are not specified twice.
///
/// # Why is this bad?
///
/// Duplicate architecture definitions are ignored and duplicate definitions don't serve any
/// purpose despite being confusing.
///
/// # Example
///
/// ```ini,ignore
/// pkgbase = test
///     pkgver = 1.0.0
///     pkgrel = 1
///     arch = x86_64
///     arch = x86_64
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
pub struct DuplicateArchitecture {}

impl DuplicateArchitecture {
    /// Create a new, boxed instance of [`DuplicateArchitecture`].
    ///
    /// This is used to register the lint on the `LintStore`.
    pub fn new_boxed(_: &LintRuleConfiguration) -> Box<dyn LintRule> {
        Box::new(DuplicateArchitecture {})
    }
}

impl LintRule for DuplicateArchitecture {
    fn name(&self) -> &'static str {
        "duplicate_architecture"
    }

    fn scope(&self) -> LintScope {
        LintScope::SourceInfo
    }

    fn documentation(&self) -> String {
        DuplicateArchitecture::DOCS.into()
    }

    fn help_text(&self) -> String {
        r#"Architecture lists for packages should always be unique.

Duplicate architecture declarations such as `arch=(x86_64 x86_64)` are ignored.
"#
        .into()
    }

    fn run(&self, resources: &Resources, issues: &mut Vec<LintIssue>) -> Result<(), Error> {
        // Extract the SourceInfo from the given resources.
        let source_info = source_info_from_resource(resources, self.scoped_name())?;

        let mut known = HashSet::new();
        for architecture in &source_info.base.architectures {
            if known.contains(&architecture) {
                issues.push(LintIssue::from_rule(
                    self,
                    SourceInfoIssue::BaseField {
                        field_name: "arch".to_string(),
                        value: architecture.to_string(),
                        context: "Found duplicate architecture".to_string(),
                        architecture: None,
                    }
                    .into(),
                ));
            }
            known.insert(architecture);
        }

        Ok(())
    }
}
