use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    str::FromStr,
};

use alpm_parsers::custom_ini::parser::Item;
use alpm_types::{SchemaVersion, semver_version::Version};

use crate::Error;

/// An enum describing all valid BUILDINFO schemas
#[derive(Clone, Debug)]
pub enum BuildInfoSchema {
    V1(SchemaVersion),
    V2(SchemaVersion),
}

impl BuildInfoSchema {
    /// Returns the schema version
    pub fn inner(&self) -> &SchemaVersion {
        match self {
            BuildInfoSchema::V1(v) => v,
            BuildInfoSchema::V2(v) => v,
        }
    }

    /// Determines the schema version from the contents of a BUILDINFO file
    ///
    /// # Errors
    ///
    /// Returns an error if the format field is missing
    pub fn from_contents(contents: &str) -> Result<BuildInfoSchema, Error> {
        // Deserialize the file into a simple map, so we can take a look at the `format` string
        // that determines the buildinfo version.
        let raw_buildinfo: HashMap<String, Item> = alpm_parsers::custom_ini::from_str(contents)?;
        if let Some(Item::Value(version)) = raw_buildinfo.get("format") {
            Self::from_str(version)
        } else {
            Err(Error::MissingFormatField)
        }
    }
}

impl Default for BuildInfoSchema {
    /// Returns the default [`BuildInfoSchema`] variant ([`BuildInfoSchema::V2`])
    fn default() -> Self {
        Self::V2(SchemaVersion::new(Version::new(2, 0, 0)))
    }
}

impl FromStr for BuildInfoSchema {
    type Err = Error;

    /// Uses the `SchemaVersion` to determine the schema
    fn from_str(s: &str) -> Result<BuildInfoSchema, Self::Err> {
        match SchemaVersion::from_str(s) {
            Ok(version) => Self::try_from(version),
            Err(_) => Err(Error::UnsupportedSchemaVersion(s.to_string())),
        }
    }
}

impl TryFrom<SchemaVersion> for BuildInfoSchema {
    type Error = Error;

    /// Converts the major version of the `SchemaVersion` to a `Schema`
    fn try_from(value: SchemaVersion) -> Result<Self, Self::Error> {
        match value.inner().major {
            1 => Ok(BuildInfoSchema::V1(value)),
            2 => Ok(BuildInfoSchema::V2(value)),
            _ => Err(Error::UnsupportedSchemaVersion(value.to_string())),
        }
    }
}

impl Display for BuildInfoSchema {
    /// Converts the `Schema` to a `String`
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                BuildInfoSchema::V1(_) => "1",
                BuildInfoSchema::V2(_) => "2",
            }
        )
    }
}
