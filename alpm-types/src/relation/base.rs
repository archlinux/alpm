//! Basic relation types used in metadata files.

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use winnow::{
    ModalResult,
    Parser,
    ascii::space1,
    combinator::{alt, cut_err, eof, opt, separated_pair, seq, terminated},
    error::{StrContext, StrContextValue},
    token::{rest, take_till, take_until},
};

use crate::{Error, Name, VersionRequirement};

/// A package relation
///
/// Describes a relation to a component.
/// Package relations may either consist of only a [`Name`] *or* of a [`Name`] and a
/// [`VersionRequirement`].
///
/// ## Note
///
/// A [`PackageRelation`] covers all [alpm-package-relations] *except* optional
/// dependencies, as those behave differently.
///
/// [alpm-package-relations]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PackageRelation {
    /// The name of the package
    pub name: Name,
    /// The version requirement for the package
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

    /// Parses a [`PackageRelation`] from a string slice.
    ///
    /// Consumes all of its input.
    ///
    /// # Examples
    ///
    /// See [`Self::from_str`] for code examples.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is not a valid _package-relation_.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        seq!(Self {
            name: take_till(1.., ('<', '>', '=')).and_then(Name::parser).context(StrContext::Label("package name")),
            version_requirement: opt(VersionRequirement::parser),
            _: eof.context(StrContext::Expected(StrContextValue::Description("end of relation version requirement"))),
        })
        .parse_next(input)
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
    /// Delegates to [`PackageRelation::parser`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`PackageRelation::parser`] fails.
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
        Ok(Self::parser.parse(s)?)
    }
}

/// An optional dependency for a package.
///
/// This type is used for representing dependencies that are not essential for base functionality
/// of a package, but may be necessary to make use of certain features of a package.
///
/// An [`OptionalDependency`] consists of a package relation and an optional description separated
/// by a colon (`:`).
///
/// - The package relation component must be a valid [`PackageRelation`].
/// - If a description is provided it must be at least one character long.
///
/// Refer to [alpm-package-relation] of type [optional dependency] for details on the format.
/// ## Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Name, OptionalDependency};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // Create OptionalDependency from &str
/// let opt_depend = OptionalDependency::from_str("example: this is an example dependency")?;
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
///
/// [alpm-package-relation]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html
/// [optional dependency]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#optional-dependency
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OptionalDependency {
    package_relation: PackageRelation,
    description: Option<String>,
}

impl OptionalDependency {
    /// Create a new OptionalDependency in a Result
    pub fn new(
        package_relation: PackageRelation,
        description: Option<String>,
    ) -> OptionalDependency {
        OptionalDependency {
            package_relation,
            description,
        }
    }

    /// Return the name of the optional dependency
    pub fn name(&self) -> &Name {
        &self.package_relation.name
    }

    /// Return the version requirement of the optional dependency
    pub fn version_requirement(&self) -> &Option<VersionRequirement> {
        &self.package_relation.version_requirement
    }

    /// Return the description for the optional dependency, if it exists
    pub fn description(&self) -> &Option<String> {
        &self.description
    }

    /// Returns a reference to the tracked [`PackageRelation`].
    pub fn package_relation(&self) -> &PackageRelation {
        &self.package_relation
    }

    /// Recognizes an [`OptionalDependency`] in a string slice.
    ///
    /// Consumes all of its input.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is not a valid _alpm-package-relation_ of type _optional
    /// dependency_.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        let description_parser = terminated(
            // Descriptions may consist of any character except '\n' and '\r'.
            // Descriptions are a also at the end of a `OptionalDependency`.
            // We enforce forbidding `\n` and `\r` by only taking until either of them
            // is hit and checking for `eof` afterwards.
            // This will **always** succeed unless `\n` and `\r` are hit, in which case an
            // error is thrown.
            take_till(0.., ('\n', '\r')),
            eof,
        )
        .context(StrContext::Label("optional dependency description"))
        .context(StrContext::Expected(StrContextValue::Description(
            r"no carriage returns or newlines",
        )))
        .map(|d: &str| match d.trim_ascii() {
            "" => None,
            t => Some(t.to_string()),
        });

        let (package_relation, description) = alt((
            // look for a ":" followed by at least one whitespace, then dispatch either side to the
            // relevant parser without allowing backtracking.
            separated_pair(
                take_until(1.., ":").and_then(cut_err(PackageRelation::parser)),
                (":", space1),
                rest.and_then(cut_err(description_parser)),
            ),
            // if we can't find ": ", then assume it's all PackageRelation
            // and assert we've reached the end of input
            (rest.and_then(PackageRelation::parser), eof.value(None)),
        ))
        .parse_next(input)?;

        Ok(Self {
            package_relation,
            description,
        })
    }
}

impl FromStr for OptionalDependency {
    type Err = Error;

    /// Creates a new [`OptionalDependency`] from a string slice.
    ///
    /// Delegates to [`OptionalDependency::parser`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`OptionalDependency::parser`] fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

impl Display for OptionalDependency {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match self.description {
            Some(ref description) => write!(fmt, "{}: {}", self.package_relation, description),
            None => write!(fmt, "{}", self.package_relation),
        }
    }
}

/// Group of a package
///
/// Represents an arbitrary collection of packages that share a common
/// characteristic or functionality.
///
/// While group names can be any valid UTF-8 string, it is recommended to follow
/// the format of [`Name`] (`[a-z\d\-._@+]` but must not start with `[-.]`)
/// to ensure consistency and ease of use.
///
/// This is a type alias for [`String`].
///
/// ## Examples
/// ```
/// use alpm_types::Group;
///
/// // Create a Group
/// let group: Group = "package-group".to_string();
/// ```
pub type Group = String;

#[cfg(test)]
mod tests {
    use proptest::{prop_assert_eq, proptest, test_runner::Config as ProptestConfig};
    use rstest::rstest;

    use super::*;
    use crate::VersionComparison;

    const COMPARATOR_REGEX: &str = r"(<|<=|=|>=|>)";
    /// NOTE: [`Epoch`][alpm_types::Epoch] is implicitly constrained by [`std::usize::MAX`].
    /// However, it's unrealistic to ever reach that many forced downgrades for a package, hence
    /// we don't test that fully
    const EPOCH_REGEX: &str = r"[1-9]{1}[0-9]{0,10}";
    const NAME_REGEX: &str = r"[a-z0-9_@+]+[a-z0-9\-._@+]*";
    const PKGREL_REGEX: &str = r"[1-9][0-9]{0,8}(|[.][1-9][0-9]{0,8})";
    const PKGVER_REGEX: &str = r"([[:alnum:]][[:alnum:]_+.]*)";
    const DESCRIPTION_REGEX: &str = "[^\n\r]*";

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
        fn opt_depend_from_str(
            name in NAME_REGEX,
            desc in DESCRIPTION_REGEX,
            use_desc in proptest::bool::ANY
        ) {
            let desc_trimmed = desc.trim_ascii();
            let desc_is_blank = desc_trimmed.is_empty();

            let (raw_in, formatted_expected) = if use_desc {
                // Raw input and expected formatted output.
                // These are different because `desc` will be trimmed by the parser;
                // if it is *only* ascii whitespace then it will be skipped altogether.
                (
                    format!("{name}: {desc}"),
                    if !desc_is_blank {
                        format!("{name}: {desc_trimmed}")
                    } else {
                        name.clone()
                    }
                )
            } else {
                (name.clone(), name.clone())
            };

            println!("input string: {raw_in}");
            let opt_depend = OptionalDependency::from_str(&raw_in).unwrap();
            let formatted_actual = format!("{opt_depend}");
            prop_assert_eq!(
                formatted_expected,
                formatted_actual,
                "Formatted output doesn't match input"
            );
        }
    }

    #[rstest]
    #[case(
        "python>=3",
        Ok(PackageRelation {
            name: Name::new("python").unwrap(),
            version_requirement: Some(VersionRequirement {
                comparison: VersionComparison::GreaterOrEqual,
                version: "3".parse().unwrap(),
            }),
        }),
    )]
    #[case(
        "java-environment>=17",
        Ok(PackageRelation {
            name: Name::new("java-environment").unwrap(),
            version_requirement: Some(VersionRequirement {
                comparison: VersionComparison::GreaterOrEqual,
                version: "17".parse().unwrap(),
            }),
        }),
    )]
    fn valid_package_relation(
        #[case] input: &str,
        #[case] expected: Result<PackageRelation, Error>,
    ) {
        assert_eq!(PackageRelation::from_str(input), expected);
    }

    #[rstest]
    #[case(
        "example: this is an example dependency",
        OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("example").unwrap(),
                version_requirement: None,
            },
            description: Some("this is an example dependency".to_string()),
        },
    )]
    #[case(
        "example-two:     a description with lots of whitespace padding     ",
        OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("example-two").unwrap(),
                version_requirement: None,
            },
            description: Some("a description with lots of whitespace padding".to_string())
        },
    )]
    #[case(
        "dep_name",
        OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("dep_name").unwrap(),
                version_requirement: None,
            },
            description: None,
        },
    )]
    #[case(
        "dep_name: ",
        OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("dep_name").unwrap(),
                version_requirement: None,
            },
            description: None,
        },
    )]
    #[case(
        "dep_name_with_special_chars-123: description with !@#$%^&*",
        OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("dep_name_with_special_chars-123").unwrap(),
                version_requirement: None,
            },
            description: Some("description with !@#$%^&*".to_string()),
        },
    )]
    // versioned optional dependencies
    #[case(
        "elfutils=0.192: for translations",
        OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("elfutils").unwrap(),
                version_requirement: Some(VersionRequirement {
                    comparison: VersionComparison::Equal,
                    version: "0.192".parse().unwrap(),
                }),
            },
            description: Some("for translations".to_string()),
        },
    )]
    #[case(
        "python>=3: For Python bindings",
        OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("python").unwrap(),
                version_requirement: Some(VersionRequirement {
                    comparison: VersionComparison::GreaterOrEqual,
                    version: "3".parse().unwrap(),
                }),
            },
            description: Some("For Python bindings".to_string()),
        },
    )]
    #[case(
        "java-environment>=17: required by extension-wiki-publisher and extension-nlpsolver",
        OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("java-environment").unwrap(),
                version_requirement: Some(VersionRequirement {
                    comparison: VersionComparison::GreaterOrEqual,
                    version: "17".parse().unwrap(),
                }),
            },
            description: Some("required by extension-wiki-publisher and extension-nlpsolver".to_string()),
        },
    )]
    fn opt_depend_from_string(#[case] input: &str, #[case] expected: OptionalDependency) {
        let opt_depend_result = OptionalDependency::from_str(input);
        let Ok(optional_dependency) = opt_depend_result else {
            panic!(
                "Encountered unexpected error when parsing optional dependency: {opt_depend_result:?}"
            )
        };

        assert_eq!(
            expected, optional_dependency,
            "Optional dependency has not been correctly parsed."
        );
    }

    #[rstest]
    #[case(
        "example: this is an example dependency",
        "example: this is an example dependency"
    )]
    #[case(
        "example-two:     a description with lots of whitespace padding     ",
        "example-two: a description with lots of whitespace padding"
    )]
    #[case(
        "tabs:    a description with a tab directly after the colon",
        "tabs: a description with a tab directly after the colon"
    )]
    #[case("dep_name", "dep_name")]
    #[case("dep_name: ", "dep_name")]
    #[case(
        "dep_name_with_special_chars-123: description with !@#$%^&*",
        "dep_name_with_special_chars-123: description with !@#$%^&*"
    )]
    // versioned optional dependencies
    #[case("elfutils=0.192: for translations", "elfutils=0.192: for translations")]
    #[case("python>=3: For Python bindings", "python>=3: For Python bindings")]
    #[case(
        "java-environment>=17: required by extension-wiki-publisher and extension-nlpsolver",
        "java-environment>=17: required by extension-wiki-publisher and extension-nlpsolver"
    )]
    fn opt_depend_to_string(#[case] input: &str, #[case] expected: &str) {
        let opt_depend_result = OptionalDependency::from_str(input);
        let Ok(optional_dependency) = opt_depend_result else {
            panic!(
                "Encountered unexpected error when parsing optional dependency: {opt_depend_result:?}"
            )
        };
        assert_eq!(
            expected,
            optional_dependency.to_string(),
            "OptionalDependency to_string is erroneous."
        );
    }

    #[rstest]
    #[case(
        "#invalid-name: this is an example dependency",
        "invalid first character of package name"
    )]
    #[case(": no_name_colon", "invalid first character of package name")]
    #[case(
        "name:description with no leading whitespace",
        "invalid character in package name"
    )]
    #[case(
        "dep-name>=10: \n\ndescription with\rnewlines",
        "expected no carriage returns or newlines"
    )]
    fn opt_depend_invalid_string_parse_error(#[case] input: &str, #[case] err_snippet: &str) {
        let Err(Error::ParseError(err_msg)) = OptionalDependency::from_str(input) else {
            panic!("'{input}' did not fail to parse as expected")
        };
        assert!(
            err_msg.contains(err_snippet),
            "Error:\n=====\n{err_msg}\n=====\nshould contain snippet:\n\n{err_snippet}"
        );
    }
}
