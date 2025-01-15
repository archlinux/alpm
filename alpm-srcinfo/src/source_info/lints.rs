//! Linter helper functions.
//!
//! If you find yourself adding a new linting error and it's not just a one-off but rather used in
//! multiple places, please add a new function in this module.
//!
//! All of these linter functions are designed to be used in the context of
//! [`SourceInfo::from_raw`](super::SourceInfo::from_raw) and the functions that're called inside
//! it. They're passed the `errors` vector that aggregates all lints/errors during the conversion
//! from the raw to the structured data format.
use alpm_types::Architecture;

use crate::error::{lint, SourceInfoError};

/// Creates a parse error for unsafe checksums.
///
/// Checksums that are considered cryptographically unsafe are marked as such on the
/// [`alpm_types::Checksum`] struct.
pub fn unsafe_checksum(errors: &mut Vec<SourceInfoError>, line: usize, digest: &str) {
    errors.push(lint(
        Some(line),
        format!(
            "Found cryptographically unsafe checksum type \"{digest}\". Its use is discouraged!"
        ),
    ));
}

/// Creates a lint error for architecture specific properties when that architecture doesn't exist
/// for a given `PackageBuild` or `Package`.
///
/// # Examples
///
/// Parsing the following SRCINFO data triggers the creation of this lint, because an assignment for
/// `depends_aarch64` is present while no `arch = aarch64` assignment exists:
///
/// ```ini
/// pkgbase = example
///   pkgver = 0.1.0
///   pkgrel = 1
///   arch = x86_64
///   depends_aarch64 = glibc
/// pkgname = example
/// ```
pub fn missing_architecture_for_property(
    errors: &mut Vec<SourceInfoError>,
    line: usize,
    architecture: Architecture,
) {
    errors.push(lint(
        Some(line),
        format!(
            "Found keyword specific to \"{architecture}\", but there is no \"arch = {architecture}\" assignment"
        ),
    ));
}

/// Creates a lint error for when an architecture is specified multiple times.
///
/// # Examples
///
/// ```ini
/// pkgbase = example
///   pkgver = 0.1.0
///   pkgrel = 1
///   arch = x86_64
///   arch = x86_64
/// pkgname = example
/// ```
pub fn duplicate_architecture(
    errors: &mut Vec<SourceInfoError>,
    line: usize,
    architecture: Architecture,
) {
    errors.push(lint(
        Some(line),
        format!("Found duplicate architecture declaration: {architecture}"),
    ));
}

/// Creates a lint error for when a license isn't compliant with the SPDX format.
///
/// Take a look at [`alpm_types::License`] for more information about this format.
pub fn non_spdx_license(errors: &mut Vec<SourceInfoError>, line: usize, license: String) {
    errors.push(lint(
        Some(line),
        format!("Found license declaration that's either not in the SPDX format or not supported by SPDX: {license}"),
    ));
}

/// Creates a lint error for a package property that is both set and unset.
///
/// In SRCINFO data, the overriding of a default keyword assignment in a `pkgbase` section works by
/// assigning the keyword a new value in a `pkgname` section. Keyword overriding does not require to
/// first unset a keyword (by assigning it an empty value).
///
/// # Examples
///
/// Parsing the following example SRCINFO data triggers the lint error, because a default set in the
/// `pkgbase` section is first explicitly unset and then explicitly set again in a `pkgname`
/// section. However, simply doing the latter is enough to override the keyword assignment!
///
/// ```ini
/// pkgbase = example
///   pkgver = 0.1.0
///   pkgrel = 1
///   depends = glibc
///
/// pkgname = example
///   # this is not needed!
///   depends =
///   depends = gcc-libs
/// ```
///
/// The following example also triggers this lint error and suggests an error in the software that
/// created the SRCINFO data. While representing legal notation, first setting and then unsetting a
/// keyword is no useful behavior, as both setting and unsetting of the keyword can simply be
/// removed.
///
/// ```ini
/// pkgbase = example
///   pkgver = 0.1.0
///   pkgrel = 1
///   depends = glibc
///
/// pkgname = example
///   depends = gcc-libs
///   # this unsets the previous override for this package!
///   depends =
/// ```
pub fn reassigned_cleared_property(errors: &mut Vec<SourceInfoError>, line: usize) {
    errors.push(lint(
        Some(line),
        "This keyword is set and unset for this package. A keyword should either only be unset or overridden.",
    ));
}
