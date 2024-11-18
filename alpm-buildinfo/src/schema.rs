use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use alpm_types::SchemaVersion;

use crate::Error;

/// An enum describing all valid BUILDINFO schemas
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Schema {
    V1(SchemaVersion),
    V2(SchemaVersion),
}

impl Schema {
    /// Returns the schema version
    pub fn inner(&self) -> &SchemaVersion {
        match self {
            Schema::V1(v) => v,
            Schema::V2(v) => v,
        }
    }
}

impl Default for Schema {
    /// Returns the default schema version which is V1
    ///
    /// # Panics
    ///
    /// Panics if the default schema version cannot be created.
    /// This should not happen normally.
    fn default() -> Self {
        match SchemaVersion::new("1") {
            Ok(v) => Schema::V1(v),
            Err(e) => panic!("failed to create default schema: {e}"),
        }
    }
}

impl FromStr for Schema {
    type Err = Error;

    /// Uses the `SchemaVersion` to determine the schema
    fn from_str(s: &str) -> Result<Schema, Self::Err> {
        match SchemaVersion::from_str(s) {
            Ok(version) => Self::try_from(version),
            Err(_) => Err(Error::InvalidBuildInfoVersion(s.to_string())),
        }
    }
}

impl TryFrom<SchemaVersion> for Schema {
    type Error = Error;

    /// Converts the major version of the `SchemaVersion` to a `Schema`
    fn try_from(value: SchemaVersion) -> Result<Self, Self::Error> {
        match value.inner().major {
            1 => Ok(Schema::V1(value)),
            2 => Ok(Schema::V2(value)),
            _ => Err(Error::InvalidBuildInfoVersion(value.to_string())),
        }
    }
}

impl Display for Schema {
    /// Converts the `Schema` to a `String`
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                Schema::V1(_) => "1",
                Schema::V2(_) => "2",
            }
        )
    }
}
