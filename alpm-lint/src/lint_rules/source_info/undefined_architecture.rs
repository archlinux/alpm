//! Ensures that [SRCINFO] data only contains architecture-specific fields for defined
//! architectures.
//!
//! [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html

use std::collections::BTreeMap;

use alpm_types::{Architectures, SystemArchitecture};
use colored::Colorize;
use documented::Documented;

use crate::{
    internal_prelude::*,
    issue::SourceInfoIssue,
    lint_rules::source_info::source_info_from_resource,
};

/// # What it does
///
/// Ensures that [SRCINFO] data only contains architecture-specific fields for declared
/// architectures (see [alpm-architecture]).
///
/// # Why is this bad?
///
/// Architecture-specific fields can be used to provide overrides for a field on a specific
/// [alpm-architecture]. If the architecture for an architecture-specific field is not specified in
/// a [PKGBUILD] or [SRCINFO], the data of the architecture-specific fields is unused. Such fields
/// are often remnants of architecture removals in the respective [PKGBUILD] that were not fully
/// cleaned up.
///
/// # Example
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
///
/// [PKGBUILD]: https://man.archlinux.org/man/PKGBUILD.5
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
/// [alpm-architecture]: https://alpm.archlinux.page/specifications/alpm-architecture.7.html
#[derive(Clone, Debug, Documented)]
pub struct UndefinedArchitecture {}

impl UndefinedArchitecture {
    /// Create a new, boxed instance of [`UndefinedArchitecture`].
    pub fn new_boxed(_: &LintRuleConfiguration) -> Box<dyn LintRule> {
        Box::new(Self {})
    }
}

/// Check if a system architecture is contained by a (set) of [Architectures].
fn contains_architecture(left: &Architectures, right: &SystemArchitecture) -> bool {
    // If left is `Any`, all architectures are valid.
    let Architectures::Some(left) = left else {
        return true;
    };

    left.contains(right)
}

impl LintRule for UndefinedArchitecture {
    fn name(&self) -> &'static str {
        "undefined_architecture"
    }

    fn scope(&self) -> LintScope {
        LintScope::SourceInfo
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

        let base_archs = &source_info.base.architectures;

        // Check package base architecture properties
        for arch in source_info.base.architecture_properties.keys() {
            if !contains_architecture(base_archs, arch) {
                issues.push(LintIssue {
                    lint_rule: self.scoped_name(),
                    level: self.level(),
                    help_text: self.help_text(),
                    scope: self.scope(),
                    issue_type: SourceInfoIssue::Generic {
                            summary: "found field for an undefined architecture".to_string(),
                            arrow_line: None,
                            message: format!(
                                "An architecture-specific field is used for the undeclared architecture '{}'",
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
            let package_archs = if let Some(archs) = &package.architectures {
                archs
            } else {
                base_archs
            };

            for arch in package.architecture_properties.keys() {
                if !contains_architecture(package_archs, arch) {
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

    fn extra_links(&self) -> Option<BTreeMap<String, String>> {
        let mut links = BTreeMap::new();
        links.insert(
            "PKGBUILD man page".to_string(),
            "https://man.archlinux.org/man/PKGBUILD.5".to_string(),
        );
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
