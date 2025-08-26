//! Ensures that each [alpm-package-source-checksum] in [SRCINFO] data uses a safe hash function.
//!
//! [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
//! [alpm-package-source-checksum]: https://alpm.archlinux.page/specifications/alpm-package-source-checksum.7.html

use alpm_types::{Architecture, ChecksumAlgorithm};
use documented::Documented;
use strum::VariantArray;

use crate::{
    internal_prelude::*,
    issue::SourceInfoIssue,
    lint_rules::source_info::source_info_from_resource,
};

/// # What it does
///
/// Ensures that each [alpm-package-source-checksum] in [SRCINFO] data uses a safe hash function.
///
/// # Why is this bad?
///
/// Upstream artifacts are validated against hash digests (see [alpm-package-source-checksum]) set
/// in [PKGBUILD] and [SRCINFO] files.
///
/// Some [hash functions] (e.g. [MD-5] and [SHA-1]) used for creating these hash digests are unsafe
/// to use from a cryptographic perspective. These algorithms should be avoided to prevent hash
/// collisions and potential abuse.
///
/// Using unsafe hash algorithms allows attackers to craft malicious artifacts that pass the
/// checksum check. Further, attackers could swap existing artifacts with these malicious artifacts
/// and compromise a package on (re)build.
///
/// # Example
///
/// ```ini,ignore
/// pkgbase = test
///     pkgver = 1.0.0
///     pkgrel = 1
///     arch = x86_64
///     source = https://domain.tld/testing/x86_64_test.tar.gz
///     md5sums = 10245815f893d79f3d779690774f0b43
/// ```
///
/// Use instead:
///
/// ```ini,ignore
/// pkgbase = test
///     pkgver = 1.0.0
///     pkgrel = 1
///     arch = x86_64
///     source = https://domain.tld/testing/x86_64_test.tar.gz
///     sha512sums = 1816c57b4abf31eb7c57a66bfb0f0ee5cef9398b5e4cc303468e08dae2702da55978402da94673e444f8c02754e94dedef4d12450319383c3a481d1c5cd90c82
/// ```
///
/// [MD-5]: https://en.wikipedia.org/wiki/MD-5
/// [PKGBUILD]: https://man.archlinux.org/man/PKGBUILD.5
/// [RFC 0046]: https://rfc.archlinux.page/0046-upstream-package-sources/
/// [SHA-1]: https://en.wikipedia.org/wiki/SHA-1
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
/// [alpm-package-source-checksum]: https://alpm.archlinux.page/specifications/alpm-package-source-checksum.7.html
/// [hash functions]: https://en.wikipedia.org/wiki/Hash_function
#[derive(Clone, Debug, Documented)]
pub struct UnsafeChecksum {}

impl UnsafeChecksum {
    /// Create a new, boxed instance of [`UnsafeChecksum`].
    pub fn new_boxed(_: &LintRuleConfiguration) -> Box<dyn LintRule> {
        Box::new(Self {})
    }

    /// Helper function to create a lint issue for unsafe checksum field.
    fn create_checksum_issue(
        &self,
        field_name: &str,
        value: &str,
        architecture: Option<Architecture>,
    ) -> LintIssue {
        LintIssue::from_rule(
            self,
            SourceInfoIssue::BaseField {
                field_name: field_name.to_string(),
                value: value.to_string(),
                context: "Unsafe algorithm".to_string(),
                architecture,
            }
            .into(),
        )
    }
}

impl LintRule for UnsafeChecksum {
    fn name(&self) -> &'static str {
        "unsafe_checksum"
    }

    fn scope(&self) -> LintScope {
        LintScope::SourceInfo
    }

    fn level(&self) -> Level {
        crate::Level::Deny
    }

    fn documentation(&self) -> String {
        UnsafeChecksum::DOCS.into()
    }

    fn help_text(&self) -> String {
        format!(
            r#"Some hash algorithms are deprecated: {}.
Using deprecated checksum algorithms for the verification of artifacts is a security risk.

Instead, use one of the following algorithms: {}
"#,
            ChecksumAlgorithm::VARIANTS
                .iter()
                .filter(|algo| algo.is_deprecated())
                .map(|var| var.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            ChecksumAlgorithm::VARIANTS
                .iter()
                .filter(|algo| !algo.is_deprecated())
                .map(|var| var.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }

    fn run(&self, resources: &Resources, issues: &mut Vec<LintIssue>) -> Result<(), Error> {
        // Extract the SourceInfo from the given resources.
        let source_info = source_info_from_resource(resources, self.scoped_name())?;
        let base = &source_info.base;

        // Check for SHA1 checksums - these are unsafe
        for (_source, checksum) in base.sources.iter().zip(base.sha1_checksums.iter()) {
            if !checksum.is_skipped() {
                issues.push(self.create_checksum_issue("sha1sums", &checksum.to_string(), None));
            }
        }

        // Check for MD5 checksums - these are unsafe
        for (_source, checksum) in base.sources.iter().zip(base.md5_checksums.iter()) {
            if !checksum.is_skipped() {
                issues.push(self.create_checksum_issue("md5sums", &checksum.to_string(), None));
            }
        }

        // Also check architecture-specific checksums
        for (architecture, arch_props) in &base.architecture_properties {
            // Check SHA1 checksums in architecture-specific properties
            for (_source, checksum) in arch_props
                .sources
                .iter()
                .zip(arch_props.sha1_checksums.iter())
            {
                if !checksum.is_skipped() {
                    issues.push(self.create_checksum_issue(
                        "sha1sums",
                        &checksum.to_string(),
                        Some(*architecture),
                    ));
                }
            }

            // Check MD5 checksums in architecture-specific properties
            for (_source, checksum) in arch_props
                .sources
                .iter()
                .zip(arch_props.md5_checksums.iter())
            {
                if !checksum.is_skipped() {
                    issues.push(self.create_checksum_issue(
                        "md5sums",
                        &checksum.to_string(),
                        Some(*architecture),
                    ));
                }
            }
        }

        Ok(())
    }
}
