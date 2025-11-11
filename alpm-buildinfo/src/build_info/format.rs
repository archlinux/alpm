use alpm_common::FileFormatSchema;
use alpm_types::SchemaVersion;
use serde_with::{DisplayFromStr, serde_as};

use crate::BuildInfoSchema;

/// Used internally to detect the BUILDINFO version when deserializing.
#[serde_as]
#[derive(Clone, Debug, serde::Deserialize)]
pub(crate) struct BuildInfoFormat {
    #[serde_as(as = "DisplayFromStr")]
    pub format: BuildInfoSchema,
}

impl From<BuildInfoFormat> for SchemaVersion {
    fn from(format: BuildInfoFormat) -> Self {
        format.format.inner().clone()
    }
}
