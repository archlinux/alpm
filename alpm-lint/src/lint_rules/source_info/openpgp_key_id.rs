//! Ensures that [OpenPGP Key IDs] are not used in [SRCINFO] data.
//!
//! [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
//! [OpenPGP Key IDs]: https://openpgp.dev/book/glossary.html#term-Key-ID

use std::collections::BTreeMap;

use documented::Documented;

use crate::{
    internal_prelude::*,
    issue::SourceInfoIssue,
    lint_rules::source_info::source_info_from_resource,
};

/// # What it does
///
/// Ensure that no [OpenPGP Key ID] is used to authenticate and verify upstream artifacts.
///
/// # Why is this bad?
///
/// An [OpenPGP certificate] can be used to verify and authenticate upstream sources.
/// In [PKGBUILD] and [SRCINFO] files these certificates are identified using an ID.
/// This allows the retrieval of matching certificates from remote resources (e.g. Web Key Directory
/// or OpenPGP keyservers).
///
/// An [OpenPGP Key ID] is a short identifier that can be used to identify an [OpenPGP certificate].
/// However, its uniqueness cannot be guaranteed and thus it does not guard against collision.
///
/// If an [OpenPGP certificate] cannot be uniquely identified:
///
/// - an arbitrary certificate may have a matching [OpenPGP Key ID] and it would not be possible to
///   use it for authentication and verification of the particular upstream sources.
/// - sophisticated attackers may be able to craft a certificate with a matching [OpenPGP Key ID]
///   and swap upstream sources and digital signatures with malicious ones.
///
/// Only an [OpenPGP fingerprint] meaningfully guards against collision and should always be used
/// instead of an [OpenPGP Key ID] to uniquely identify an [OpenPGP certificate].
///
/// # Example
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
///
/// [PKGBUILD]: https://man.archlinux.org/man/PKGBUILD.5
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
/// [OpenPGP Key ID]: https://openpgp.dev/book/glossary.html#term-Key-ID
/// [OpenPGP certificate]: https://openpgp.dev/book/certificates.html
/// [OpenPGP fingerprint]: https://openpgp.dev/book/certificates.html#fingerprint
#[derive(Clone, Debug, Documented)]
pub struct OpenPGPKeyId {}

impl OpenPGPKeyId {
    /// Create a new, boxed instance of [`OpenPGPKeyId`].
    pub fn new_boxed(_: &LintRuleConfiguration) -> Box<dyn LintRule> {
        Box::new(OpenPGPKeyId {})
    }
}

impl LintRule for OpenPGPKeyId {
    fn name(&self) -> &'static str {
        "openpgp_key_id"
    }

    fn scope(&self) -> LintScope {
        LintScope::SourceInfo
    }

    fn level(&self) -> Level {
        crate::Level::Deny
    }

    fn documentation(&self) -> String {
        OpenPGPKeyId::DOCS.into()
    }

    fn help_text(&self) -> String {
        r#"OpenPGP Key IDs are not safe and must not be used for authentication and verification.

Key IDs are short identifiers (8 or 16 hex characters) that are not guaranteed to be unique.
Use 40-character long OpenPGP fingerprints instead to prevent collision attacks.
"#
        .into()
    }

    fn run(&self, resources: &Resources, issues: &mut Vec<LintIssue>) -> Result<(), Error> {
        // Extract the SourceInfo from the given resources.
        let source_info = source_info_from_resource(resources, self.scoped_name())?;

        // Check PGP identifiers to see if any are key IDs (not fingerprints)
        for identifier in &source_info.base.pgp_fingerprints {
            if matches!(identifier, alpm_types::OpenPGPIdentifier::OpenPGPKeyId(_)) {
                issues.push(LintIssue::from_rule(
                    self,
                    SourceInfoIssue::BaseField {
                        field_name: "validpgpkeys".to_string(),
                        value: identifier.to_string(),
                        context: "OpenPGP Key IDs are not allowed".to_string(),
                        architecture: None,
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
            "OpenPGP Key ID - OpenPGP for application developers".to_string(),
            "https://openpgp.dev/book/glossary.html#term-Key-ID".to_string(),
        );
        links.insert(
            "OpenPGP certificate - OpenPGP for application developers".to_string(),
            "https://openpgp.dev/book/certificates.html".to_string(),
        );
        links.insert(
            "OpenPGP fingerprint - OpenPGP for application developers".to_string(),
            "https://openpgp.dev/book/certificates.html#fingerprint".to_string(),
        );

        Some(links)
    }
}
