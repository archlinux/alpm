use std::{convert::Infallible, fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::{Error, Name};

/// The type of a package
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::PackageType;
///
/// // create PackageType from str
/// assert_eq!(PackageType::from_str("pkg"), Ok(PackageType::Package));
///
/// // format as String
/// assert_eq!("debug", format!("{}", PackageType::Debug));
/// assert_eq!("pkg", format!("{}", PackageType::Package));
/// assert_eq!("src", format!("{}", PackageType::Source));
/// assert_eq!("split", format!("{}", PackageType::Split));
/// ```
#[derive(Clone, Copy, Debug, Display, EnumString, Eq, PartialEq, Serialize)]
pub enum PackageType {
    /// a debug package
    #[strum(to_string = "debug")]
    Debug,
    /// a single (non-split) package
    #[strum(to_string = "pkg")]
    Package,
    /// a source-only package
    #[strum(to_string = "src")]
    Source,
    /// one split package out of a set of several
    #[strum(to_string = "split")]
    Split,
}

/// Description of a package
///
/// This type enforces the following invariants on the contained string:
/// - No leading/trailing spaces
/// - Tabs and newlines are substituted with spaces.
/// - Multiple, consecutive spaces are substituted with a single space.
///
/// This is a type alias for [`String`].
///
/// ## Examples
///
/// ```
/// use alpm_types::PackageDescription;
///
/// # fn main() {
/// // Create PackageDescription from a string slice
/// let description = PackageDescription::from("my special package ");
///
/// assert_eq!(&description.to_string(), "my special package");
/// # }
/// ```
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PackageDescription(String);

impl PackageDescription {
    /// Create a new `PackageDescription` from a given `String`.
    pub fn new(description: &str) -> Self {
        Self::from(description)
    }
}

impl FromStr for PackageDescription {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl AsRef<str> for PackageDescription {
    /// Returns a reference to the inner [`String`].
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<&str> for PackageDescription {
    /// Creates a new [`PackageDescription`] from a string slice.
    ///
    /// Trims leading and trailing whitespace.
    /// Replaces any new lines and tabs with a space.
    /// Replaces any consecutive spaces with a single space.
    fn from(value: &str) -> Self {
        // Trim front and back and replace unwanted whitespace chars.
        let mut description = value.trim().replace(['\n', '\r', '\t'], " ");

        // Remove all spaces that follow a space.
        let mut previous = ' ';
        description.retain(|ch| {
            if ch == ' ' && previous == ' ' {
                return false;
            };
            previous = ch;
            true
        });

        Self(description)
    }
}

impl Display for PackageDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Name of the base package information that one or more packages are built from.
///
/// This is a type alias for [`Name`].
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Error, Name};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // create PackageBaseName from &str
/// let pkgbase = Name::from_str("test-123@.foo_+")?;
///
/// // format as String
/// let pkgbase = Name::from_str("foo")?;
/// assert_eq!("foo", pkgbase.to_string());
/// # Ok(())
/// # }
/// ```
pub type PackageBaseName = Name;

/// Extra data associated with a package
///
/// This type wraps a key-value pair of data as String, which is separated by an equal sign (`=`).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ExtraData {
    key: String,
    value: String,
}

impl ExtraData {
    /// Create a new extra_data
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }

    /// Return the key of the extra_data
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Return the value of the extra_data
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl FromStr for ExtraData {
    type Err = Error;

    /// Parses an `extra_data` from string.
    ///
    /// The string is expected to be in the format `key=value`.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the string is missing the key or value component.
    ///
    /// ## Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::{Error, ExtraData, PackageType};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// // create ExtraData from str
    /// let extra_data: ExtraData = ExtraData::from_str("pkgtype=debug")?;
    /// assert_eq!(extra_data.key(), "pkgtype");
    /// assert_eq!(extra_data.value(), "debug");
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const DELIMITER: char = '=';
        let mut parts = s.splitn(2, DELIMITER);
        let key = parts
            .next()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
            .ok_or(Error::MissingComponent { component: "key" })?;
        let value = parts
            .next()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
            .ok_or(Error::MissingComponent { component: "value" })?;
        Ok(Self::new(key.to_string(), value.to_string()))
    }
}

impl Display for ExtraData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.key, self.value)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("debug", Ok(PackageType::Debug))]
    #[case("pkg", Ok(PackageType::Package))]
    #[case("src", Ok(PackageType::Source))]
    #[case("split", Ok(PackageType::Split))]
    #[case("foo", Err(strum::ParseError::VariantNotFound))]
    fn pkgtype_from_string(
        #[case] from_str: &str,
        #[case] result: Result<PackageType, strum::ParseError>,
    ) {
        assert_eq!(PackageType::from_str(from_str), result);
    }

    #[rstest]
    #[case(PackageType::Debug, "debug")]
    #[case(PackageType::Package, "pkg")]
    #[case(PackageType::Source, "src")]
    #[case(PackageType::Split, "split")]
    fn pkgtype_format_string(#[case] pkgtype: PackageType, #[case] pkgtype_str: &str) {
        assert_eq!(pkgtype_str, format!("{pkgtype}"));
    }

    #[rstest]
    #[case("key=value", "key", "value")]
    #[case("pkgtype=debug", "pkgtype", "debug")]
    #[case("test-123@.foo_+=1000", "test-123@.foo_+", "1000")]
    fn extra_data_from_str(
        #[case] data: &str,
        #[case] key: &str,
        #[case] value: &str,
    ) -> testresult::TestResult<()> {
        let extra_data: ExtraData = ExtraData::from_str(data)?;
        assert_eq!(extra_data.key(), key);
        assert_eq!(extra_data.value(), value);
        assert_eq!(extra_data.to_string(), data);
        Ok(())
    }

    #[rstest]
    #[case("key", Err(Error::MissingComponent { component: "value" }))]
    #[case("key=", Err(Error::MissingComponent { component: "value" }))]
    #[case("=value", Err(Error::MissingComponent { component: "key" }))]
    fn extra_data_from_str_error(
        #[case] extra_data: &str,
        #[case] result: Result<ExtraData, Error>,
    ) {
        assert_eq!(ExtraData::from_str(extra_data), result);
    }

    #[rstest]
    #[case("  trailing  ", "trailing")]
    #[case("in    between    words", "in between words")]
    #[case("\nsome\t whitespace\n chars\n", "some whitespace chars")]
    #[case("  \neverything\t   combined\n yeah \n   ", "everything combined yeah")]
    fn package_description(#[case] input: &str, #[case] result: &str) {
        assert_eq!(PackageDescription::new(input).to_string(), result);
    }
}
