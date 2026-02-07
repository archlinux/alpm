//! Ensures that values of [SRCINFO] keywords do not exceed their byte limits.
//!
//! [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html

use std::collections::BTreeMap;

use alpm_lint_config::LintRuleConfiguration;
use alpm_srcinfo::source_info::v1::package::Override;
use documented::Documented;

use crate::{
    internal_prelude::*,
    issue::SourceInfoIssue,
    lint_rules::source_info::source_info_from_resource,
};

/// Grouping together field properties for validation and error messages.
///
/// `.0` - name-or-keyword
/// `.1` - value
/// `.2` - byte-limit
type Field = (&'static str, String, usize);

/// Byte limits for values of [SRCINFO] keywords.
///
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
pub mod limits {
    /// Prints the docs.rs URL to this module.
    ///
    /// This is meant to inform users of the byte limits for values of SRCINFO keywords in
    /// `<LongValuesAurweb as LintRule>::extra_links()`.
    pub(super) fn docsrs_page() -> String {
        const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");
        let module_path = const { module_path!() }.replace("::", "/");

        format!("https://docs.rs/{CARGO_PKG_NAME}/latest/{module_path}/index.html")
    }

    // Writing byte limits in docs so that we don't have to open up each const in our browser just
    // to see their value.

    /// Limit for `pkgbase` (`255` bytes).
    pub const PKGBASE: usize = 255;
    /// Limit for `pkgname` (`255` bytes).
    pub const PKGNAME: usize = 255;
    /// Limit for `pkgdesc` (`255` bytes).
    pub const PKGDESC: usize = 255;
    /// Limit for `url` (`8000` bytes).
    pub const URL: usize = 8000;
}

/// # What it does
///
/// Ensures that values of [SRCINFO] keywords do not exceed their byte [`limits`].
///
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
#[derive(Clone, Debug, Documented)]
pub struct LongValuesAurweb;

impl LongValuesAurweb {
    /// Creates a new, boxed instance of [`LongValuesAurweb`].
    pub fn new_boxed(_config: &LintRuleConfiguration) -> Box<dyn LintRule> {
        Box::new(Self {})
    }
}

impl LintRule for LongValuesAurweb {
    fn name(&self) -> &'static str {
        "long_values_aurweb"
    }

    fn scope(&self) -> LintScope {
        LintScope::SourceInfo
    }

    fn level(&self) -> Level {
        Level::Warn
    }
    fn documentation(&self) -> String {
        Self::DOCS.into()
    }

    fn help_text(&self) -> String {
        "Value for SRCINFO keywords exceed the byte length restrictions enforced by the aurweb application."
            .to_string()
    }

    fn run(&self, resources: &Resources, issues: &mut Vec<LintIssue>) -> Result<(), Error> {
        // Extract the SourceInfo from the given resources.
        let source_info = source_info_from_resource(resources, self.scoped_name())?;

        let fields: Vec<Field> = {
            let base = &source_info.base;
            let mut fields = vec![("pkgbase", base.name.to_string(), limits::PKGBASE)];

            if let Some(desc) = base.description.as_ref() {
                fields.push(("pkgdesc", desc.to_string(), limits::PKGDESC));
            }

            if let Some(url) = base.url.as_ref() {
                fields.push(("url", url.to_string(), limits::URL));
            }

            fields
        };

        for (keyword, value, limit) in fields {
            if value.len() > limit {
                issues.push(LintIssue::from_rule(
                    self,
                    SourceInfoIssue::BaseField {
                        field_name: keyword.into(),
                        value,
                        context: format!("`{keyword}` value exceeded {limit} bytes"),
                        architecture: None,
                    }
                    .into(),
                ));
            }
        }

        for package in &source_info.packages {
            let fields: Vec<Field> = {
                let mut fields = vec![("pkgname", package.name.to_string(), limits::PKGNAME)];

                if let Override::Yes { ref value } = package.description {
                    fields.push(("pkgdesc", value.to_string(), limits::PKGDESC));
                }

                if let Override::Yes { ref value } = package.url {
                    fields.push(("url", value.to_string(), limits::URL));
                }

                fields
            };

            for (keyword, value, limit) in fields {
                if value.len() > limit {
                    issues.push(LintIssue::from_rule(
                        self,
                        SourceInfoIssue::PackageField {
                            field_name: keyword.into(),
                            package_name: package.name.to_string(),
                            value,
                            context: format!("`{keyword}` value exceeded {limit} bytes"),
                            architecture: None,
                        }
                        .into(),
                    ));
                }
            }
        }

        Ok(())
    }

    fn extra_links(&self) -> Option<BTreeMap<String, String>> {
        let mut links = BTreeMap::new();

        links.insert("Byte limits".to_string(), limits::docsrs_page());
        links.insert(
            "SRCINFO".to_string(),
            "https://alpm.archlinux.page/specifications/SRCINFO.5.html".to_string(),
        );

        Some(links)
    }
}
