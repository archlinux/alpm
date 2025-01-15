use alpm_types::Architecture;

use crate::error::{lint, SourceInfoError};

pub fn unsafe_checksum(errors: &mut Vec<SourceInfoError>, line: usize, digest: &str) {
    errors.push(lint(
        Some(line),
        format!("Found discouraged checksum of type {digest}, as it's cryptographically unsafe."),
    ));
}

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

pub fn non_spdx_license(errors: &mut Vec<SourceInfoError>, line: usize, license: String) {
    errors.push(lint(
        Some(line),
        format!("Found license declaration that's either not in the SPDX format or not supported by SPDX: {license}"),
    ));
}
