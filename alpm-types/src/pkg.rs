use std::{convert::Infallible, fmt::Display, str::FromStr};

use alpm_parsers::{iter_str_context, prelude::*};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::{Display, EnumString, VariantNames};
use winnow::{
    Parser,
    ascii::{alpha1, space0},
    combinator::{alt, not, peek, repeat_till},
    error::{ErrMode, StrContext, StrContextValue},
    token::any,
};

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
#[derive(Clone, Copy, Debug, Display, EnumString, Eq, PartialEq, Serialize, VariantNames)]
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

impl AlpmParser for PackageType {
    /// Recognizes a [`PackageType`] in a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if the immediate alphanumeric `input` is not a valid variant
    /// a `PackageType`.
    fn parser<'a>(input: &mut Input<'a>) -> PResult<'a, Self> {
        alpha1
            .try_map(PackageType::from_str)
            .context(StrContext::Label("package type"))
            .context_with(iter_str_context!([PackageType::VARIANTS]))
            .parse_next(input)
    }

    fn delimiter_error_context<'a, O, P>(
        parser: P,
    ) -> impl Parser<Input<'a>, O, ErrMode<ParseStack<'a>>>
    where
        P: Parser<Input<'a>, O, ErrMode<ParseStack<'a>>>,
    {
        parser
            .context(StrContext::Label("package type"))
            .context(StrContext::Expected(StrContextValue::Description(
                "an alphanbetic string",
            )))
    }
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

impl Default for PackageDescription {
    /// Returns the default [`PackageDescription`].
    ///
    /// Following the default for [`String`], this returns a [`PackageDescription`] wrapping an
    /// empty string.
    fn default() -> Self {
        Self::new("")
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

/// Extra data entry associated with a package
///
/// This type wraps a key-value pair of data as String, which is separated by an equal sign (`=`).
#[derive(Clone, Debug, DeserializeFromStr, PartialEq, SerializeDisplay)]
pub struct ExtraDataEntry {
    key: String,
    value: String,
}

impl ExtraDataEntry {
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

impl ParserUntil for ExtraDataEntry {
    /// Parses a [`ExtraDataEntry`] from a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` does not represent a valid [`ExtraDataEntry`].
    fn parser_until<'a, P>(delimiter: P) -> impl Parser<Input<'a>, Self, ErrMode<ParseStack<'a>>>
    where
        P: Parser<Input<'a>, &'a str, ErrMode<ParseStack<'a>>>,
    {
        // Define the actual parser closure.
        // The delimiter is moved into the closure and borrowed via `by_ref()` on each call.
        let mut delimiter_parser = delimiter;
        move |input: &mut Input<'a>| -> PResult<'a, Self> {
            // Handle the case were there's no key
            not("=")
                .context(StrContext::Label("extra data"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "a utf-8 key before the `=` delimiter",
                )))
                .parse_next(input)?;

            let key: &str = repeat_till::<_, _, (), _, _, _, _>(
                1..,
                any,
                peek(alt((
                    (space0, "=", space0).take(),
                    delimiter_parser.by_ref(),
                ))),
            )
            .take()
            .context(StrContext::Label("extra data key"))
            .context(StrContext::Expected(StrContextValue::Description(
                "an utf-8 string, followed by a `=`.",
            )))
            .parse_next(input)?;

            (space0, "=", space0)
                .context(StrContext::Label("extra data delimiter"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "a `=` between the key and value",
                )))
                .parse_next(input)?;

            let value: &str =
                repeat_till::<_, _, (), _, _, _, _>(1.., any, peek(delimiter_parser.by_ref()))
                    .take()
                    .context(StrContext::Label("extra data value"))
                    .context(StrContext::Expected(StrContextValue::Description(
                        "an utf-8 string",
                    )))
                    .parse_next(input)?;

            peek(delimiter_parser.by_ref())
                .context(StrContext::Label("extra data value"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "end of input",
                )))
                .parse_next(input)?;

            Ok(Self::new(key.trim().to_string(), value.trim().to_string()))
        }
    }
}

impl FromStr for ExtraDataEntry {
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
    /// use alpm_types::{ExtraDataEntry, PackageType};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// // create ExtraDataEntry from str
    /// let extra_data: ExtraDataEntry = ExtraDataEntry::from_str("pkgtype=debug")?;
    /// assert_eq!(extra_data.key(), "pkgtype");
    /// assert_eq!(extra_data.value(), "debug");
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser_until_eof.parse(Input::new(s))?)
    }
}

impl Display for ExtraDataEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.key, self.value)
    }
}

/// Extra data associated with a package.
///
/// This type wraps a vector of [`ExtraDataEntry`] items enforcing that it includes a valid
/// `pkgtype` entry.
///
/// Can be created from a [`Vec<ExtraDataEntry>`] or [`ExtraDataEntry`] using [`TryFrom::try_from`].
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ExtraData(Vec<ExtraDataEntry>);

impl ExtraData {
    /// Returns the package type.
    pub fn pkg_type(&self) -> PackageType {
        self.0
            .iter()
            .find(|v| v.key() == "pkgtype")
            .map(|v| PackageType::from_str(v.value()).expect("Invalid package type"))
            .unwrap_or_else(|| unreachable!("Valid xdata should always contain a pkgtype entry."))
    }

    /// Returns the number of extra data entries.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if there are no extra data entries.
    ///
    /// Due to the invariant enforced in [`TryFrom`], this will always return `false` and is only
    /// included for consistency with [`Vec::is_empty`] in the standard library.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl TryFrom<Vec<ExtraDataEntry>> for ExtraData {
    type Error = Error;

    /// Creates an [`ExtraData`] from a vector of [`ExtraDataEntry`].
    ///
    /// ## Errors
    ///
    /// Returns an error in the following cases:
    ///
    /// - if the `value` does not contain a `pkgtype` key.
    /// - if the `pkgtype` entry does not contain a valid package type.
    fn try_from(value: Vec<ExtraDataEntry>) -> Result<Self, Self::Error> {
        if let Some(pkg_type) = value.iter().find(|v| v.key() == "pkgtype") {
            let _ = PackageType::from_str(pkg_type.value())?;
            Ok(Self(value))
        } else {
            Err(Error::MissingComponent {
                component: "extra_data with a valid \"pkgtype\" entry",
            })
        }
    }
}

impl TryFrom<ExtraDataEntry> for ExtraData {
    type Error = Error;

    /// Creates an [`ExtraData`] from a single [`ExtraDataEntry`].
    ///
    /// Delegates to [`TryFrom::try_from`] for [`Vec<ExtraDataEntry>`].
    ///
    /// ## Errors
    ///
    /// If the [`TryFrom::try_from`] for [`Vec<ExtraDataEntry>`] returns an error.
    fn try_from(value: ExtraDataEntry) -> Result<Self, Self::Error> {
        Self::try_from(vec![value])
    }
}

impl From<ExtraData> for Vec<ExtraDataEntry> {
    /// Converts the [`ExtraData`] into a [`Vec<ExtraDataEntry>`].
    fn from(value: ExtraData) -> Self {
        value.0
    }
}

impl IntoIterator for ExtraData {
    type Item = ExtraDataEntry;
    type IntoIter = std::vec::IntoIter<ExtraDataEntry>;

    /// Consumes the [`ExtraData`] and returns an iterator over [`ExtraDataEntry`] items.
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl AsRef<[ExtraDataEntry]> for ExtraData {
    /// Returns a reference to the inner [`Vec<ExtraDataEntry>`].
    fn as_ref(&self) -> &[ExtraDataEntry] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use insta::assert_snapshot;
    use rstest::rstest;
    use testresult::TestResult;

    use super::*;
    use crate::configure_insta;

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
    fn extra_data_entry_from_str(
        #[case] data: &str,
        #[case] key: &str,
        #[case] value: &str,
    ) -> TestResult {
        let extra_data = ExtraDataEntry::from_str(data)?;
        assert_eq!(extra_data.key(), key);
        assert_eq!(extra_data.value(), value);
        assert_eq!(extra_data.to_string(), data);
        Ok(())
    }

    #[rstest]
    #[case("key")]
    #[case("key=")]
    #[case("=value")]
    fn extra_data_entry_from_str_error(#[case] input: &str) {
        let Err(Error::ParseError(err_msg)) = ExtraDataEntry::from_str(input) else {
            panic!("'{input}' erroneously parsed as a ExtraDataEntry")
        };

        let (test_name, _guard) = configure_insta();
        assert_snapshot!(test_name, err_msg.to_string());
    }

    #[rstest]
    #[case::empty_list(vec![])]
    #[case::invalid_pkgtype(vec![ExtraDataEntry::from_str("pkgtype=foo")?])]
    fn extra_data_invalid(#[case] xdata: Vec<ExtraDataEntry>) -> TestResult {
        assert!(ExtraData::try_from(xdata).is_err());
        Ok(())
    }

    #[rstest]
    #[case::only_pkgtype(vec![ExtraDataEntry::from_str("pkgtype=pkg")?])]
    #[case::with_additional_xdata_entry(vec![ExtraDataEntry::from_str("pkgtype=pkg")?, ExtraDataEntry::from_str("foo=bar")?])]
    fn extra_data_valid(#[case] xdata: Vec<ExtraDataEntry>) -> TestResult {
        let xdata = ExtraData::try_from(xdata)?;
        assert_eq!(xdata.pkg_type(), PackageType::Package);
        Ok(())
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
