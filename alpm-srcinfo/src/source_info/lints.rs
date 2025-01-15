use alpm_types::Architecture;

use crate::error::{lint, SourceInfoError};

/// Creates a parse error for unsafe checksums.
/// Checksums that're considered unsafe by us are marked such on the [`alpm_types::Checksum`]
/// struct
pub fn unsafe_checksum(errors: &mut Vec<SourceInfoError>, line: usize, digest: &str) {
    errors.push(lint(
        Some(line),
        format!("Found discouraged checksum of type {digest}, as it's cryptographically unsafe."),
    ));
}

/// Creates a lint error for architecture specific properties when that architecture doesn't exist
/// for a given `PackageBuild` or `Package`.
///
/// For example, the following pseudo SRCINFO file would create this lint:
///
/// ```txt
/// pkgbase = foo
/// ...
///   arch = (x86_64)
///   depends_aarch64 = glibc
/// ...
/// ```
pub fn missing_architecture_for_property(
    errors: &mut Vec<SourceInfoError>,
    line: usize,
    architecture: Architecture,
) {
    errors.push(lint(
        Some(line),
        format!(
            "Found {architecture} specific property, but {architecture} isn't specified in 'arch'"
        ),
    ));
}

/// Creates a lint error for when an architecture is specified multiple times.
/// For example: `arch = (x86_64 x86_64)`
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
/// Take a look at [`alpm_types::License`] for more information about this format.
pub fn non_spdx_license(errors: &mut Vec<SourceInfoError>, line: usize, license: String) {
    errors.push(lint(
        Some(line),
        format!("Found license declaration that's either not in the SPDX format or not supported by SPDX: {license}"),
    ));
}

/// A property that's both cleared and declared in the context of a single package.
/// This is considered a bad practice as any property declaration in the context of a package
/// implicitly overwrites any defaults from the `PackageBase`, making the clear unnecessary.
///
/// Example:
/// ```txt
/// depends =
/// depends = glibc
/// ```
///
/// In the case property is cleared that has been declared beforehand, this might even be unwanted
/// behavior.
///
/// Example:
/// ```txt
/// depends = glibc
/// depends =
/// ```
pub fn reassigned_cleared_property(errors: &mut Vec<SourceInfoError>, line: usize) {
    errors.push(lint(
        Some(line),
        "This property is being set even though that property is also explicitly cleared in this package.",
    ));
}
