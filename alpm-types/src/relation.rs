use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::Serialize;
use strum::IntoEnumIterator;

use crate::{Error, Name, VersionComparison, VersionRequirement};

/// Representation of [soname] data of a shared object based on the [alpm-sonamev2] specification.
///
/// Soname data may be used as [alpm-package-relation] of type _provision_ or _run-time dependency_
/// in [`PackageInfoV1`] and [`PackageInfoV2`]. The data consists of the arbitrarily
/// defined `prefix`, which denotes the use name of a specific library directory, and the `soname`,
/// which refers to the value of either the _SONAME_ or a _NEEDED_ field in the _dynamic section_ of
/// an [ELF] file.
///
/// # Examples
///
/// This example assumpes that `lib` is used as the `prefix` for the library directory `/usr/lib`
/// and the following files are contained in it:
///
/// ```bash
/// /usr/lib/libexample.so -> libexample.so.1
/// /usr/lib/libexample.so.1 -> libexample.so.1.0.0
/// /usr/lib/libexample.so.1.0.0
/// ```
///
/// The above file `/usr/lib/libexample.so.1.0.0` represents an [ELF] file, that exposes
/// `libexample.so.1` as value of the _SONAME_ field in its _dynamic section_. This data can be
/// represented as follows, using [`SonameV2`]:
///
/// ```rust
/// use alpm_types::SonameV2;
///
/// let soname_data = SonameV2 {
///     prefix: "lib".to_string(),
///     soname: "libexample.so.1".to_string(),
/// };
/// assert_eq!(soname_data.to_string(), "lib:libexample.so.1");
/// ```
///
/// [alpm-sonamev2]: https://alpm.archlinux.page/specifications/alpm-sonamev2.7.html
/// [alpm-package-relation]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html
/// [ELF]: https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
/// [soname]: https://en.wikipedia.org/wiki/Soname
/// [`PackageInfoV1`]: https://docs.rs/alpm_pkginfo/latest/alpm_pkginfo/struct.PackageInfoV1.html
/// [`PackageInfoV2`]: https://docs.rs/alpm_pkginfo/latest/alpm_pkginfo/struct.PackageInfoV2.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SonameV2 {
    /// The directory prefix of the shared object file.
    pub prefix: String,
    /// The shared object name.
    pub soname: String,
}

impl SonameV2 {
    /// Creates a new [`SonameV2`].
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_types::SonameV2;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// SonameV2::new("lib".to_string(), "libexample.so.1".into());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(prefix: String, soname: String) -> Self {
        Self { prefix, soname }
    }
}

impl FromStr for SonameV2 {
    type Err = Error;

    /// Parses a [`SonameV2`] from a string slice.
    ///
    /// The string slice must be in the format `<prefix>:<soname>`.
    ///
    /// # Errors
    ///
    /// Returns an error if a [`SonameV2`] can not be parsed from input.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::SonameV2;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// assert_eq!(
    ///     SonameV2::from_str("lib:libexample.so.1")?,
    ///     SonameV2::new("lib".to_string(), "libexample.so.1".into()),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, ':');
        let prefix = parts.next().ok_or(Error::MissingComponent {
            component: "prefix",
        })?;
        let soname = parts
            .next()
            .ok_or(Error::MissingComponent {
                component: "soname",
            })
            .map(String::from)?;
        Ok(Self::new(prefix.to_string(), soname))
    }
}

impl Display for SonameV2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.prefix, self.soname)
    }
}

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
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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
                    name: Name::new(name)?,
                    version_requirement: Some(VersionRequirement {
                        comparison,
                        version: version.parse()?,
                    }),
                });
            }
        }

        Ok(Self {
            name: Name::new(s)?,
            version_requirement: None,
        })
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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
}

impl FromStr for OptionalDependency {
    type Err = Error;

    /// Create an OptionalDependency from a string slice
    fn from_str(s: &str) -> Result<OptionalDependency, Self::Err> {
        if let Some((name, description)) = s.split_once(":") {
            let description = description.trim_start();
            let relation = PackageRelation::from_str(name)?;
            Ok(Self::new(
                relation,
                (!description.is_empty()).then_some(description.to_string()),
            ))
        } else {
            Ok(Self::new(PackageRelation::new(Name::new(s)?, None), None))
        }
    }
}

impl Display for OptionalDependency {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match self.description {
            Some(ref description) => write!(fmt, "{}: {}", self.name(), description),
            None => write!(fmt, "{}", self.name()),
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
            let opt_depend = OptionalDependency::from_str(&s).unwrap();
            let formatted = format!("{}", opt_depend);
            prop_assert_eq!(s.trim_end(), formatted.trim_end(), "Formatted output doesn't match input");
        }
    }

    #[rstest]
    #[case(
        "example: this is an example dependency",
        Ok(OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("example").unwrap(),
                version_requirement: None,
            },
            description: Some("this is an example dependency".to_string()),
        }),
    )]
    #[case(
        "dep_name",
        Ok(OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("dep_name").unwrap(),
                version_requirement: None,
            },
            description: None,
        }),
    )]
    #[case(
        "dep_name: ",
        Ok(OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("dep_name").unwrap(),
                version_requirement: None,
            },
            description: None,
        }),
    )]
    #[case(
        "dep_name_with_special_chars-123: description with !@#$%^&*",
        Ok(OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("dep_name_with_special_chars-123").unwrap(),
                version_requirement: None,
            },
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
    // versioned optional dependencies
    #[case(
        "elfutils=0.192: for translations",
        Ok(OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("elfutils").unwrap(),
                version_requirement: Some(VersionRequirement {
                    comparison: VersionComparison::Equal,
                    version: "0.192".parse().unwrap(),
                }),
            },
            description: Some("for translations".to_string()),
        }),
    )]
    #[case(
        "python>=3: For Python bindings",
        Ok(OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("python").unwrap(),
                version_requirement: Some(VersionRequirement {
                    comparison: VersionComparison::GreaterOrEqual,
                    version: "3".parse().unwrap(),
                }),
            },
            description: Some("For Python bindings".to_string()),
        }),
    )]
    #[case(
        "java-environment>=17: required by extension-wiki-publisher and extension-nlpsolver",
        Ok(OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("java-environment").unwrap(),
                version_requirement: Some(VersionRequirement {
                    comparison: VersionComparison::GreaterOrEqual,
                    version: "17".parse().unwrap(),
                }),
            },
            description: Some("required by extension-wiki-publisher and extension-nlpsolver".to_string()),
        }),
    )]
    fn opt_depend_from_string(
        #[case] input: &str,
        #[case] expected_result: Result<OptionalDependency, Error>,
    ) {
        let opt_depend_result = OptionalDependency::from_str(input);
        assert_eq!(expected_result, opt_depend_result);
    }

    #[rstest]
    #[case(
        "lib:libexample.so.1",
        SonameV2 {
            prefix: "lib".to_string(),
            soname: "libexample.so.1".parse().unwrap(),
        },
    )]
    #[case(
        "usr:libexample.so.1",
        SonameV2 {
            prefix: "usr".to_string(),
            soname: "libexample.so.1".parse().unwrap(),
        },
    )]
    #[case(
        "lib:libexample.so.1.2.3",
        SonameV2 {
            prefix: "lib".to_string(),
            soname: "libexample.so.1.2.3".parse().unwrap(),
        },
    )]
    fn sonamev2_from_string(
        #[case] input: &str,
        #[case] expected_result: SonameV2,
    ) -> testresult::TestResult<()> {
        let soname = SonameV2::from_str(input)?;
        assert_eq!(expected_result, soname);
        assert_eq!(input, soname.to_string());
        Ok(())
    }
}
