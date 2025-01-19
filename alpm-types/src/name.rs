use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    string::ToString,
};

use lazy_regex::{Lazy, lazy_regex};
use regex::Regex;
use serde::Serialize;
use winnow::{
    ModalResult,
    Parser,
    combinator::{alt, cut_err, eof, peek, repeat, repeat_till},
    error::{StrContext, StrContextValue},
    token::any,
};

use crate::Error;

pub(crate) static NAME_REGEX: Lazy<Regex> = lazy_regex!(r"^[a-zA-Z\d_@+]+[a-zA-Z\d\-._@+]*$");

/// A build tool name
///
/// The same character restrictions as with `Name` apply.
/// Further name restrictions may be enforced on an existing instances using
/// `matches_restriction()`.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{BuildTool, Error, Name};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // create BuildTool from &str
/// assert!(BuildTool::from_str("test-123@.foo_+").is_ok());
/// assert!(BuildTool::from_str(".test").is_err());
///
/// // format as String
/// assert_eq!("foo", format!("{}", BuildTool::from_str("foo")?));
///
/// // validate that BuildTool follows naming restrictions
/// let buildtool = BuildTool::from_str("foo")?;
/// let restrictions = vec![Name::from_str("foo")?, Name::from_str("bar")?];
/// assert!(buildtool.matches_restriction(&restrictions));
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BuildTool(Name);

impl BuildTool {
    /// Create a new BuildTool
    pub fn new(name: Name) -> Self {
        BuildTool(name)
    }

    /// Create a new BuildTool in a Result, which matches one Name in a list of restrictions
    ///
    /// ## Examples
    /// ```
    /// use alpm_types::{BuildTool, Error, Name};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// assert!(BuildTool::new_with_restriction("foo", &[Name::new("foo")?]).is_ok());
    /// assert!(BuildTool::new_with_restriction("foo", &[Name::new("bar")?]).is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_with_restriction(name: &str, restrictions: &[Name]) -> Result<Self, Error> {
        let buildtool = BuildTool::from_str(name)?;
        if buildtool.matches_restriction(restrictions) {
            Ok(buildtool)
        } else {
            Err(Error::ValueDoesNotMatchRestrictions {
                restrictions: restrictions.iter().map(ToString::to_string).collect(),
            })
        }
    }

    /// Validate that the BuildTool has a name matching one Name in a list of restrictions
    pub fn matches_restriction(&self, restrictions: &[Name]) -> bool {
        restrictions
            .iter()
            .any(|restriction| restriction.eq(self.inner()))
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &Name {
        &self.0
    }
}

impl FromStr for BuildTool {
    type Err = Error;
    /// Create a BuildTool from a string
    fn from_str(s: &str) -> Result<BuildTool, Self::Err> {
        Name::new(s).map(BuildTool)
    }
}

impl Display for BuildTool {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

/// A package name
///
/// Package names may contain the characters `[a-z\d\-._@+]`, but must not
/// start with `[-.]`.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Error, Name};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // create Name from &str
/// assert_eq!(
///     Name::from_str("test-123@.foo_+"),
///     Ok(Name::new("test-123@.foo_+")?)
/// );
/// assert!(Name::from_str(".test").is_err());
///
/// // format as String
/// assert_eq!("foo", format!("{}", Name::new("foo")?));
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Name(String);

impl Name {
    /// Create a new `Name`
    pub fn new(name: &str) -> Result<Self, Error> {
        Self::from_str(name)
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &str {
        &self.0
    }
}

impl FromStr for Name {
    type Err = Error;
    /// Create a Name from a string
    fn from_str(s: &str) -> Result<Name, Self::Err> {
        if NAME_REGEX.is_match(s) {
            Ok(Name(s.to_string()))
        } else {
            Err(Error::RegexDoesNotMatch {
                value: s.to_string(),
                regex_type: "pkgname".to_string(),
                regex: NAME_REGEX.to_string(),
            })
        }
    }
}

impl Display for Name {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        self.inner()
    }
}

/// A shared object name.
///
/// This type wraps a [`Name`] and is used to represent the name of a shared object file
/// that ends with the `.so` suffix.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedObjectName(Name);

impl SharedObjectName {
    /// Creates a new [`SharedObjectName`].
    ///
    /// # Errors
    ///
    /// Returns an error if the input does not end with `.so`.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_types::SharedObjectName;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let shared_object_name = SharedObjectName::new("example.so")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(name: &str) -> Result<Self, Error> {
        Self::from_str(name)
    }

    /// Returns the name of the shared object as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    /// Parses a [`SharedObjectName`] from a string slice.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        // Save the input for parsing the full name
        let raw_input = input.to_string();

        // Parse the name of the shared object until eof or the `.so` is hit.
        repeat_till::<_, _, String, _, _, _, _>(1.., any, peek(alt((".so", eof))))
            .context(StrContext::Label("name"))
            .parse_next(input)?;

        // Parse at least one or more `.so` suffix(es).
        cut_err(repeat::<_, _, String, _, _>(1.., ".so").take())
            .context(StrContext::Label("suffix"))
            .context(StrContext::Expected(StrContextValue::Description(
                "shared object name suffix '.so'",
            )))
            .parse_next(input)?;

        // Ensure that there is no trailing content
        cut_err(eof)
            .context(StrContext::Label(
                "unexpected trailing content after shared object name.",
            ))
            .context(StrContext::Expected(StrContextValue::Description(
                "end of input.",
            )))
            .parse_next(input)?;

        let name = repeat_till(1.., any, eof)
            .try_map(|(name, _): (String, &str)| Name::from_str(&name))
            .context(StrContext::Label("name"))
            .parse_next(&mut raw_input.as_str())?;

        Ok(SharedObjectName(name))
    }
}

impl FromStr for SharedObjectName {
    type Err = Error;
    /// Create an [`SharedObjectName`] from a string and return it in a Result
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

impl Display for SharedObjectName {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(
        "bar",
        ["foo".parse(), "bar".parse()].into_iter().flatten().collect::<Vec<Name>>(),
        Ok(BuildTool::from_str("bar").unwrap()),
    )]
    #[case(
        "bar",
        ["foo".parse(), "foo".parse()].into_iter().flatten().collect::<Vec<Name>>(),
        Err(Error::ValueDoesNotMatchRestrictions {
            restrictions: vec!["foo".to_string(), "foo".to_string()],
        }),
    )]
    fn buildtool_new_with_restriction(
        #[case] buildtool: &str,
        #[case] restrictions: Vec<Name>,
        #[case] result: Result<BuildTool, Error>,
    ) {
        assert_eq!(
            BuildTool::new_with_restriction(buildtool, &restrictions),
            result
        );
    }

    #[rstest]
    #[case("bar", ["foo".parse(), "bar".parse()].into_iter().flatten().collect::<Vec<Name>>(), true)]
    #[case("bar", ["foo".parse(), "foo".parse()].into_iter().flatten().collect::<Vec<Name>>(), false)]
    fn buildtool_matches_restriction(
        #[case] buildtool: &str,
        #[case] restrictions: Vec<Name>,
        #[case] result: bool,
    ) {
        let buildtool = BuildTool::from_str(buildtool).unwrap();
        assert_eq!(buildtool.matches_restriction(&restrictions), result);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn valid_name_from_string(name_str in r"[a-zA-Z\d_@+]+[a-zA-Z\d\-._@+]*") {
            let name = Name::from_str(&name_str).unwrap();
            prop_assert_eq!(name_str, format!("{}", name));
        }

        #[test]
        fn invalid_name_from_string_start(name_str in r"[-.][a-zA-Z0-9@._+-]*") {
            let error = Name::from_str(&name_str).unwrap_err();
            assert_eq!(error, Error::RegexDoesNotMatch {
                value: name_str.to_string(),
                regex_type: "pkgname".to_string(),
                regex: NAME_REGEX.to_string(),
            });
        }

        #[test]
        fn invalid_name_with_invalid_characters(name_str in r"[^\w@._+-]+") {
            let error = Name::from_str(&name_str).unwrap_err();
            assert_eq!(error, Error::RegexDoesNotMatch {
                value: name_str.to_string(),
                regex_type: "pkgname".to_string(),
                regex: NAME_REGEX.to_string(),
            });
        }
    }

    #[rstest]
    #[case("example.so", SharedObjectName("example.so".parse().unwrap()))]
    #[case("example.so.so", SharedObjectName("example.so.so".parse().unwrap()))]
    #[case("libexample.1.so", SharedObjectName("libexample.1.so".parse().unwrap()))]
    fn shared_object_name_parser(
        #[case] input: &str,
        #[case] expected_result: SharedObjectName,
    ) -> testresult::TestResult<()> {
        let shared_object_name = SharedObjectName::new(input)?;
        assert_eq!(expected_result, shared_object_name);
        assert_eq!(input, shared_object_name.as_str());
        Ok(())
    }

    #[rstest]
    #[case("noso", "expected shared object name suffix '.so'")]
    #[case("example.so.1", "unexpected trailing content after shared object name")]
    fn invalid_shared_object_name_parser(#[case] input: &str, #[case] error_snippet: &str) {
        let result = SharedObjectName::from_str(input);
        assert!(result.is_err(), "Expected SharedObjectName parsing to fail");
        let err = result.unwrap_err();
        let pretty_error = err.to_string();
        assert!(
            pretty_error.contains(error_snippet),
            "Error:\n=====\n{pretty_error}\n=====\nshould contain snippet:\n\n{error_snippet}"
        );
    }
}
