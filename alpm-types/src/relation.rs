use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use strum::IntoEnumIterator;

use crate::{Error, Name, VersionComparison, VersionRequirement};

/// A package relation
///
/// Describes a relation to a component.
/// Package relations may either consist of only a [`Name`] *or* of a [`Name`] and a
/// [`VersionRequirement`].
///
/// ## Note
///
/// A [`PackageRelation`] covers all **alpm-package-relations** *except* optional
/// dependencies, as those behave differently.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageRelation {
    pub name: Name,
    pub version_requirement: Option<VersionRequirement>,
}

impl PackageRelation {
    /// Creates a new [`PackageRelation`]
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_types::{PackageRelation, VersionComparison, VersionRequirement};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// PackageRelation::new(
    ///     "example".parse()?,
    ///     Some(VersionRequirement {
    ///         comparison: VersionComparison::Less,
    ///         version: "1.0.0".parse()?,
    ///     }),
    /// );
    ///
    /// PackageRelation::new("example".parse()?, None);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(name: Name, version_requirement: Option<VersionRequirement>) -> Self {
        Self {
            name,
            version_requirement,
        }
    }
}

impl Display for PackageRelation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(version_requirement) = self.version_requirement.as_ref() {
            write!(f, "{}{}", self.name, version_requirement)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

impl FromStr for PackageRelation {
    type Err = Error;
    /// Parses a [`PackageRelation`] from a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if a [`PackageRelation`] can not be parsed from input.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::{PackageRelation, VersionComparison, VersionRequirement};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// assert_eq!(
    ///     PackageRelation::from_str("example<1.0.0")?,
    ///     PackageRelation::new(
    ///         "example".parse()?,
    ///         Some(VersionRequirement {
    ///             comparison: VersionComparison::Less,
    ///             version: "1.0.0".parse()?
    ///         })
    ///     ),
    /// );
    ///
    /// assert_eq!(
    ///     PackageRelation::from_str("example<=1.0.0")?,
    ///     PackageRelation::new(
    ///         "example".parse()?,
    ///         Some(VersionRequirement {
    ///             comparison: VersionComparison::LessOrEqual,
    ///             version: "1.0.0".parse()?
    ///         })
    ///     ),
    /// );
    ///
    /// assert_eq!(
    ///     PackageRelation::from_str("example=1.0.0")?,
    ///     PackageRelation::new(
    ///         "example".parse()?,
    ///         Some(VersionRequirement {
    ///             comparison: VersionComparison::Equal,
    ///             version: "1.0.0".parse()?
    ///         })
    ///     ),
    /// );
    ///
    /// assert_eq!(
    ///     PackageRelation::from_str("example>1.0.0")?,
    ///     PackageRelation::new(
    ///         "example".parse()?,
    ///         Some(VersionRequirement {
    ///             comparison: VersionComparison::Greater,
    ///             version: "1.0.0".parse()?
    ///         })
    ///     ),
    /// );
    ///
    /// assert_eq!(
    ///     PackageRelation::from_str("example>=1.0.0")?,
    ///     PackageRelation::new(
    ///         "example".parse()?,
    ///         Some(VersionRequirement {
    ///             comparison: VersionComparison::GreaterOrEqual,
    ///             version: "1.0.0".parse()?
    ///         })
    ///     ),
    /// );
    ///
    /// assert_eq!(
    ///     PackageRelation::from_str("example")?,
    ///     PackageRelation::new("example".parse()?, None),
    /// );
    ///
    /// assert!(PackageRelation::from_str("example<").is_err());
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // NOTE: The string splitting relies on the specific ordering of the VersionComparison
        // variants (which orders two-letter comparators over one-letter ones)!
        for comparison in VersionComparison::iter() {
            if let Some((name, version)) = s.split_once(comparison.as_ref()) {
                return Ok(Self {
                    name: Name::new(name.to_string())?,
                    version_requirement: Some(VersionRequirement {
                        comparison,
                        version: version.parse()?,
                    }),
                });
            }
        }

        Ok(Self {
            name: Name::new(s.to_string())?,
            version_requirement: None,
        })
    }
}

/// An optional dependency for a package.
///
/// This type is used for representing dependencies that are not essential for base functionality
/// of a package, but may be necessary to make use of certain features of a package.
///
/// An [`OptDepend`] consists of a name and an optional description separated by a colon (`:`).
///
/// - The name component must be a valid [`Name`].
/// - If a description is provided it must be at least one character long.
///
/// ## Note
///
/// It's currently not possible to specify a version in an optional dependency due to
/// the limitations of the current file format.
///
/// ## Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Name, OptDepend};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // Create OptDepend from &str
/// let opt_depend = OptDepend::from_str("example: this is an example dependency")?;
///
/// // Get the name
/// assert_eq!("example", opt_depend.name().as_ref());
///
/// // Get the description
/// assert_eq!(
///     Some("this is an example dependency"),
///     opt_depend.description().as_deref()
/// );
///
/// // Format as String
/// assert_eq!(
///     "example: this is an example dependency",
///     format!("{opt_depend}")
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OptDepend {
    name: Name,
    description: Option<String>,
}

impl OptDepend {
    /// Create a new OptDepend in a Result
    pub fn new(name: Name, description: Option<String>) -> OptDepend {
        OptDepend { name, description }
    }

    /// Return the name of the optional dependency
    pub fn name(&self) -> &Name {
        &self.name
    }

    /// Return the description for the optional dependency, if it exists
    pub fn description(&self) -> &Option<String> {
        &self.description
    }
}

impl FromStr for OptDepend {
    type Err = Error;

    /// Create an OptDepend from a string slice
    fn from_str(s: &str) -> Result<OptDepend, Self::Err> {
        if let Some((name, description)) = s.split_once(":") {
            let description = description.trim_start();
            Ok(Self::new(
                Name::new(name.to_string())?,
                (!description.is_empty()).then_some(description.to_string()),
            ))
        } else {
            Ok(Self::new(Name::new(s.to_string())?, None))
        }
    }
}

impl Display for OptDepend {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match self.description {
            Some(ref description) => write!(fmt, "{}: {}", self.name(), description),
            None => write!(fmt, "{}", self.name()),
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::{prop_assert_eq, proptest, test_runner::Config as ProptestConfig};
    use rstest::rstest;

    use super::*;

    const COMPARATOR_REGEX: &str = r"(<|<=|=|>=|>)";
    /// NOTE: [`Epoch`][alpm_types::Epoch] is implicitly constrained by [`std::usize::MAX`].
    /// However, it's unrealistic to ever reach that many forced downgrades for a package, hence
    /// we don't test that fully
    const EPOCH_REGEX: &str = r"[1-9]{1}[0-9]{0,10}";
    const NAME_REGEX: &str = r"[a-z0-9_@+]+[a-z0-9\-._@+]*";
    const PKGREL_REGEX: &str = r"[1-9]+[0-9]*(|[.]{1}[1-9]{1}[0-9]*)";
    const PKGVER_REGEX: &str = r"([[:alnum:]][[:alnum:]_+.]*)";
    const DESCRIPTION_REGEX: &str = r"[[:alnum:]][[:alnum:] _+.,-]*";

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]


        #[test]
        fn valid_package_relation_from_str(s in format!("{NAME_REGEX}(|{COMPARATOR_REGEX}(|{EPOCH_REGEX}:){PKGVER_REGEX}(|-{PKGREL_REGEX}))").as_str()) {
            println!("s: {s}");
            let name = PackageRelation::from_str(&s).unwrap();
            prop_assert_eq!(s, format!("{}", name));
        }
    }

    proptest! {
        #[test]
        fn opt_depend_from_str(s in format!("{NAME_REGEX}(: {DESCRIPTION_REGEX})?").as_str()) {
            println!("s: {s}");
            let opt_depend = OptDepend::from_str(&s).unwrap();
            let formatted = format!("{}", opt_depend);
            prop_assert_eq!(s.trim_end(), formatted.trim_end(), "Formatted output doesn't match input");
        }
    }

    #[rstest]
    #[case(
        "example: this is an example dependency",
        Ok(OptDepend {
            name: Name::new("example".to_string()).unwrap(),
            description: Some("this is an example dependency".to_string()),
        }),
    )]
    #[case(
        "dep_name",
        Ok(OptDepend {
            name: Name::new("dep_name".to_string()).unwrap(),
            description: None,
        }),
    )]
    #[case(
        "dep_name: ",
        Ok(OptDepend {
            name: Name::new("dep_name".to_string()).unwrap(),
            description: None,
        }),
    )]
    #[case(
        "dep_name_with_special_chars-123: description with !@#$%^&*",
        Ok(OptDepend {
            name: Name::new("dep_name_with_special_chars-123".to_string()).unwrap(),
            description: Some("description with !@#$%^&*".to_string()),
        }),
    )]
    #[case(
        "#invalid-name: this is an example dependency",
        Err(Error::RegexDoesNotMatch {
            value: "#invalid-name".to_string(),
            regex_type: "pkgname".to_string(),
            regex: crate::name::NAME_REGEX.to_string(),
        }),
    )]
    #[case(
        ": no_name_colon",
        Err(Error::RegexDoesNotMatch {
            value: "".to_string(),
            regex_type: "pkgname".to_string(),
            regex: crate::name::NAME_REGEX.to_string(),
        }),
    )]
    fn opt_depend_from_string(
        #[case] input: &str,
        #[case] expected_result: Result<OptDepend, Error>,
    ) {
        let opt_depend_result = OptDepend::from_str(input);
        assert_eq!(expected_result, opt_depend_result);
    }
}
