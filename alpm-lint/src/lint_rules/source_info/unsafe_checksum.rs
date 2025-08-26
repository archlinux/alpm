//! The file verification for some source in a .SRCINFO file uses an unsafe hash algorithm.

use alpm_types::{Architecture, ChecksumAlgorithm};
use documented::Documented;
use strum::VariantArray;

use crate::{
    internal_prelude::*,
    issue::SourceInfoIssue,
    lint_rules::source_info::source_info_from_resource,
};

/// ### What it does
///
/// Ensures that no cryptographically unsafe hashing algorithms are used.
/// Right now, the following algorithms are considered unsafe:
///
/// - MD5: Vulnerable to collision attacks
/// - SHA1: Vulnerable to collision attacks
///
/// ### Why is this bad?
///
/// Upstream artifacts are checked via these hashes. Using insecure hash algorithms would allow
/// potential attackers to craft malicious artifacts that pass the checksum check.
///
/// Attackers could then to swap existing artifacts with these malicious artifacts. Any packages
/// that're then re-build on those artifacts would be compromised, without any indicator that
/// something changed.
///
/// ### Example
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
#[derive(Clone, Debug, Documented)]
pub struct UnsafeChecksum {}

impl UnsafeChecksum {
    /// Create a new, boxed instance of [`UnsafeChecksum`].
    ///
    /// This is used to register the lint on the `LintStore`.
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

    fn groups(&self) -> &'static [LintGroup] {
        &[LintGroup::Pedantic]
    }

    fn documentation(&self) -> String {
        UnsafeChecksum::DOCS.into()
    }

    fn help_text(&self) -> String {
        format!(
            r#"Some checksum algorithms, such as `MD5` or `SHA1`, have been deprecated.
Using such checksum algorithm to verify downloaded source artifacts is a security risk.

Instead, use one of the following algorithms instead: {}
"#,
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
