use std::fmt::Display;
use std::str::FromStr;

use serde::Serialize;
use strum::{Display, EnumString};

use crate::{Error, Name};

/// The type of a package
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::PkgType;
///
/// // create PkgType from str
/// assert_eq!(PkgType::from_str("pkg"), Ok(PkgType::Package));
///
/// // format as String
/// assert_eq!("debug", format!("{}", PkgType::Debug));
/// assert_eq!("pkg", format!("{}", PkgType::Package));
/// assert_eq!("src", format!("{}", PkgType::Source));
/// assert_eq!("split", format!("{}", PkgType::Split));
/// ```
#[derive(Copy, Clone, Debug, Display, EnumString, Eq, PartialEq, Serialize)]
#[non_exhaustive]
pub enum PkgType {
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
/// This is a type alias for [`String`].
///
/// ## Examples
/// ```
/// use alpm_types::{Error, PkgDesc};
///
/// // Create a PkgDesc
/// let desc: PkgDesc = "A simple package".to_string();
/// ```
pub type PkgDesc = String;

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
#[derive(Debug, Clone, PartialEq, Serialize)]
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
    /// use alpm_types::{Error, ExtraData, PkgType};
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
    #[case("debug", Ok(PkgType::Debug))]
    #[case("pkg", Ok(PkgType::Package))]
    #[case("src", Ok(PkgType::Source))]
    #[case("split", Ok(PkgType::Split))]
    #[case("foo", Err(strum::ParseError::VariantNotFound))]
    fn pkgtype_from_string(
        #[case] from_str: &str,
        #[case] result: Result<PkgType, strum::ParseError>,
    ) {
        assert_eq!(PkgType::from_str(from_str), result);
    }

    #[rstest]
    #[case(PkgType::Debug, "debug")]
    #[case(PkgType::Package, "pkg")]
    #[case(PkgType::Source, "src")]
    #[case(PkgType::Split, "split")]
    fn pkgtype_format_string(#[case] pkgtype: PkgType, #[case] pkgtype_str: &str) {
        assert_eq!(pkgtype_str, format!("{}", pkgtype));
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
}
