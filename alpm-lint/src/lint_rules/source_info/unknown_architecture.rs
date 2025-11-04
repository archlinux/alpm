//! Ensures that all [alpm-architecture] names specified in the [SRCINFO] are known.
//!
//! [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
//! [alpm-architecture]: https://alpm.archlinux.page/specifications/alpm-architecture.7.html

use std::collections::BTreeMap;

use alpm_types::{Architecture, SystemArchitecture};
use documented::Documented;
use strum::VariantNames;

use crate::{
    internal_prelude::*,
    issue::SourceInfoIssue,
    lint_rules::source_info::source_info_from_resource,
    utils::EditDistance,
};

/// # What it does?
///
/// Ensures that all [alpm-architecture] names specified in the [SRCINFO] are known.
///
/// Any string passing the [alpm-architecture] requirements is a valid architecture. However,
/// in most cases, only a limited set of architecture names is used in practice.
///
/// This lint rule checks whether any uncommon architecture names are used in the [SRCINFO] which
/// often indicates a typo.
///
/// Warnings emitted by this lint rule can be safely ignored if the specified architecture name is
/// correct, but simply uncommon.
///
/// # Why is this bad?
///
/// Using an unknown architecture name is often a sign of a typo.
///
/// # Example
///
/// ```ini,ignore
/// pkgbase = test
///     pkgver = 1.0.0
///     pkgrel = 1
///     arch = 86_64
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
pub struct UnknownArchitecture {}

impl UnknownArchitecture {
    /// The maximum edit distance to suggest a similar architecture name.
    const EDIT_DISTANCE_THRESHOLD: usize = 3;

    /// Create a new, boxed instance of [`UnknownArchitecture`].
    pub fn new_boxed(_: &LintRuleConfiguration) -> Box<dyn LintRule> {
        Box::new(Self {})
    }

    /// Get all known architecture variant names, including 'any'.
    fn known_variants() -> Vec<&'static str> {
        let mut known = vec!["any"];
        known.append(
            &mut SystemArchitecture::VARIANTS
                .iter()
                .filter_map(|v| if *v != "unknown" { Some(*v) } else { None })
                .collect::<Vec<_>>(),
        );
        known
    }
}

impl LintRule for UnknownArchitecture {
    fn name(&self) -> &'static str {
        "unknown_architecture"
    }

    fn scope(&self) -> LintScope {
        LintScope::SourceInfo
    }

    fn documentation(&self) -> String {
        UnknownArchitecture::DOCS.into()
    }

    fn help_text(&self) -> String {
        let known = Self::known_variants()
            .iter()
            .map(|arch| format!("- {}", arch))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "If you are certain that the architecture name is correct, \
        you can ignore this warning. \n\
        Known values include: \n{known}"
        )
    }

    fn run(&self, resources: &Resources, issues: &mut Vec<LintIssue>) -> Result<(), Error> {
        let source_info = source_info_from_resource(resources, self.scoped_name())?;

        for arch in &source_info.base.architectures {
            if let Architecture::Some(SystemArchitecture::Unknown(arch)) = arch {
                let closest_match = Self::known_variants()
                    .iter()
                    .map(|known| (*known, arch.to_string().edit_distance(&known.to_string())))
                    .min_by_key(|(_, dist)| *dist);

                let suggestion = if let Some((closest, distance)) = closest_match
                    && distance <= Self::EDIT_DISTANCE_THRESHOLD
                {
                    Some(format!("\nDid you mean '{closest}'?"))
                } else {
                    None
                };

                issues.push(LintIssue::from_rule(
                    self,
                    SourceInfoIssue::Generic {
                        summary: "Uncommon architecture specified - possible typo.".to_string(),
                        arrow_line: None,
                        message: format!(
                            "The architecture '{}' is not common. \
                            This is allowed, but may indicate a typo. \
                            {}",
                            arch,
                            suggestion.unwrap_or_default()
                        ),
                    }
                    .into(),
                ));
            }
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
