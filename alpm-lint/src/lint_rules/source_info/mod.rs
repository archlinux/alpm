//! All lints for `.SRCINFO` files and data.

use alpm_srcinfo::{SourceInfo, SourceInfoV1};

use crate::{Error, LintScope, Resources};

pub mod duplicate_architecture;
pub mod no_architecture;
pub mod no_spdx_license;
pub mod openpgp_key_id;
pub mod undefined_architecture;
pub mod unsafe_checksum;

/// Extract the required resources.
///
/// The lint_rule must be provided in case of an error
///
/// # Errors
///
/// Returns an error if the given resource type doesn't contain any type of SourceInfo data.
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
