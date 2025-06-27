//! The [alpm-package-version] form _full_ and _full with epoch_.
//!
//! [alpm-package-version]: https://alpm.archlinux.page/specifications/alpm-package-version.7.html

use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use winnow::{
    ModalResult,
    Parser,
    combinator::{cut_err, eof, opt, preceded, terminated},
    error::{StrContext, StrContextValue},
    token::{take_till, take_until},
};

use crate::{Epoch, Error, PackageRelease, PackageVersion, Version};

/// A package version with mandatory [`PackageRelease`].
///
/// Tracks an optional [`Epoch`], a [`PackageVersion`] and a [`PackageRelease`].
/// This reflects the _full_ and _full with epoch_ forms of [alpm-package-version].
///
/// # Note
///
/// If [`PackageRelease`] should be optional for your use-case, use [`Version`] instead.
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::FullVersion;
///
/// # fn main() -> testresult::TestResult {
/// // A full version.
/// let version = FullVersion::from_str("1.0.0-1")?;
///
/// // A full version with epoch.
/// let version = FullVersion::from_str("1:1.0.0-1")?;
/// # Ok(())
/// # }
/// ```
///
/// [alpm-package-version]: https://alpm.archlinux.page/specifications/alpm-package-version.7.html
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FullVersion {
    /// The version of the package
    pub pkgver: PackageVersion,
    /// The release of the package
    pub pkgrel: PackageRelease,
    /// The epoch of the package
    pub epoch: Option<Epoch>,
}

impl FullVersion {
    /// Creates a new [`FullVersion`].
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_types::{Epoch, FullVersion, PackageRelease, PackageVersion};
    ///
    /// # fn main() -> testresult::TestResult {
    /// // A full version.
    /// let version = FullVersion::new(
    ///     PackageVersion::new("1.0.0".to_string())?,
    ///     PackageRelease::new(1, None),
    ///     None,
    /// );
    ///
    /// // A full version with epoch.
    /// let version = FullVersion::new(
    ///     PackageVersion::new("1.0.0".to_string())?,
    ///     PackageRelease::new(1, None),
    ///     Some(Epoch::new(1.try_into()?)),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(pkgver: PackageVersion, pkgrel: PackageRelease, epoch: Option<Epoch>) -> Self {
        Self {
            pkgver,
            pkgrel,
            epoch,
        }
    }

    /// Compares `self` to another [`FullVersion`] and returns a number.
    ///
    /// - `1` if `self` is newer than `other`
    /// - `0` if `self` and `other` are equal
    /// - `-1` if `self` is older than `other`
    ///
    /// This output behavior is based on the behavior of the [vercmp] tool.
    ///
    /// Delegates to [`FullVersion::cmp`] for comparison.
    /// The rules and algorithms used for comparison are explained in more detail in
    /// [alpm-package-version] and [alpm-pkgver].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::FullVersion;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// assert_eq!(
    ///     FullVersion::from_str("1.0.0-1")?.vercmp(&FullVersion::from_str("0.1.0-1")?),
    ///     1
    /// );
    /// assert_eq!(
    ///     FullVersion::from_str("1.0.0-1")?.vercmp(&FullVersion::from_str("1.0.0-1")?),
    ///     0
    /// );
    /// assert_eq!(
    ///     FullVersion::from_str("0.1.0-1")?.vercmp(&FullVersion::from_str("1.0.0-1")?),
    ///     -1
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [alpm-package-version]: https://alpm.archlinux.page/specifications/alpm-package-version.7.html
    /// [alpm-pkgver]: https://alpm.archlinux.page/specifications/alpm-pkgver.7.html
    /// [vercmp]: https://man.archlinux.org/man/vercmp.8
    pub fn vercmp(&self, other: &FullVersion) -> i8 {
        match self.cmp(other) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }

    /// Recognizes a [`FullVersion`] in a string slice.
    ///
    /// Consumes all of its input.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is not a valid [alpm-package-version] (_full_ or _full with
    /// epoch_).
    ///
    /// [alpm-package-version]: https://alpm.archlinux.page/specifications/alpm-package-version.7.html
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        // Advance the parser until after a ':' if there is one, e.g.:
        // "1:1.0.0-1" -> "1.0.0-1"
        let epoch = opt(terminated(take_till(1.., ':'), ':').and_then(
            // cut_err now that we've found a pattern with ':'
            cut_err(Epoch::parser),
        ))
        .context(StrContext::Expected(StrContextValue::Description(
            "followed by a ':'",
        )))
        .parse_next(input)?;

        // Advance the parser until the next '-', e.g.:
        // "1.0.0-1" -> "-1"
        let pkgver: PackageVersion = cut_err(take_until(0.., "-"))
            .context(StrContext::Expected(StrContextValue::Description(
                "alpm-pkgver string, followed by a '-' and an alpm-pkgrel string",
            )))
            .take()
            .and_then(cut_err(PackageVersion::parser))
            .parse_next(input)?;

        // Consume the delimiter '-'
        // "-1" -> "1"
        // and parse everything until eof as a PackageRelease, e.g.:
        // "1" -> ""
        let pkgrel: PackageRelease = preceded("-", cut_err(PackageRelease::parser))
            .context(StrContext::Expected(StrContextValue::Description(
                "alpm-pkgrel string",
            )))
            .parse_next(input)?;

        // Ensure that there are no trailing chars left.
        eof.context(StrContext::Expected(StrContextValue::Description(
            "end of full alpm-package-version string",
        )))
        .parse_next(input)?;

        Ok(Self {
            epoch,
            pkgver,
            pkgrel,
        })
    }
}

impl Display for FullVersion {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        if let Some(epoch) = self.epoch {
            write!(fmt, "{epoch}:")?;
        }
        write!(fmt, "{}-{}", self.pkgver, self.pkgrel)?;

        Ok(())
    }
}

impl FromStr for FullVersion {
    type Err = Error;
    /// Creates a new [`FullVersion`] from a string slice.
    ///
    /// Delegates to [`FullVersion::parser`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`Version::parser`] fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

impl Ord for FullVersion {
    /// Compares `self` to another [`FullVersion`].
    ///
    /// The comparison rules and algorithms are explained in more detail in [alpm-package-version]
    /// and [alpm-pkgver].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{cmp::Ordering, str::FromStr};
    ///
    /// use alpm_types::FullVersion;
    ///
    /// # fn main() -> testresult::TestResult {
    /// // Examples for "full"
    /// let version_a = FullVersion::from_str("1.0.0-1")?;
    /// let version_b = FullVersion::from_str("1.0.0-2")?;
    /// assert_eq!(version_a.cmp(&version_b), Ordering::Less);
    /// assert_eq!(version_b.cmp(&version_a), Ordering::Greater);
    ///
    /// let version_a = FullVersion::from_str("1.0.0-1")?;
    /// let version_b = FullVersion::from_str("1.0.0-1")?;
    /// assert_eq!(version_a.cmp(&version_b), Ordering::Equal);
    ///
    /// // Examples for "full with epoch"
    /// let version_a = FullVersion::from_str("1:1.0.0-1")?;
    /// let version_b = FullVersion::from_str("1.0.0-2")?;
    /// assert_eq!(version_a.cmp(&version_b), Ordering::Greater);
    /// assert_eq!(version_b.cmp(&version_a), Ordering::Less);
    ///
    /// let version_a = FullVersion::from_str("1:1.0.0-1")?;
    /// let version_b = FullVersion::from_str("1:1.0.0-1")?;
    /// assert_eq!(version_a.cmp(&version_b), Ordering::Equal);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [alpm-package-version]: https://alpm.archlinux.page/specifications/alpm-package-version.7.html
    /// [alpm-pkgver]: https://alpm.archlinux.page/specifications/alpm-pkgver.7.html
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

impl PartialOrd for FullVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl TryFrom<Version> for FullVersion {
    type Error = crate::Error;

    /// Creates a [`FullVersion`] from a [`Version`].
    ///
    /// # Errors
    ///
    /// Returns an error if `value.pkgrel` is [`None`].
    fn try_from(value: Version) -> Result<Self, Self::Error> {
        Ok(Self {
            pkgver: value.pkgver,
            pkgrel: value.pkgrel.ok_or(Error::MissingComponent {
                component: "pkgrel",
            })?,
            epoch: value.epoch,
        })
    }
}

impl TryFrom<&Version> for FullVersion {
    type Error = crate::Error;

    /// Creates a [`FullVersion`] from a [`Version`] reference.
    ///
    /// # Errors
    ///
    /// Returns an error if `value.pkgrel` is [`None`].
    fn try_from(value: &Version) -> Result<Self, Self::Error> {
        Self::try_from(value.clone())
    }
}

impl From<FullVersion> for Version {
    /// Creates a [`Version`] from a [`FullVersion`].
    fn from(value: FullVersion) -> Self {
        Self {
            pkgver: value.pkgver,
            pkgrel: Some(value.pkgrel),
            epoch: value.epoch,
        }
    }
}

impl From<&FullVersion> for Version {
    /// Creates a [`Version`] from a [`FullVersion`] reference.
    fn from(value: &FullVersion) -> Self {
        Self::from(value.clone())
    }
}

#[cfg(test)]
mod tests {
    use log::{LevelFilter, debug};
    use rstest::rstest;
    use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
    use testresult::TestResult;

    use super::*;

    /// Initialize a logger that shows trace messages on stderr.
    fn init_logger() {
        if TermLogger::init(
            LevelFilter::Trace,
            Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        )
        .is_err()
        {
            debug!("Not initializing another logger, as one is initialized already.");
        }
    }

    /// Ensures that valid [`FullVersion`] strings are parsed successfully as expected.
    #[rstest]
    #[case::full_with_epoch(
        "1:foo-1",
        FullVersion {
            pkgver: PackageVersion::from_str("foo")?,
            epoch: Some(Epoch::from_str("1")?),
            pkgrel: PackageRelease::from_str("1")?,
        },
    )]
    #[case::full(
        "foo-1",
        FullVersion {
            pkgver: PackageVersion::from_str("foo")?,
            epoch: None,
            pkgrel: PackageRelease::from_str("1")?
        }
    )]
    fn valid_full_version_from_string(
        #[case] version: &str,
        #[case] expected: FullVersion,
    ) -> TestResult {
        init_logger();

        assert_eq!(
            FullVersion::from_str(version),
            Ok(expected),
            "Expected valid parsing for FullVersion {version}"
        );

        Ok(())
    }

    /// Ensures that invalid [`FullVersion`] strings lead to parse errors.
    #[rstest]
    #[case::two_pkgrel("1:foo-1-1", "expected end of package release value")]
    #[case::two_epoch("1:1:foo-1", "invalid pkgver character")]
    #[case::empty_string(
        "",
        "expected alpm-pkgver string, followed by a '-' and an alpm-pkgrel string"
    )]
    #[case::colon(
        ":",
        "expected alpm-pkgver string, followed by a '-' and an alpm-pkgrel string"
    )]
    #[case::dot(
        ".",
        "expected alpm-pkgver string, followed by a '-' and an alpm-pkgrel string"
    )]
    #[case::no_pkgrel_with_epoch(
        "1:1.0.0",
        "expected alpm-pkgver string, followed by a '-' and an alpm-pkgrel string"
    )]
    #[case::no_pkgrel(
        "1.0.0",
        "expected alpm-pkgver string, followed by a '-' and an alpm-pkgrel string"
    )]
    #[case::no_pkgrel_dash_end(
        "1.0.0-",
        "invalid package release\nexpected positive decimal integer, alpm-pkgrel string"
    )]
    #[case::starts_with_dash(
        "-1foo:1",
        "invalid package epoch\nexpected positive non-zero decimal integer, followed by a ':'"
    )]
    #[case::ends_with_colon(
        "1-foo:",
        "invalid package epoch\nexpected positive non-zero decimal integer, followed by a ':'"
    )]
    #[case::ends_with_colon_number(
        "1-foo:1",
        "invalid package epoch\nexpected positive non-zero decimal integer, followed by a ':'"
    )]
    fn parse_error_in_full_version_from_string(#[case] version: &str, #[case] err_snippet: &str) {
        init_logger();

        let Err(Error::ParseError(err_msg)) = FullVersion::from_str(version) else {
            panic!("parsing '{version}' as FullVersion did not fail as expected")
        };
        assert!(
            err_msg.contains(err_snippet),
            "Error:\n=====\n{err_msg}\n=====\nshould contain snippet:\n\n{err_snippet}"
        );
    }

    /// Ensures that [`FullVersion`] can be created from valid/compatible [`Version`] (and
    /// [`Version`] reference) and fails otherwise.
    #[rstest]
    #[case::full_with_epoch(Version::from_str("1:1.0.0-1")?, Ok(FullVersion::from_str("1:1.0.0-1")?))]
    #[case::full(Version::from_str("1.0.0-1")?, Ok(FullVersion::from_str("1.0.0-1")?))]
    #[case::minimal_with_epoch(Version::from_str("1:1.0.0")?, Err(Error::MissingComponent{component: "pkgrel"}))]
    #[case::minimal(Version::from_str("1.0.0")?, Err(Error::MissingComponent{component: "pkgrel"}))]
    fn full_version_try_from_version(
        #[case] version: Version,
        #[case] expected: Result<FullVersion, Error>,
    ) -> TestResult {
        assert_eq!(FullVersion::try_from(&version), expected);
        assert_eq!(FullVersion::try_from(version), expected);
        Ok(())
    }

    /// Ensures that [`Version`] can be created from [`FullVersion`] (and [`FullVersion`]
    /// reference).
    #[rstest]
    #[case::full_with_epoch(Version::from_str("1:1.0.0-1")?, FullVersion::from_str("1:1.0.0-1")?)]
    #[case::full(Version::from_str("1.0.0-1")?, FullVersion::from_str("1.0.0-1")?)]
    fn version_from_full_version(
        #[case] version: Version,
        #[case] full_version: FullVersion,
    ) -> TestResult {
        assert_eq!(Version::from(&full_version), version);
        Ok(())
    }

    /// Ensures that [`FullVersion`] is properly serialized back to its string representation.
    #[rstest]
    #[case::with_epoch("1:1-1")]
    #[case::plain("1-1")]
    fn full_version_to_string(#[case] input: &str) -> TestResult {
        assert_eq!(format!("{}", FullVersion::from_str(input)?), input);
        Ok(())
    }

    /// Ensures that [`FullVersion`]s can be compared.
    ///
    /// For more detailed version comparison tests refer to the unit tests for [`Version`] and
    /// [`PackageRelease`].
    #[rstest]
    #[case::full_equal("1.0.0-1", "1.0.0-1", Ordering::Equal)]
    #[case::full_less("1.0.0-1", "1.0.0-2", Ordering::Less)]
    #[case::full_greater("1.0.0-2", "1.0.0-1", Ordering::Greater)]
    #[case::full_with_epoch_equal("1:1.0.0-1", "1:1.0.0-1", Ordering::Equal)]
    #[case::full_with_epoch_less("1.0.0-1", "1:1.0.0-1", Ordering::Less)]
    #[case::full_with_epoch_less("1:1.0.0-1", "2:1.0.0-1", Ordering::Less)]
    #[case::full_with_epoch_greater("1:1.0.0-1", "1.0.0-1", Ordering::Greater)]
    #[case::full_with_epoch_greater("2:1.0.0-1", "1:1.0.0-1", Ordering::Greater)]
    fn full_version_comparison(
        #[case] version_a: &str,
        #[case] version_b: &str,
        #[case] expected: Ordering,
    ) -> TestResult {
        let version_a = FullVersion::from_str(version_a)?;
        let version_b = FullVersion::from_str(version_b)?;

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

        assert_eq!(version_a.vercmp(&version_b), vercmp_result);

        // If we find the `vercmp` binary, also run the test against the actual binary.
        #[cfg(feature = "compatibility_tests")]
        {
            let output = std::process::Command::new("vercmp")
                .arg(version_a.to_string())
                .arg(version_b.to_string())
                .output()?;
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

        assert_eq!(version_b.vercmp(&version_a), reverse_vercmp_result);

        Ok(())
    }
}
