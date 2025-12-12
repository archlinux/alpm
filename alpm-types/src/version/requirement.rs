//! Version requirement declarations and comparisons based on them.

use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
    str::FromStr,
};

use alpm_parsers::iter_str_context;
use serde::{Deserialize, Serialize};
use strum::VariantNames;
use winnow::{
    ModalResult,
    Parser,
    combinator::{alt, eof, fail, seq},
    error::{StrContext, StrContextValue},
    token::take_while,
};

use crate::{Error, Version};

/// A version requirement, e.g. for a dependency package.
///
/// It consists of a target version and a comparison function. A version requirement of `>=1.5` has
/// a target version of `1.5` and a comparison function of [`VersionComparison::GreaterOrEqual`].
/// See [alpm-comparison] for details on the format.
///
/// ## Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Version, VersionComparison, VersionRequirement};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// let requirement = VersionRequirement::from_str(">=1.5")?;
///
/// assert_eq!(requirement.comparison, VersionComparison::GreaterOrEqual);
/// assert_eq!(requirement.version, Version::from_str("1.5")?);
/// # Ok(())
/// # }
/// ```
///
/// [alpm-comparison]: https://alpm.archlinux.page/specifications/alpm-comparison.7.html
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VersionRequirement {
    /// Version comparison function
    pub comparison: VersionComparison,
    /// Target version
    pub version: Version,
}

impl VersionRequirement {
    /// Create a new `VersionRequirement`
    pub fn new(comparison: VersionComparison, version: Version) -> Self {
        VersionRequirement {
            comparison,
            version,
        }
    }

    /// Returns `true` if the requirement is satisfied by the given package version.
    ///
    /// ## Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::{Version, VersionRequirement};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let requirement = VersionRequirement::from_str(">=1.5-3")?;
    ///
    /// assert!(!requirement.is_satisfied_by(&Version::from_str("1.5")?));
    /// assert!(requirement.is_satisfied_by(&Version::from_str("1.5-3")?));
    /// assert!(requirement.is_satisfied_by(&Version::from_str("1.6")?));
    /// assert!(requirement.is_satisfied_by(&Version::from_str("2:1.0")?));
    /// assert!(!requirement.is_satisfied_by(&Version::from_str("1.0")?));
    ///
    /// // If pkgrel is not specified in the requirement, it is ignored in the comparison.
    /// let requirement = VersionRequirement::from_str("=1.5")?;
    /// assert!(requirement.is_satisfied_by(&Version::from_str("1.5-3")?));
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_satisfied_by(&self, ver: &Version) -> bool {
        // If the requirement does not specify a pkgrel, we ignore it in the comparison.
        // so that `foo=1` can be satisfied by `foo=1-1`.
        let other_version = if self.version.pkgrel.is_none() {
            &Version {
                pkgrel: None,
                ..ver.clone()
            }
        } else {
            ver
        };
        self.comparison
            .is_compatible_with(other_version.cmp(&self.version))
    }

    /// Recognizes a [`VersionRequirement`] in a string slice.
    ///
    /// Consumes all of its input.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is not a valid _alpm-comparison_.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        seq!(Self {
            comparison: take_while(1.., ('<', '>', '='))
                // add context here because otherwise take_while can fail and provide no information
                .context(StrContext::Expected(StrContextValue::Description(
                    "version comparison operator"
                )))
                .and_then(VersionComparison::parser),
            version: Version::parser,
        })
        .parse_next(input)
    }

    /// Checks whether another [`VersionRequirement`] forms an intersection with this one.
    ///
    /// The intersection operation `∩` on versions simply checks if there is _any_ possible set of
    /// versions that can exist while upholding the constraints (e.g. `>`/`<=`) on both versions.
    ///
    /// # Examples
    ///
    /// - The expression `<3 ∩ <1` forms the intersection of all versions `<1`
    /// - The expression `<2 ∩ >1` forms the intersection `X` of all versions `1<X<2`
    /// - The expression `=2 ∩ <3` forms the intersection of the exact version `2`
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::VersionRequirement;
    ///
    /// # fn main() -> testresult::TestResult {
    /// let requirement: VersionRequirement = "<1".parse()?;
    /// assert!(requirement.is_intersection(&"<0.1".parse()?));
    ///
    /// let requirement: VersionRequirement = "<2".parse()?;
    /// assert!(requirement.is_intersection(&">1".parse()?));
    ///
    /// let requirement: VersionRequirement = "=2".parse()?;
    /// assert!(!requirement.is_intersection(&"<3".parse()?));
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_intersection(&self, other: &VersionRequirement) -> bool {
        // This documentation uses the `∩` set intersection operator to better visualize examples.
        //
        // In the following, we need to consider the ordering relationship between the actual
        // versions of the two `VersionRequirement`s.
        // If we have `self = ">1.0.1"` and `other = "<2"`, this handles the part of
        // `"1.0.1".cmp("2")`.
        let version_comparison = self.version.cmp(&other.version);

        match self.comparison {
            // Consider the case where we have a `Less`, e.g. `<1`.
            VersionComparison::Less => {
                match version_comparison {
                    // The other version is greater, so its comparison must be "Less" or
                    // "LessOrEqual" to form an intersection.
                    //
                    // Example:
                    // - `<1.0.1 ∩ <2` forms the intersection of all versions `<1.0.1`
                    Ordering::Less => matches!(
                        other.comparison,
                        VersionComparison::Less | VersionComparison::LessOrEqual
                    ),
                    // Both versions are matching. The comparison for other must be "Less" or
                    // "LessOrEqual"
                    //
                    // Example:
                    // - `<=2 ∩ <2` forms the intersection of all versions `<2`
                    // - `<2 ∩ <=2` forms the intersection of all versions `<2`
                    Ordering::Equal => matches!(
                        other.comparison,
                        VersionComparison::Less | VersionComparison::LessOrEqual
                    ),

                    // The other version is smaller.
                    // Since `self` enforces the `Less` constraint, there will always be at least
                    // **some** intersection.
                    //
                    // Example: Even if `other` also has a "Less" constraint, the expression
                    // `<3 ∩ <1` forms the intersection of all versions `<1`
                    Ordering::Greater => true,
                }
            }
            // Consider the case where we have a `LessOrEqual`, e.g. `<=1`.
            VersionComparison::LessOrEqual => {
                match version_comparison {
                    // The other version is greater, so its comparison must be "Less" or
                    // "LessOrEqual" to form an intersection.
                    //
                    // Example:
                    // - `>=1.0.1 ∩ <2` forms the intersection of all versions `1.0.1<=X<2`
                    // - `>= 1.0.1 <= 1.2` forms the intersection of all versions `1.0.1<=X<=1.2`
                    Ordering::Less => matches!(
                        other.comparison,
                        VersionComparison::Less | VersionComparison::LessOrEqual
                    ),
                    // Both versions are matching, the comparison for other must be either "Less"
                    // or one of "Equal", "LessOrEqual", or "GreaterOrEqual".
                    // Any `other` "*Equal" constraint will directly match the `self`
                    // "Less**Equal**" constraint.
                    //
                    // Examples:
                    // - `<=1 ∩ >=1` forms the intersection of the version `1`
                    // - `<=1 ∩ <1` forms the intersection of all version `<1`
                    // - `<=1 ∩ <=1` forms the intersection of all version `<=1`
                    Ordering::Equal => matches!(
                        other.comparison,
                        VersionComparison::Less
                            | VersionComparison::LessOrEqual
                            | VersionComparison::Equal
                            | VersionComparison::GreaterOrEqual
                    ),
                    // The other version is smaller.
                    // Since `self` enforces the `Less` constraint, there will always be at least
                    // **some** intersection.
                    //
                    // Example: Even if `other` also has a "Less" constraint, the expression
                    // `=<3 ∩ <1` forms the intersection of all versions `<1`
                    Ordering::Greater => true,
                }
            }
            // Consider the case where we have a `Equal`, e.g. `=1`.
            VersionComparison::Equal => match version_comparison {
                // Both versions are matching, the comparison for `other` must be
                // "LessOrEqual", "Equal", or "GreaterOrEqual" to match the "Equal" constraint on
                // `self`
                //
                // Examples:
                // - `=1 ∩ >=1` forms the intersection of the version `1`
                // - `=1 ∩ <=1` forms the intersection of the version `1`
                // - `=2 ∩ =2` forms the intersection of the version `2`
                Ordering::Equal => matches!(
                    other.comparison,
                    VersionComparison::LessOrEqual
                        | VersionComparison::Equal
                        | VersionComparison::GreaterOrEqual
                ),
                // The other version must be greater or smaller, so it can be inherently not be
                // equal.
                Ordering::Less | Ordering::Greater => false,
            },
            // Consider the case where we have a `GreaterOrEqual`, e.g. `>=1`.
            VersionComparison::GreaterOrEqual => match version_comparison {
                // The other version is greater.
                // Since `self` enforces a `Greater` constraint, so there will always be at least
                // **some** intersection.
                //
                // Example: Even if `other` also has a "Less" constraint, the expression
                // `>=1 ∩ <3` forms the intersection of all versions `1<=X<3`
                Ordering::Less => true,
                // Both versions are matching, the comparison for other must be either "Greater"
                // or one of "Equal", "LessOrEqual", or "GreaterOrEqual".
                // Any `other` "*Equal" constraint will directly match `self`'s
                // "LesserOr**Equal**" constraint.
                //
                // Examples:
                // - `>=1 ∩ <=1` forms the intersection of the version `1`
                // - `>=1 ∩ >1` forms the intersection of all version `>1`
                // - `>=1 ∩ >=1` forms the intersection of all version `>=1`
                Ordering::Equal => matches!(
                    other.comparison,
                    VersionComparison::LessOrEqual
                        | VersionComparison::Equal
                        | VersionComparison::GreaterOrEqual
                        | VersionComparison::Greater
                ),
                // The other version is smaller, so its comparison must be at least "Greater" or
                // "GreaterOrEqual" to form an intersection.
                //
                // Example:
                // - `>=2 ∩ >1.1` forms the intersection of all versions `>=2`
                Ordering::Greater => matches!(
                    other.comparison,
                    VersionComparison::GreaterOrEqual | VersionComparison::Greater
                ),
            },
            // Consider the case where we have a `Greater`, e.g. `>1`.
            VersionComparison::Greater => {
                match version_comparison {
                    // The other version is greater.
                    // Since `self` enforces a `Greater` constraint, so there will always be at
                    // least **some** intersection.
                    //
                    // Example: Even if `other` also has a "Less" constraint, the expression
                    // `>1 ∩ <3` forms the intersection of all versions `1<X<3`
                    Ordering::Less => true,
                    // Both versions are matching. The comparison for other must be "Greater" or
                    // "GreaterOrEqual"
                    //
                    // Example:
                    // - `>2 ∩ >2` forms the intersection of all versions `>2`
                    // - `>2 ∩ >=2` forms the intersection of all versions `>2`
                    Ordering::Equal => matches!(
                        other.comparison,
                        VersionComparison::GreaterOrEqual | VersionComparison::Greater
                    ),
                    // The other version is smaller, so its comparison must be at least "Greater" or
                    // "GreaterOrEqual" to form an intersection.
                    //
                    // Example:
                    // - `>2 ∩ >=1.1` forms the intersection of all versions `>=2`
                    Ordering::Greater => matches!(
                        other.comparison,
                        VersionComparison::GreaterOrEqual | VersionComparison::Greater
                    ),
                }
            }
        }
    }
}

impl Display for VersionRequirement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.comparison, self.version)
    }
}

impl FromStr for VersionRequirement {
    type Err = Error;

    /// Creates a new [`VersionRequirement`] from a string slice.
    ///
    /// Delegates to [`VersionRequirement::parser`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`VersionRequirement::parser`] fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

/// Specifies the comparison function for a [`VersionRequirement`].
///
/// The package version can be required to be:
/// - less than (`<`)
/// - less than or equal to (`<=`)
/// - equal to (`=`)
/// - greater than or equal to (`>=`)
/// - greater than (`>`)
///
/// the specified version.
///
/// See [alpm-comparison] for details on the format.
///
/// ## Note
///
/// The variants of this enum are sorted in a way, that prefers the two-letter comparators over
/// the one-letter ones.
/// This is because when splitting a string on the string representation of [`VersionComparison`]
/// variant and relying on the ordering of [`strum::EnumIter`], the two-letter comparators must be
/// checked before checking the one-letter ones to yield robust results.
///
/// [alpm-comparison]: https://alpm.archlinux.page/specifications/alpm-comparison.7.html
#[derive(
    strum::AsRefStr,
    Clone,
    Copy,
    Debug,
    strum::Display,
    strum::EnumIter,
    PartialEq,
    Eq,
    strum::VariantNames,
    Serialize,
    Deserialize,
)]
pub enum VersionComparison {
    /// Less than or equal to
    #[strum(to_string = "<=")]
    LessOrEqual,

    /// Greater than or equal to
    #[strum(to_string = ">=")]
    GreaterOrEqual,

    /// Equal to
    #[strum(to_string = "=")]
    Equal,

    /// Less than
    #[strum(to_string = "<")]
    Less,

    /// Greater than
    #[strum(to_string = ">")]
    Greater,
}

impl VersionComparison {
    /// Returns `true` if the result of a comparison between the actual and required package
    /// versions satisfies the comparison function.
    fn is_compatible_with(self, ord: Ordering) -> bool {
        match (self, ord) {
            (VersionComparison::Less, Ordering::Less)
            | (VersionComparison::LessOrEqual, Ordering::Less | Ordering::Equal)
            | (VersionComparison::Equal, Ordering::Equal)
            | (VersionComparison::GreaterOrEqual, Ordering::Greater | Ordering::Equal)
            | (VersionComparison::Greater, Ordering::Greater) => true,

            (VersionComparison::Less, Ordering::Equal | Ordering::Greater)
            | (VersionComparison::LessOrEqual, Ordering::Greater)
            | (VersionComparison::Equal, Ordering::Less | Ordering::Greater)
            | (VersionComparison::GreaterOrEqual, Ordering::Less)
            | (VersionComparison::Greater, Ordering::Less | Ordering::Equal) => false,
        }
    }

    /// Recognizes a [`VersionComparison`] in a string slice.
    ///
    /// Consumes all of its input.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is not a valid _alpm-comparison_.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        alt((
            // insert eofs here (instead of after alt call) so correct error message is thrown
            ("<=", eof).value(Self::LessOrEqual),
            (">=", eof).value(Self::GreaterOrEqual),
            ("=", eof).value(Self::Equal),
            ("<", eof).value(Self::Less),
            (">", eof).value(Self::Greater),
            fail.context(StrContext::Label("comparison operator"))
                .context_with(iter_str_context!([VersionComparison::VARIANTS])),
        ))
        .parse_next(input)
    }
}

impl FromStr for VersionComparison {
    type Err = Error;

    /// Creates a new [`VersionComparison`] from a string slice.
    ///
    /// Delegates to [`VersionComparison::parser`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`VersionComparison::parser`] fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use testresult::TestResult;

    use super::*;
    /// Ensure that valid version comparison strings can be parsed.
    #[rstest]
    #[case("<", VersionComparison::Less)]
    #[case("<=", VersionComparison::LessOrEqual)]
    #[case("=", VersionComparison::Equal)]
    #[case(">=", VersionComparison::GreaterOrEqual)]
    #[case(">", VersionComparison::Greater)]
    fn valid_version_comparison(#[case] comparison: &str, #[case] expected: VersionComparison) {
        assert_eq!(comparison.parse(), Ok(expected));
    }

    /// Ensure that invalid version comparisons will throw an error.
    #[rstest]
    #[case("", "invalid comparison operator")]
    #[case("<<", "invalid comparison operator")]
    #[case("==", "invalid comparison operator")]
    #[case("!=", "invalid comparison operator")]
    #[case(" =", "invalid comparison operator")]
    #[case("= ", "invalid comparison operator")]
    #[case("<1", "invalid comparison operator")]
    fn invalid_version_comparison(#[case] comparison: &str, #[case] err_snippet: &str) {
        let Err(Error::ParseError(err_msg)) = VersionComparison::from_str(comparison) else {
            panic!("'{comparison}' did not fail as expected")
        };
        assert!(
            err_msg.contains(err_snippet),
            "Error:\n=====\n{err_msg}\n=====\nshould contain snippet:\n\n{err_snippet}"
        );
    }

    /// Test successful parsing for version requirement strings.
    #[rstest]
    #[case("=1", VersionRequirement {
        comparison: VersionComparison::Equal,
        version: Version::from_str("1").unwrap(),
    })]
    #[case("<=42:abcd-2.4", VersionRequirement {
        comparison: VersionComparison::LessOrEqual,
        version: Version::from_str("42:abcd-2.4").unwrap(),
    })]
    #[case(">3.1", VersionRequirement {
        comparison: VersionComparison::Greater,
        version: Version::from_str("3.1").unwrap(),
    })]
    fn valid_version_requirement(#[case] requirement: &str, #[case] expected: VersionRequirement) {
        assert_eq!(
            requirement.parse(),
            Ok(expected),
            "Expected successful parse for version requirement '{requirement}'"
        );
    }

    #[rstest]
    #[case::bad_operator("<>3.1", "invalid comparison operator")]
    #[case::no_operator("3.1", "expected version comparison operator")]
    #[case::arrow_operator("=>3.1", "invalid comparison operator")]
    #[case::no_version("<=", "expected pkgver string")]
    fn invalid_version_requirement(#[case] requirement: &str, #[case] err_snippet: &str) {
        let Err(Error::ParseError(err_msg)) = VersionRequirement::from_str(requirement) else {
            panic!("'{requirement}' erroneously parsed as VersionRequirement")
        };
        assert!(
            err_msg.contains(err_snippet),
            "Error:\n=====\n{err_msg}\n=====\nshould contain snippet:\n\n{err_snippet}"
        );
    }

    #[rstest]
    #[case("<3.1>3.2", "invalid pkgver character")]
    fn invalid_version_requirement_pkgver_parse(
        #[case] requirement: &str,
        #[case] err_snippet: &str,
    ) {
        let Err(Error::ParseError(err_msg)) = VersionRequirement::from_str(requirement) else {
            panic!("'{requirement}' erroneously parsed as VersionRequirement")
        };
        assert!(
            err_msg.contains(err_snippet),
            "Error:\n=====\n{err_msg}\n=====\nshould contain snippet:\n\n{err_snippet}"
        );
    }

    /// Check whether a version requirement (>= 1.0) is fulfilled by a given version string.
    #[rstest]
    #[case("=1", "1", true)]
    #[case("=1", "1.0", false)]
    #[case("=1", "1-1", true)]
    #[case("=1", "1:1", false)]
    #[case("=1", "0.9", false)]
    #[case("<42", "41", true)]
    #[case("<42", "42", false)]
    #[case("<42", "43", false)]
    #[case("<=42", "41", true)]
    #[case("<=42", "42", true)]
    #[case("<=42", "43", false)]
    #[case(">42", "41", false)]
    #[case(">42", "42", false)]
    #[case(">42", "43", true)]
    #[case(">=42", "41", false)]
    #[case(">=42", "42", true)]
    #[case(">=42", "43", true)]
    fn version_requirement_satisfied(
        #[case] requirement: &str,
        #[case] version: &str,
        #[case] result: bool,
    ) {
        let requirement = VersionRequirement::from_str(requirement).unwrap();
        let version = Version::from_str(version).unwrap();
        assert_eq!(requirement.is_satisfied_by(&version), result);
    }

    #[rstest]
    #[case::self_less_matching_other_less("<1", "<1")]
    #[case::self_less_matching_other_less_or_equal("<1", "<=1")]
    #[case::self_less_bigger_other_less("<1", "<2")]
    #[case::self_less_bigger_other_less_or_equal("<1", "<=2")]
    #[case::self_less_smaller_other_less("<1", "<0.1")]
    #[case::self_less_smaller_other_less_or_equal("<1", "<=0.1")]
    #[case::self_less_smaller_other_equal("<1", "=0.1")]
    #[case::self_less_smaller_other_greater_or_equal("<1", ">=0.1")]
    #[case::self_less_smaller_other_greater("<1", ">0.1")]
    #[case::self_less_smaller_other_equal("<1", "=0.1")]
    #[case::self_less_or_equal_matching_other_less("<=1", "<1")]
    #[case::self_less_or_equal_matching_other_less_or_equal("<=1", "<=1")]
    #[case::self_less_or_equal_matching_other_equal("<=1", "=1")]
    #[case::self_less_or_equal_matching_other_greater_or_equal("<=1", ">=1")]
    #[case::self_less_or_equal_bigger_other_less("<=1", "<2")]
    #[case::self_less_or_equal_bigger_other_less_or_equal("<=1", "<=2")]
    #[case::self_less_or_equal_smaller_other_greater_or_equal("<=1", ">=0.1")]
    #[case::self_less_or_equal_smaller_other_greater("<=1", ">0.1")]
    #[case::self_equal_matching_other_less_or_equal("=1", "<=1")]
    #[case::self_equal_matching_other_equal("=1", "=1")]
    #[case::self_equal_matching_other_greater_or_equal("=1", ">=1")]
    #[case::self_greater_or_equal_matching_other_less_or_equal(">=1", "<=1")]
    #[case::self_greater_or_equal_matching_other_equal(">=1", "=1")]
    #[case::self_greater_or_equal_matching_other_greater_or_equal(">=1", ">=1")]
    #[case::self_greater_or_equal_matching_other_greater(">=1", ">1")]
    #[case::self_greater_or_equal_bigger_other_less(">=1", "<2")]
    #[case::self_greater_or_equal_bigger_other_less_or_equal(">=1", "<=2")]
    #[case::self_greater_or_equal_bigger_other_equal(">=1", "=2")]
    #[case::self_greater_or_equal_bigger_other_greater_or_equal(">=1", ">=2")]
    #[case::self_greater_or_equal_bigger_other_greater(">=1", ">2")]
    #[case::self_greater_or_equal_smaller_other_greater_or_equal(">=1", ">=0.1")]
    #[case::self_greater_or_equal_smaller_other_greater(">=1", ">0.1")]
    #[case::self_greater_matching_other_greater_or_equal(">1", ">=1")]
    #[case::self_greater_matching_other_greater(">1", ">1")]
    #[case::self_greater_bigger_other_less(">1", "<2")]
    #[case::self_greater_bigger_other_less_or_equal(">1", "<=2")]
    #[case::self_greater_bigger_other_equal(">1", "=2")]
    #[case::self_greater_bigger_other_greater_or_equal(">1", ">=2")]
    #[case::self_greater_bigger_other_greater(">1", ">2")]
    #[case::self_greater_smaller_other_greater_or_equal(">1", ">=0.1")]
    #[case::self_greater_smaller_other_greater(">1", ">0.1")]
    fn version_requirements_form_intersection(
        #[case] self_requirement: &str,
        #[case] other_requirement: &str,
    ) -> TestResult {
        let self_requirement: VersionRequirement = self_requirement.parse()?;
        let other_requirement: VersionRequirement = other_requirement.parse()?;

        assert!(self_requirement.is_intersection(&other_requirement));

        Ok(())
    }

    #[rstest]
    #[case::self_less_matching_other_equal("<1", "=1")]
    #[case::self_less_matching_other_greater_or_equal("<1", ">=1")]
    #[case::self_less_matching_other_greater("<1", ">1")]
    #[case::self_less_or_equal_matching_other_greater("<=1", ">1")]
    #[case::self_equal_matching_other_less("=1", "<1")]
    #[case::self_equal_matching_other_greater("=1", ">1")]
    #[case::self_equal_bigger_other_less("=1", "<2")]
    #[case::self_equal_bigger_other_greater("=1", ">2")]
    #[case::self_equal_smaller_other_less("=1", "<0.1")]
    #[case::self_equal_smaller_other_greater("=1", ">0.1")]
    #[case::self_greater_or_equal_matching_other_less(">=1", "<1")]
    #[case::self_greater_matching_other_less(">1", "<1")]
    #[case::self_greater_matching_other_less_or_equal(">1", "<=1")]
    #[case::self_greater_matching_other_equal(">1", "=1")]
    fn version_requirements_do_not_form_intersection(
        #[case] self_requirement: &str,
        #[case] other_requirement: &str,
    ) -> TestResult {
        let self_requirement: VersionRequirement = self_requirement.parse()?;
        let other_requirement: VersionRequirement = other_requirement.parse()?;

        assert!(!self_requirement.is_intersection(&other_requirement));
        Ok(())
    }
}
