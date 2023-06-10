// SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
// SPDX-License-Identifier: LGPL-3.0-or-later
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;
use std::string::ToString;

use crate::regex_once;
use crate::Error;

/// A build tool name
///
/// The same character restrictions as with `Name` apply.
/// Further name restrictions may be enforced on an existing instances using `matches_restriction()`.
///
/// ## Examples
/// ```
/// use alpm_types::{BuildTool, Name, Error};
/// use std::str::FromStr;
///
/// // create BuildTool from &str
/// assert_eq!(
///     BuildTool::from_str("test-123@.foo_+"),
///     Ok(BuildTool::new("test-123@.foo_+").unwrap()),
/// );
/// assert_eq!(
///     BuildTool::from_str(".test"),
///     Err(Error::InvalidBuildTool(".test".to_string()))
/// );
///
/// // format as String
/// assert_eq!("foo", format!("{}", BuildTool::new("foo").unwrap()));
///
/// // validate that BuildTool follows naming restrictions
/// let buildtool = BuildTool::new("foo").unwrap();
/// let restrictions = vec![Name::new("foo".to_string()).unwrap(), Name::new("bar".to_string()).unwrap()];
/// assert!(buildtool.matches_restriction(&restrictions));
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BuildTool(Name);

impl BuildTool {
    /// Create a new BuildTool in a Result
    pub fn new(buildtool: &str) -> Result<Self, Error> {
        match Name::new(buildtool.to_string()) {
            Ok(name) => Ok(BuildTool(name)),
            Err(_) => Err(Error::InvalidBuildTool(buildtool.to_string())),
        }
    }

    /// Create a new BuildTool in a Result, which matches one Name in a list of restrictions
    ///
    /// ## Examples
    /// ```
    /// use alpm_types::{BuildTool, Name, Error};
    ///
    /// assert!(BuildTool::new_with_restriction("foo", &[Name::new("foo".to_string()).unwrap()]).is_ok());
    /// assert!(BuildTool::new_with_restriction("foo", &[Name::new("bar".to_string()).unwrap()]).is_err());
    /// ```
    pub fn new_with_restriction(name: &str, restrictions: &[Name]) -> Result<Self, Error> {
        match BuildTool::new(name) {
            Ok(buildtool) => {
                if buildtool.matches_restriction(restrictions) {
                    Ok(buildtool)
                } else {
                    Err(Error::InvalidBuildTool(name.to_string()))
                }
            }
            Err(_) => Err(Error::InvalidBuildTool(name.to_string())),
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
    fn from_str(input: &str) -> Result<BuildTool, Self::Err> {
        BuildTool::new(input)
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
/// use alpm_types::{Name, Error};
/// use std::str::FromStr;
///
/// // create Name from &str
/// assert_eq!(
///     Name::from_str("test-123@.foo_+"),
///     Ok(Name::new("test-123@.foo_+".to_string()).unwrap())
/// );
/// assert_eq!(
///     Name::from_str(".test"),
///     Err(Error::InvalidName(".test".to_string()))
/// );
///
/// // format as String
/// assert_eq!("foo", format!("{}", Name::new("foo".to_string()).unwrap()));
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Name(String);

impl Name {
    /// Create a new Name in a Result
    pub fn new(name: String) -> Result<Self, Error> {
        if regex_once!(r"^[a-z\d_@+]+[a-z\d\-._@+]*$").is_match(name.as_str()) {
            Ok(Name(name))
        } else {
            Err(Error::InvalidName(name))
        }
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &str {
        &self.0
    }
}

impl FromStr for Name {
    type Err = Error;
    /// Create a Name from a string
    fn from_str(input: &str) -> Result<Name, Self::Err> {
        Name::new(input.to_string())
    }
}

impl Display for Name {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use rstest::rstest;

    #[rstest]
    #[case(
        "bar",
        vec![
            Name::new("foo".to_string()).unwrap(),
            Name::new("bar".to_string()).unwrap()
        ],
        Ok(BuildTool::new("bar").unwrap()),
    )]
    #[case(
        "bar",
        vec![
            Name::new("foo".to_string()).unwrap(),
            Name::new("foo".to_string()).unwrap(),
        ],
        Err(Error::InvalidBuildTool("bar".to_string())),
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
    #[case("bar", vec![Name::new("foo".to_string()).unwrap(), Name::new("bar".to_string()).unwrap()], true)]
    #[case("bar", vec![Name::new("foo".to_string()).unwrap(), Name::new("foo".to_string()).unwrap()], false)]
    fn buildtool_matches_restriction(
        #[case] buildtool: &str,
        #[case] restrictions: Vec<Name>,
        #[case] result: bool,
    ) {
        let buildtool = BuildTool::new(buildtool).unwrap();
        assert_eq!(buildtool.matches_restriction(&restrictions), result);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn valid_name_from_string(name_str in r"[a-z\d_@+]+[a-z\d\-._@+]*") {
            let name = Name::from_str(&name_str).unwrap();
            prop_assert_eq!(name_str, format!("{}", name));
        }

        #[test]
        fn invalid_name_from_string_start(name_str in r"[\-.]+[a-z\d\-._@+]*") {
            let error = Name::from_str(&name_str).unwrap_err();
            assert!(format!("{}", error).ends_with(&name_str));
        }
    }
}
