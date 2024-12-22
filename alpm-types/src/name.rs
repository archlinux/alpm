use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    string::ToString,
};

use lazy_regex::{lazy_regex, Lazy};
use regex::Regex;

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
/// // create BuildTool from &str
/// assert!(BuildTool::from_str("test-123@.foo_+").is_ok());
/// assert!(BuildTool::from_str(".test").is_err());
///
/// // format as String
/// assert_eq!("foo", format!("{}", BuildTool::from_str("foo").unwrap()));
///
/// // validate that BuildTool follows naming restrictions
/// let buildtool = BuildTool::from_str("foo").unwrap();
/// let restrictions = vec![
///     Name::from_str("foo").unwrap(),
///     Name::from_str("bar").unwrap(),
/// ];
/// assert!(buildtool.matches_restriction(&restrictions));
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
    /// assert!(
    ///     BuildTool::new_with_restriction("foo", &[Name::new("foo".to_string()).unwrap()]).is_ok()
    /// );
    /// assert!(
    ///     BuildTool::new_with_restriction("foo", &[Name::new("bar".to_string()).unwrap()]).is_err()
    /// );
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
        Name::new(s.to_string()).map(BuildTool)
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
/// // create Name from &str
/// assert_eq!(
///     Name::from_str("test-123@.foo_+"),
///     Ok(Name::new("test-123@.foo_+".to_string()).unwrap())
/// );
/// assert!(Name::from_str(".test").is_err());
///
/// // format as String
/// assert_eq!("foo", format!("{}", Name::new("foo".to_string()).unwrap()));
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Name(String);

impl Name {
    /// Create a new `Name`
    pub fn new(name: String) -> Result<Self, Error> {
        Self::from_str(&name)
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
}
