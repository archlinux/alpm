//! A flexible and generic package version.

use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use winnow::{
    ModalResult,
    Parser,
    combinator::{cut_err, eof, opt, preceded, seq, terminated},
    error::{StrContext, StrContextValue},
    token::take_till,
};

use crate::{Epoch, Error, PackageRelease, PackageVersion};
#[cfg(doc)]
use crate::{FullVersion, MinimalVersion};

/// A version of a package
///
/// A [`Version`] generically tracks an optional [`Epoch`], a [`PackageVersion`] and an optional
/// [`PackageRelease`].
/// See [alpm-package-version] for details on the format.
///
/// # Notes
///
/// - If [`PackageRelease`] should be mandatory for your use-case, use [`FullVersion`] instead.
/// - If [`PackageRelease`] should not be used in your use-case, use [`MinimalVersion`] instead.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Epoch, PackageRelease, PackageVersion, Version};
///
/// # fn main() -> Result<(), alpm_types::Error> {
///
/// let version = Version::from_str("1:2-3")?;
/// assert_eq!(version.epoch, Some(Epoch::from_str("1")?));
/// assert_eq!(version.pkgver, PackageVersion::new("2".to_string())?);
/// assert_eq!(version.pkgrel, Some(PackageRelease::new(3, None)));
/// # Ok(())
/// # }
/// ```
///
/// [alpm-package-version]: https://alpm.archlinux.page/specifications/alpm-package-version.7.html
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Version {
    /// The version of the package
    pub pkgver: PackageVersion,
    /// The epoch of the package
    pub epoch: Option<Epoch>,
    /// The release of the package
    pub pkgrel: Option<PackageRelease>,
}

impl Version {
    /// Create a new Version
    pub fn new(
        pkgver: PackageVersion,
        epoch: Option<Epoch>,
        pkgrel: Option<PackageRelease>,
    ) -> Self {
        Version {
            pkgver,
            epoch,
            pkgrel,
        }
    }

    /// Compare two Versions and return a number
    ///
    /// The comparison algorithm is based on libalpm/ pacman's vercmp behavior.
    ///
    /// * `1` if `a` is newer than `b`
    /// * `0` if `a` and `b` are considered to be the same version
    /// * `-1` if `a` is older than `b`
    ///
    /// ## Examples
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::Version;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    ///
    /// assert_eq!(
    ///     Version::vercmp(&Version::from_str("1.0.0")?, &Version::from_str("0.1.0")?),
    ///     1
    /// );
    /// assert_eq!(
    ///     Version::vercmp(&Version::from_str("1.0.0")?, &Version::from_str("1.0.0")?),
    ///     0
    /// );
    /// assert_eq!(
    ///     Version::vercmp(&Version::from_str("0.1.0")?, &Version::from_str("1.0.0")?),
    ///     -1
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn vercmp(a: &Version, b: &Version) -> i8 {
        match a.cmp(b) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }

    /// Recognizes a [`Version`] in a string slice.
    ///
    /// Consumes all of its input.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is not a valid _alpm-package-version_.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        let mut epoch = opt(terminated(take_till(1.., ':'), ':').and_then(
            // cut_err now that we've found a pattern with ':'
            cut_err(Epoch::parser),
        ))
        .context(StrContext::Expected(StrContextValue::Description(
            "followed by a ':'",
        )));

        seq!(Self {
            epoch: epoch,
            pkgver: take_till(1.., '-')
                // this context will trigger on empty pkgver due to 1.. above
                .context(StrContext::Expected(StrContextValue::Description("pkgver string")))
                .and_then(PackageVersion::parser),
            pkgrel: opt(preceded('-', cut_err(PackageRelease::parser))),
            _: eof.context(StrContext::Expected(StrContextValue::Description("end of version string"))),
        })
        .parse_next(input)
    }
}

impl FromStr for Version {
    type Err = Error;
    /// Creates a new [`Version`] from a string slice.
    ///
    /// Delegates to [`Version::parser`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`Version::parser`] fails.
    fn from_str(s: &str) -> Result<Version, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

impl Display for Version {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        if let Some(epoch) = self.epoch {
            write!(fmt, "{epoch}:")?;
        }

        write!(fmt, "{}", self.pkgver)?;

        if let Some(pkgrel) = &self.pkgrel {
            write!(fmt, "-{pkgrel}")?;
        }

        Ok(())
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.epoch, other.epoch) {
            (Some(self_epoch), Some(other_epoch)) if self_epoch.cmp(&other_epoch).is_ne() => {
                return self_epoch.cmp(&other_epoch);
            }
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
            (_, _) => {}
        }

        let pkgver_cmp = self.pkgver.cmp(&other.pkgver);
        if pkgver_cmp.is_ne() {
            return pkgver_cmp;
        }

        self.pkgrel.cmp(&other.pkgrel)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use rstest::rstest;

    use super::*;

    /// Ensure that valid version strings are parsed as expected.
    #[rstest]
    #[case(
        "foo",
        Version {
            epoch: None,
            pkgver: PackageVersion::new("foo".to_string()).unwrap(),
            pkgrel: None
        },
    )]
    #[case(
        "1:foo-1",
        Version {
            pkgver: PackageVersion::new("foo".to_string()).unwrap(),
            epoch: Some(Epoch::new(NonZeroUsize::new(1).unwrap())),
            pkgrel: Some(PackageRelease::new(1, None))
        },
    )]
    #[case(
        "1:foo",
        Version {
            pkgver: PackageVersion::new("foo".to_string()).unwrap(),
            epoch: Some(Epoch::new(NonZeroUsize::new(1).unwrap())),
            pkgrel: None,
        },
    )]
    #[case(
        "foo-1",
        Version {
            pkgver: PackageVersion::new("foo".to_string()).unwrap(),
            epoch: None,
            pkgrel: Some(PackageRelease::new(1, None))
        }
    )]
    // yes, this is valid
    #[case(
        ".-1",
        Version {
            pkgver: PackageVersion::new(".".to_string()).unwrap(),
            epoch: None,
            pkgrel: Some(PackageRelease::new(1, None))
            }
    )]
    fn valid_version_from_string(#[case] version: &str, #[case] expected: Version) {
        assert_eq!(
            Version::from_str(version),
            Ok(expected),
            "Expected valid parsing for version {version}"
        )
    }

    /// Ensure that invalid version strings produce the respective errors.
    #[rstest]
    #[case::two_pkgrel("1:foo-1-1", "expected end of package release value")]
    #[case::two_epoch("1:1:foo-1", "invalid pkgver character")]
    #[case::no_version("", "expected pkgver string")]
    #[case::no_version(":", "invalid pkgver character")]
    #[case::invalid_integer(
        "-1foo:1",
        "invalid package epoch\nexpected positive non-zero decimal integer, followed by a ':'"
    )]
    #[case::invalid_integer(
        "1-foo:1",
        "invalid package epoch\nexpected positive non-zero decimal integer, followed by a ':'"
    )]
    fn parse_error_in_version_from_string(#[case] version: &str, #[case] err_snippet: &str) {
        let Err(Error::ParseError(err_msg)) = Version::from_str(version) else {
            panic!("parsing '{version}' did not fail as expected")
        };
        assert!(
            err_msg.contains(err_snippet),
            "Error:\n=====\n{err_msg}\n=====\nshould contain snippet:\n\n{err_snippet}"
        );
    }

    /// Ensure that versions are properly serialized back to their string representation.
    #[rstest]
    #[case(Version::from_str("1:1-1").unwrap(), "1:1-1")]
    #[case(Version::from_str("1-1").unwrap(), "1-1")]
    #[case(Version::from_str("1").unwrap(), "1")]
    #[case(Version::from_str("1:1").unwrap(), "1:1")]
    fn version_to_string(#[case] version: Version, #[case] to_str: &str) {
        assert_eq!(format!("{version}"), to_str);
    }

    #[rstest]
    // Major version comparisons
    #[case(Version::from_str("1"), Version::from_str("1"), Ordering::Equal)]
    #[case(Version::from_str("1"), Version::from_str("2"), Ordering::Less)]
    #[case(
        Version::from_str("20220102"),
        Version::from_str("20220202"),
        Ordering::Less
    )]
    // Major vs Major.Minor
    #[case(Version::from_str("1"), Version::from_str("1.1"), Ordering::Less)]
    #[case(Version::from_str("01"), Version::from_str("1"), Ordering::Equal)]
    #[case(Version::from_str("001a"), Version::from_str("1a"), Ordering::Equal)]
    #[case(Version::from_str("a1a"), Version::from_str("a1b"), Ordering::Less)]
    #[case(Version::from_str("foo"), Version::from_str("1.1"), Ordering::Less)]
    // Major.Minor version comparisons
    #[case(Version::from_str("1.0"), Version::from_str("1..0"), Ordering::Less)]
    #[case(Version::from_str("1.1"), Version::from_str("1.1"), Ordering::Equal)]
    #[case(Version::from_str("1.1"), Version::from_str("1.2"), Ordering::Less)]
    #[case(Version::from_str("1..0"), Version::from_str("1..0"), Ordering::Equal)]
    #[case(Version::from_str("1..0"), Version::from_str("1..1"), Ordering::Less)]
    #[case(Version::from_str("1+0"), Version::from_str("1.0"), Ordering::Equal)]
    #[case(Version::from_str("1+1"), Version::from_str("1+2"), Ordering::Less)]
    // Major.Minor version comparisons with alphanumerics
    #[case(Version::from_str("1.1"), Version::from_str("1.1.a"), Ordering::Less)]
    #[case(Version::from_str("1.1"), Version::from_str("1.11a"), Ordering::Less)]
    #[case(Version::from_str("1.1"), Version::from_str("1.1_a"), Ordering::Less)]
    #[case(Version::from_str("1.1a"), Version::from_str("1.1"), Ordering::Less)]
    #[case(Version::from_str("1.1a1"), Version::from_str("1.1"), Ordering::Less)]
    #[case(Version::from_str("1.a"), Version::from_str("1.1"), Ordering::Less)]
    #[case(Version::from_str("1.a"), Version::from_str("1.alpha"), Ordering::Less)]
    #[case(Version::from_str("1.a1"), Version::from_str("1.1"), Ordering::Less)]
    #[case(Version::from_str("1.a11"), Version::from_str("1.1"), Ordering::Less)]
    #[case(Version::from_str("1.a1a"), Version::from_str("1.a1"), Ordering::Less)]
    #[case(Version::from_str("1.alpha"), Version::from_str("1.b"), Ordering::Less)]
    #[case(Version::from_str("a.1"), Version::from_str("1.1"), Ordering::Less)]
    #[case(
        Version::from_str("1.alpha0.0"),
        Version::from_str("1.alpha.0"),
        Ordering::Less
    )]
    // Major.Minor vs Major.Minor.Patch
    #[case(Version::from_str("1.0"), Version::from_str("1.0."), Ordering::Less)]
    // Major.Minor.Patch
    #[case(Version::from_str("1.0."), Version::from_str("1.0.0"), Ordering::Less)]
    #[case(Version::from_str("1.0.."), Version::from_str("1.0."), Ordering::Equal)]
    #[case(
        Version::from_str("1.0.alpha.0"),
        Version::from_str("1.0."),
        Ordering::Less
    )]
    #[case(
        Version::from_str("1.a001a.1"),
        Version::from_str("1.a1a.1"),
        Ordering::Equal
    )]
    fn version_cmp(
        #[case] version_a: Result<Version, Error>,
        #[case] version_b: Result<Version, Error>,
        #[case] expected: Ordering,
    ) {
        // Simply unwrap the Version as we expect all test strings to be valid.
        let version_a = version_a.unwrap();
        let version_b = version_b.unwrap();

        // Derive the expected vercmp binary exitcode from the expected Ordering.
        let vercmp_result = match &expected {
            Ordering::Equal => 0,
            Ordering::Greater => 1,
            Ordering::Less => -1,
        };

        let ordering = version_a.cmp(&version_b);
        assert_eq!(
            ordering, expected,
            "Failed to compare '{version_a}' and '{version_b}'. Expected {expected:?} got {ordering:?}"
        );

        assert_eq!(Version::vercmp(&version_a, &version_b), vercmp_result);

        // If we find the `vercmp` binary, also run the test against the actual binary.
        #[cfg(feature = "compatibility_tests")]
        {
            let output = std::process::Command::new("vercmp")
                .arg(version_a.to_string())
                .arg(version_b.to_string())
                .output()
                .unwrap();
            let result = String::from_utf8_lossy(&output.stdout);
            assert_eq!(result.trim(), vercmp_result.to_string());
        }

        // Now check that the opposite holds true as well.
        let reverse_vercmp_result = match &expected {
            Ordering::Equal => 0,
            Ordering::Greater => -1,
            Ordering::Less => 1,
        };
        let reverse_expected = match &expected {
            Ordering::Equal => Ordering::Equal,
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
        };

        let reverse_ordering = version_b.cmp(&version_a);
        assert_eq!(
            reverse_ordering, reverse_expected,
            "Failed to compare '{version_a}' and '{version_b}'. Expected {expected:?} got {ordering:?}"
        );

        assert_eq!(
            Version::vercmp(&version_b, &version_a),
            reverse_vercmp_result
        );
    }
}
