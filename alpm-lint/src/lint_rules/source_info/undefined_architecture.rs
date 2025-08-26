//! Ensures that SourceInfo data does not contain some sort of architecture-specific fields for an
//! architecture that isn't specified for a given architecture (or pkg base).

use colored::Colorize;
use documented::Documented;

use crate::{
    internal_prelude::*,
    issue::SourceInfoIssue,
    lint_rules::source_info::source_info_from_resource,
};

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
        r#"Architecture-specific fields should only be used for declared architectures.

Make sure all architecture-specific fields correspond to architectures declared in the 'arch' field.
"#
        .into()
    }

    fn run(&self, resources: &Resources, issues: &mut Vec<LintIssue>) -> Result<(), Error> {
        // Extract the SourceInfo from the given resources.
        let source_info = source_info_from_resource(resources, self.scoped_name())?;

        // Get the set of declared architectures
        let declared_architectures: std::collections::HashSet<_> =
            source_info.base.architectures.iter().collect();

        // Check package base architecture properties
        for arch in source_info.base.architecture_properties.keys() {
            if !declared_architectures.contains(arch) {
                issues.push(LintIssue {
                    lint_rule: self.scoped_name(),
                    level: self.level(),
                    help_text: self.help_text(),
                    scope: self.scope(),
                    issue_type: SourceInfoIssue::Generic {
                            summary: "found variable for an undefined architecture".to_string(),
                            arrow_line: None,
                            message: format!(
                                "An architecture-specific variable for the undeclared architecture '{}' was found.",
                                arch.to_string().bold()
                            )
                        }
                        .into(),
                    links: std::collections::BTreeMap::new(),
                });
            }
        }

        // Check package architecture properties
        for package in &source_info.packages {
            for arch in package.architecture_properties.keys() {
                if !declared_architectures.contains(arch) {
                    issues.push(LintIssue::from_rule(self,
                        SourceInfoIssue::Generic {
                                summary: "found variable for an undefined architecture".to_string(),
                                arrow_line: Some(format!("for package '{}'", package.name)),
                                message: format!(
                                    "An architecture-specific variable has been for the architecture '{}'",
                                    arch.to_string().bold()
                                )
                            }.into(),
                    ));
                }
            }
        }

        Ok(())
    }
}
