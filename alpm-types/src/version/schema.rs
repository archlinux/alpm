//! Schema version handling.

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use semver::Version as SemverVersion;

use crate::Error;

/// The schema version of a type
///
/// A `SchemaVersion` wraps a `semver::Version`, which means that the tracked version should follow [semver](https://semver.org).
/// However, for backwards compatibility reasons it is possible to initialize a `SchemaVersion`
/// using a non-semver compatible string, *if* it can be parsed to a single `u64` (e.g. `"1"`).
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::SchemaVersion;
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // create SchemaVersion from str
/// let version_one = SchemaVersion::from_str("1.0.0")?;
/// let version_also_one = SchemaVersion::from_str("1")?;
/// assert_eq!(version_one, version_also_one);
///
/// // format as String
/// assert_eq!("1.0.0", format!("{}", version_one));
/// assert_eq!("1.0.0", format!("{}", version_also_one));
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SchemaVersion(SemverVersion);

impl SchemaVersion {
    /// Create a new SchemaVersion
    pub fn new(version: SemverVersion) -> Self {
        SchemaVersion(version)
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &SemverVersion {
        &self.0
    }
}

impl FromStr for SchemaVersion {
    type Err = Error;
    /// Create a new SchemaVersion from a string
    ///
    /// When providing a non-semver string with only a number (i.e. no minor or patch version), the
    /// number is treated as the major version (e.g. `"23"` -> `"23.0.0"`).
    fn from_str(s: &str) -> Result<SchemaVersion, Self::Err> {
        if !s.contains('.') {
            match s.parse() {
                Ok(major) => Ok(SchemaVersion(SemverVersion::new(major, 0, 0))),
                Err(e) => Err(Error::InvalidInteger { kind: *e.kind() }),
            }
        } else {
            match SemverVersion::parse(s) {
                Ok(version) => Ok(SchemaVersion(version)),
                Err(e) => Err(Error::InvalidSemver {
                    kind: e.to_string(),
                }),
            }
        }
    }
}

impl Display for SchemaVersion {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("1.0.0", Ok(SchemaVersion(SemverVersion::new(1, 0, 0))))]
    #[case("1", Ok(SchemaVersion(SemverVersion::new(1, 0, 0))))]
    #[case("-1.0.0", Err(Error::InvalidSemver { kind: String::from("unexpected character '-' while parsing major version number") }))]
    fn schema_version(#[case] version: &str, #[case] result: Result<SchemaVersion, Error>) {
        assert_eq!(result, SchemaVersion::from_str(version))
    }

    #[rstest]
    #[case(
        SchemaVersion(SemverVersion::new(1, 0, 0)),
        SchemaVersion(SemverVersion::new(0, 1, 0))
    )]
    fn compare_schema_version(#[case] version_a: SchemaVersion, #[case] version_b: SchemaVersion) {
        assert!(version_a > version_b);
    }
}
