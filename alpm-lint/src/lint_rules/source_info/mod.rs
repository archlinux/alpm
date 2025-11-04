//! All lints for [SRCINFO] files and data.
//!
//! [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html

use alpm_srcinfo::{SourceInfo, SourceInfoV1};

use crate::{Error, LintScope, Resources};

pub mod duplicate_architecture;
pub mod invalid_spdx_license;
pub mod no_architecture;
pub mod openpgp_key_id;
pub mod undefined_architecture;
pub mod unknown_architecture;
pub mod unsafe_checksum;

/// Extracts a [`SourceInfoV1`] from a [`Resources`].
///
/// # Note
///
/// The `lint_rule` needs to be provided to provide a meaningful message in case of an error.
///
/// # Errors
///
/// Returns an error if `resources` does not contain [`SourceInfo`] data.
fn source_info_from_resource(
    resources: &Resources,
    lint_rule: String,
) -> Result<&SourceInfoV1, Error> {
    match resources {
        Resources::SourceRepository {
            source_info: SourceInfo::V1(source_info),
            ..
        }
        | Resources::SourceInfo(SourceInfo::V1(source_info)) => Ok(source_info),
        _ => Err(Error::InvalidResources {
            scope: resources.scope(),
            lint_rule,
            expected: LintScope::SourceInfo,
        }),
    }
}
