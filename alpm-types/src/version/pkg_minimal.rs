//! The [alpm-package-version] form _minimal_ and _minimal with epoch_.
//!
//! [alpm-package-version]: https://alpm.archlinux.page/specifications/alpm-package-version.7.html

use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
    str::FromStr,
};

use fluent_i18n::t;
use serde::{Deserialize, Serialize};
use winnow::{
    ModalResult,
    Parser,
    combinator::{cut_err, eof, opt, terminated},
    error::{StrContext, StrContextValue},
    token::take_till,
};

use crate::{Epoch, Error, PackageVersion, Version};
#[cfg(doc)]
use crate::{FullVersion, PackageRelease};

/// A package version without a [`PackageRelease`].
///
/// Tracks an optional [`Epoch`] and a [`PackageVersion`], but no [`PackageRelease`].
/// This reflects the _minimal_ and _minimal with epoch_ forms of [alpm-package-version].
///
/// # Notes
///
/// - If [`PackageRelease`] should be optional for your use-case, use [`Version`] instead.
/// - If [`PackageRelease`] should be mandatory for your use-case, use [`FullVersion`] instead.
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::MinimalVersion;
///
/// # fn main() -> testresult::TestResult {
/// // A minimal version.
/// let version = MinimalVersion::from_str("1.0.0")?;
///
/// // A minimal version with epoch.
/// let version = MinimalVersion::from_str("1:1.0.0")?;
/// # Ok(())
/// # }
/// ```
///
/// [alpm-package-version]: https://alpm.archlinux.page/specifications/alpm-package-version.7.html
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MinimalVersion {
    /// The version of the package
    pub pkgver: PackageVersion,
    /// The epoch of the package
    pub epoch: Option<Epoch>,
}

impl MinimalVersion {
    /// Creates a new [`MinimalVersion`].
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_types::{Epoch, MinimalVersion, PackageVersion};
    ///
    /// # fn main() -> testresult::TestResult {
    /// // A minimal version.
    /// let version = MinimalVersion::new(PackageVersion::new("1.0.0".to_string())?, None);
    ///
    /// // A minimal version with epoch.
    /// let version = MinimalVersion::new(
    ///     PackageVersion::new("1.0.0".to_string())?,
    ///     Some(Epoch::new(1.try_into()?)),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(pkgver: PackageVersion, epoch: Option<Epoch>) -> Self {
        Self { pkgver, epoch }
    }

    /// Compares `self` to another [`MinimalVersion`] and returns a number.
    ///
    /// - `1` if `self` is newer than `other`
    /// - `0` if `self` and `other` are equal
    /// - `-1` if `self` is older than `other`
    ///
    /// This output behavior is based on the behavior of the [vercmp] tool.
    ///
    /// Delegates to [`MinimalVersion::cmp`] for comparison.
    /// The rules and algorithms used for comparison are explained in more detail in
    /// [alpm-package-version] and [alpm-pkgver].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::MinimalVersion;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// assert_eq!(
    ///     MinimalVersion::from_str("1.0.0")?.vercmp(&MinimalVersion::from_str("0.1.0")?),
    ///     1
    /// );
    /// assert_eq!(
    ///     MinimalVersion::from_str("1.0.0")?.vercmp(&MinimalVersion::from_str("1.0.0")?),
    ///     0
    /// );
    /// assert_eq!(
    ///     MinimalVersion::from_str("0.1.0")?.vercmp(&MinimalVersion::from_str("1.0.0")?),
    ///     -1
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [alpm-package-version]: https://alpm.archlinux.page/specifications/alpm-package-version.7.html
    /// [alpm-pkgver]: https://alpm.archlinux.page/specifications/alpm-pkgver.7.html
    /// [vercmp]: https://man.archlinux.org/man/vercmp.8
    pub fn vercmp(&self, other: &MinimalVersion) -> i8 {
        match self.cmp(other) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }

    /// Recognizes a [`MinimalVersion`] in a string slice.
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
        let pkgver: PackageVersion = cut_err(PackageVersion::parser)
            .context(StrContext::Expected(StrContextValue::Description(
                "alpm-pkgver string",
            )))
            .parse_next(input)?;

        // Ensure that there are no trailing chars left.
        eof.context(StrContext::Expected(StrContextValue::Description(
            "end of full alpm-package-version string",
        )))
        .parse_next(input)?;

        Ok(Self { epoch, pkgver })
    }
}

impl Display for MinimalVersion {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        if let Some(epoch) = self.epoch {
            write!(fmt, "{epoch}:")?;
        }
        write!(fmt, "{}", self.pkgver)?;

        Ok(())
    }
}

impl FromStr for MinimalVersion {
    type Err = Error;
    /// Creates a new [`MinimalVersion`] from a string slice.
    ///
    /// Delegates to [`MinimalVersion::parser`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`Version::parser`] fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

impl Ord for MinimalVersion {
    /// Compares `self` to another [`MinimalVersion`].
    ///
    /// The comparison rules and algorithms are explained in more detail in [alpm-package-version]
    /// and [alpm-pkgver].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{cmp::Ordering, str::FromStr};
    ///
    /// use alpm_types::MinimalVersion;
    ///
    /// # fn main() -> testresult::TestResult {
    /// // Examples for "minimal"
    /// let version_a = MinimalVersion::from_str("0.1.0")?;
    /// let version_b = MinimalVersion::from_str("1.0.0")?;
    /// assert_eq!(version_a.cmp(&version_b), Ordering::Less);
    /// assert_eq!(version_b.cmp(&version_a), Ordering::Greater);
    ///
    /// let version_a = MinimalVersion::from_str("1.0.0")?;
    /// let version_b = MinimalVersion::from_str("1.0.0")?;
    /// assert_eq!(version_a.cmp(&version_b), Ordering::Equal);
    ///
    /// // Examples for "minimal with epoch"
    /// let version_a = MinimalVersion::from_str("1:1.0.0")?;
    /// let version_b = MinimalVersion::from_str("1.0.0")?;
    /// assert_eq!(version_a.cmp(&version_b), Ordering::Greater);
    /// assert_eq!(version_b.cmp(&version_a), Ordering::Less);
    ///
    /// let version_a = MinimalVersion::from_str("1:1.0.0")?;
    /// let version_b = MinimalVersion::from_str("1:1.0.0")?;
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

        self.pkgver.cmp(&other.pkgver)
    }
}

impl PartialOrd for MinimalVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl TryFrom<Version> for MinimalVersion {
    type Error = crate::Error;

    /// Creates a [`MinimalVersion`] from a [`Version`].
    ///
    /// # Errors
    ///
    /// Returns an error if `value.pkgrel` is [`None`].
    fn try_from(value: Version) -> Result<Self, Self::Error> {
        if value.pkgrel.is_some() {
            Err(Error::InvalidComponent {
                component: "pkgrel",
                context: t!("error-context-convert-full-to-minimal"),
            })
        } else {
            Ok(Self {
                pkgver: value.pkgver,
                epoch: value.epoch,
            })
        }
    }
}

impl TryFrom<&Version> for MinimalVersion {
    type Error = crate::Error;

    /// Creates a [`MinimalVersion`] from a [`Version`] reference.
    ///
    /// # Errors
    ///
    /// Returns an error if `value.pkgrel` is [`None`].
    fn try_from(value: &Version) -> Result<Self, Self::Error> {
        Self::try_from(value.clone())
    }
}

impl From<MinimalVersion> for Version {
    /// Creates a [`Version`] from a [`MinimalVersion`].
    fn from(value: MinimalVersion) -> Self {
        Self {
            pkgver: value.pkgver,
            pkgrel: None,
            epoch: value.epoch,
        }
    }
}

impl From<&MinimalVersion> for Version {
    /// Creates a [`Version`] from a [`MinimalVersion`] reference.
    fn from(value: &MinimalVersion) -> Self {
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

    /// Ensures that valid [`MinimalVersion`] strings are parsed successfully as expected.
    #[rstest]
    #[case::minimal_with_epoch(
        "1:foo",
        MinimalVersion {
            pkgver: PackageVersion::from_str("foo")?,
            epoch: Some(Epoch::from_str("1")?),
        },
    )]
    #[case::minimal(
        "foo",
        MinimalVersion {
            pkgver: PackageVersion::from_str("foo")?,
            epoch: None,
        }
    )]
    // yes, valid
    #[case::minimal_dot(
        ".",
        MinimalVersion {
            pkgver: PackageVersion::from_str(".")?,
            epoch: None,
            }
    )]
    fn minimal_version_from_str_succeeds(
        #[case] version: &str,
        #[case] expected: MinimalVersion,
    ) -> TestResult {
        init_logger();

        assert_eq!(
            MinimalVersion::from_str(version),
            Ok(expected),
            "Expected valid parsing for MinimalVersion {version}"
        );

        Ok(())
    }

    /// Ensures that invalid [`MinimalVersion`] strings lead to parse errors.
    #[rstest]
    #[case::two_pkgrel(
        "1:foo-1-1",
        "invalid pkgver character\nexpected an ASCII character, except for ':', '/', '-', '<', '>', '=', or any whitespace characters, alpm-pkgver string"
    )]
    #[case::two_epoch(
        "1:1:foo-1",
        "invalid pkgver character\nexpected an ASCII character, except for ':', '/', '-', '<', '>', '=', or any whitespace characters, alpm-pkgver string"
    )]
    #[case::empty_string(
        "",
        "invalid pkgver character\nexpected an ASCII character, except for ':', '/', '-', '<', '>', '=', or any whitespace characters, alpm-pkgver string"
    )]
    #[case::colon(
        ":",
        "invalid pkgver character\nexpected an ASCII character, except for ':', '/', '-', '<', '>', '=', or any whitespace characters, alpm-pkgver string"
    )]
    #[case::full_with_epoch(
        "1:1.0.0-1",
        "invalid pkgver character\nexpected an ASCII character, except for ':', '/', '-', '<', '>', '=', or any whitespace characters, alpm-pkgver string"
    )]
    #[case::full(
        "1.0.0-1",
        "invalid pkgver character\nexpected an ASCII character, except for ':', '/', '-', '<', '>', '=', or any whitespace characters, alpm-pkgver string"
    )]
    #[case::no_pkgrel_dash_end(
        "1.0.0-",
        "invalid pkgver character\nexpected an ASCII character, except for ':', '/', '-', '<', '>', '=', or any whitespace characters, alpm-pkgver string"
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
    fn minimal_version_from_str_parse_error(#[case] version: &str, #[case] err_snippet: &str) {
        init_logger();

        let Err(Error::ParseError(err_msg)) = MinimalVersion::from_str(version) else {
            panic!("parsing '{version}' as MinimalVersion did not fail as expected")
        };
        assert!(
            err_msg.contains(err_snippet),
            "Error:\n=====\n{err_msg}\n=====\nshould contain snippet:\n\n{err_snippet}"
        );
    }

    /// Ensures that [`MinimalVersion`] can be created from valid/compatible [`Version`] (and
    /// [`Version`] reference) and fails otherwise.
    #[rstest]
    #[case::minimal_with_epoch(Version::from_str("1:1.0.0")?, Ok(MinimalVersion::from_str("1:1.0.0")?))]
    #[case::minimal(Version::from_str("1.0.0")?, Ok(MinimalVersion::from_str("1.0.0")?))]
    #[case::full_with_epoch(Version::from_str("1:1.0.0-1")?, Err(Error::InvalidComponent{component: "pkgrel", context: t!("error-context-convert-full-to-minimal")}))]
    #[case::full(Version::from_str("1.0.0-1")?, Err(Error::InvalidComponent{component: "pkgrel", context: t!("error-context-convert-full-to-minimal")}))]
    fn minimal_version_try_from_version(
        #[case] version: Version,
        #[case] expected: Result<MinimalVersion, Error>,
    ) -> TestResult {
        assert_eq!(MinimalVersion::try_from(&version), expected);
        Ok(())
    }

    /// Ensures that [`Version`] can be created from [`MinimalVersion`] (and [`MinimalVersion`]
    /// reference).
    #[rstest]
    #[case::minimal_with_epoch(Version::from_str("1:1.0.0")?, MinimalVersion::from_str("1:1.0.0")?)]
    #[case::minimal(Version::from_str("1.0.0")?, MinimalVersion::from_str("1.0.0")?)]
    fn version_from_minimal_version(
        #[case] version: Version,
        #[case] full_version: MinimalVersion,
    ) -> TestResult {
        assert_eq!(Version::from(&full_version), version);
        Ok(())
    }

    /// Ensures that [`MinimalVersion`] is properly serialized back to its string representation.
    #[rstest]
    #[case::with_epoch("1:1.0.0")]
    #[case::plain("1.0.0")]
    fn minimal_version_to_string(#[case] input: &str) -> TestResult {
        assert_eq!(format!("{}", MinimalVersion::from_str(input)?), input);
        Ok(())
    }

    /// Ensures that [`MinimalVersion`]s can be compared.
    ///
    /// For more detailed version comparison tests refer to the unit tests for [`Version`] and
    /// [`PackageRelease`].
    #[rstest]
    #[case::minimal_equal("1.0.0", "1.0.0", Ordering::Equal)]
    #[case::minimal_less("1.0.0", "2.0.0", Ordering::Less)]
    #[case::minimal_greater("2.0.0", "1.0.0", Ordering::Greater)]
    #[case::minimal_with_epoch_equal("1:1.0.0", "1:1.0.0", Ordering::Equal)]
    #[case::minimal_with_epoch_less("1.0.0", "1:1.0.0", Ordering::Less)]
    #[case::minimal_with_epoch_less("1:1.0.0", "2:1.0.0", Ordering::Less)]
    #[case::minimal_with_epoch_greater("1:1.0.0", "1.0.0", Ordering::Greater)]
    #[case::minimal_with_epoch_greater("2:1.0.0", "1:1.0.0", Ordering::Greater)]
    fn minimal_version_comparison(
        #[case] version_a: &str,
        #[case] version_b: &str,
        #[case] expected: Ordering,
    ) -> TestResult {
        let version_a = MinimalVersion::from_str(version_a)?;
        let version_b = MinimalVersion::from_str(version_b)?;

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
