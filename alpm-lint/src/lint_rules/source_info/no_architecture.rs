//! Ensures that [SRCINFO] data contains at least one [alpm-architecture].
//!
//! [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
//! [alpm-architecture]: https://alpm.archlinux.page/specifications/alpm-architecture.7.html

use std::collections::BTreeMap;

use documented::Documented;

use crate::{
    Level,
    internal_prelude::*,
    issue::SourceInfoIssue,
    lint_rules::source_info::source_info_from_resource,
};

/// # What it does?
///
/// Ensures that an architecture (see [alpm-architecture]) is set in a [SRCINFO].
///
/// # Why is this bad?
///
/// An [alpm-architecture] must be set specifically in a [SRCINFO] as otherwise `any` would be
/// implied.
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
///
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
/// [alpm-architecture]: https://alpm.archlinux.page/specifications/alpm-architecture.7.html
#[derive(Clone, Debug, Documented)]
pub struct NoArchitecture {}

impl NoArchitecture {
    /// Create a new, boxed instance of [`NoArchitecture`].
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

    fn extra_links(&self) -> Option<BTreeMap<String, String>> {
        let mut links = BTreeMap::new();
        links.insert(
            "SRCINFO specification".to_string(),
            "https://alpm.archlinux.page/specifications/SRCINFO.5.html".to_string(),
        );
        links.insert(
            "alpm-architecture specification".to_string(),
            "https://alpm.archlinux.page/specifications/alpm-architecture.7.html".to_string(),
        );

        Some(links)
    }
}
