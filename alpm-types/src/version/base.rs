//! The base components for [alpm-package-version].
//!
//! An [alpm-package-version] is defined by the [alpm-epoch], [alpm-pkgver] and [alpm-pkgrel]
//! components.
//!
//! [alpm-package-version]: https://alpm.archlinux.page/specifications/alpm-package-version.7.html
//! [alpm-epoch]: https://alpm.archlinux.page/specifications/alpm-epoch.7.html
//! [alpm-pkgver]: https://alpm.archlinux.page/specifications/alpm-pkgver.7.html
//! [alpm-pkgrel]: https://alpm.archlinux.page/specifications/alpm-pkgrel.7.html

use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
    num::NonZeroUsize,
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use winnow::{
    ModalResult,
    Parser,
    ascii::{dec_uint, digit1},
    combinator::{Repeat, cut_err, eof, opt, preceded, repeat, seq, terminated},
    error::{StrContext, StrContextValue},
    token::one_of,
};

#[cfg(doc)]
use crate::Version;
use crate::{Error, VersionSegments};

/// An epoch of a package
///
/// Epoch is used to indicate the downgrade of a package and is prepended to a version, delimited by
/// a `":"` (e.g. `1:` is added to `0.10.0-1` to form `1:0.10.0-1` which then orders newer than
/// `1.0.0-1`).
/// See [alpm-epoch] for details on the format.
///
/// An Epoch wraps a usize that is guaranteed to be greater than `0`.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::Epoch;
///
/// assert!(Epoch::from_str("1").is_ok());
/// assert!(Epoch::from_str("0").is_err());
/// ```
///
/// [alpm-epoch]: https://alpm.archlinux.page/specifications/alpm-epoch.7.html
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Epoch(pub NonZeroUsize);

impl Epoch {
    /// Create a new Epoch
    pub fn new(epoch: NonZeroUsize) -> Self {
        Epoch(epoch)
    }

    /// Recognizes an [`Epoch`] in a string slice.
    ///
    /// Consumes all of its input.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is not a valid _alpm_epoch_.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        terminated(dec_uint, eof)
            .verify_map(NonZeroUsize::new)
            .context(StrContext::Label("package epoch"))
            .context(StrContext::Expected(StrContextValue::Description(
                "positive non-zero decimal integer",
            )))
            .map(Self)
            .parse_next(input)
    }
}

impl FromStr for Epoch {
    type Err = Error;
    /// Create an Epoch from a string and return it in a Result
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

impl Display for Epoch {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

/// The release version of a package.
///
/// A [`PackageRelease`] wraps a [`usize`] for its `major` version and an optional [`usize`] for its
/// `minor` version.
///
/// [`PackageRelease`] is used to indicate the build version of a package.
/// It is mostly useful in conjunction with a [`PackageVersion`] (see [`Version`]).
/// Refer to [alpm-pkgrel] for more details on the format.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::PackageRelease;
///
/// assert!(PackageRelease::from_str("1").is_ok());
/// assert!(PackageRelease::from_str("1.1").is_ok());
/// assert!(PackageRelease::from_str("0").is_ok());
/// assert!(PackageRelease::from_str("a").is_err());
/// assert!(PackageRelease::from_str("1.a").is_err());
/// ```
///
/// [alpm-pkgrel]: https://alpm.archlinux.page/specifications/alpm-pkgrel.7.html
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PackageRelease {
    /// The major version of this package release.
    pub major: usize,
    /// The optional minor version of this package release.
    pub minor: Option<usize>,
}

impl PackageRelease {
    /// Creates a new [`PackageRelease`] from a `major` and optional `minor` integer version.
    ///
    /// ## Examples
    /// ```
    /// use alpm_types::PackageRelease;
    ///
    /// # fn main() {
    /// let release = PackageRelease::new(1, Some(2));
    /// assert_eq!(format!("{release}"), "1.2");
    /// # }
    /// ```
    pub fn new(major: usize, minor: Option<usize>) -> Self {
        PackageRelease { major, minor }
    }

    /// Recognizes a [`PackageRelease`] in a string slice.
    ///
    /// Consumes all of its input.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` does not contain a valid [`PackageRelease`].
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        seq!(Self {
            major: digit1.try_map(FromStr::from_str)
                .context(StrContext::Label("package release"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "positive decimal integer",
                ))),
            minor: opt(preceded('.', cut_err(digit1.try_map(FromStr::from_str))))
                .context(StrContext::Label("package release"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "single '.' followed by positive decimal integer",
                ))),
            _: eof.context(StrContext::Expected(StrContextValue::Description(
                "end of package release value",
            ))),
        })
        .parse_next(input)
    }
}

impl FromStr for PackageRelease {
    type Err = Error;
    /// Creates a [`PackageRelease`] from a string slice.
    ///
    /// Delegates to [`PackageRelease::parser`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`PackageRelease::parser`] fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

impl Display for PackageRelease {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.major)?;
        if let Some(minor) = self.minor {
            write!(fmt, ".{minor}")?;
        }
        Ok(())
    }
}

impl PartialOrd for PackageRelease {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PackageRelease {
    fn cmp(&self, other: &Self) -> Ordering {
        let major_order = self.major.cmp(&other.major);
        if major_order != Ordering::Equal {
            return major_order;
        }

        match (self.minor, other.minor) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(minor), Some(other_minor)) => minor.cmp(&other_minor),
        }
    }
}

/// A pkgver of a package
///
/// PackageVersion is used to denote the upstream version of a package.
///
/// A PackageVersion wraps a `String`, which is guaranteed to only contain ASCII characters,
/// excluding the ':', '/', '-', '<', '>', '=', or any whitespace characters and must be at least
/// one character long.
///
/// NOTE: This implementation of PackageVersion is stricter than that of libalpm/pacman. It does not
/// allow empty strings `""`.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::PackageVersion;
///
/// assert!(PackageVersion::new("1".to_string()).is_ok());
/// assert!(PackageVersion::new("1.1".to_string()).is_ok());
/// assert!(PackageVersion::new("foo".to_string()).is_ok());
/// assert!(PackageVersion::new("0".to_string()).is_ok());
/// assert!(PackageVersion::new(".0.1".to_string()).is_ok());
/// assert!(PackageVersion::new("=1.0".to_string()).is_err());
/// assert!(PackageVersion::new("1<0".to_string()).is_err());
/// ```
#[derive(Clone, Debug, Deserialize, Eq, Serialize)]
pub struct PackageVersion(pub(crate) String);

impl PackageVersion {
    /// Create a new PackageVersion from a string and return it in a Result
    pub fn new(pkgver: String) -> Result<Self, Error> {
        PackageVersion::from_str(pkgver.as_str())
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &str {
        &self.0
    }

    /// Return an iterator over all segments of this version.
    pub fn segments(&self) -> VersionSegments<'_> {
        VersionSegments::new(&self.0)
    }

    /// Recognizes a [`PackageVersion`] in a string slice.
    ///
    /// Consumes all of its input.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is not a valid _alpm-pkgver_.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        // General rule for all characters:
        // only ASCII except for ':', '/', '-', '<', '>', '=' or any whitespace
        let allowed = |c: char| {
            c.is_ascii() && ![':', '/', '-', '<', '>', '='].contains(&c) && !c.is_whitespace()
        };

        // note the empty tuple collection to avoid allocation
        let pkgver: Repeat<_, _, _, (), _> = repeat(1.., one_of(allowed));

        (
            pkgver,
            eof
        )
            .context(StrContext::Label("pkgver character"))
            .context(StrContext::Expected(StrContextValue::Description(
                "an ASCII character, except for ':', '/', '-', '<', '>', '=', or any whitespace characters",
            )))
            .take()
            .map(|s: &str| Self(s.to_string()))
            .parse_next(input)
    }
}

impl FromStr for PackageVersion {
    type Err = Error;
    /// Create a PackageVersion from a string and return it in a Result
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

impl Display for PackageVersion {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("1", Ok(Epoch(NonZeroUsize::new(1).unwrap())))]
    fn epoch(#[case] version: &str, #[case] result: Result<Epoch, Error>) {
        assert_eq!(result, Epoch::from_str(version));
    }

    #[rstest]
    #[case("0", "expected positive non-zero decimal integer")]
    #[case("-0", "expected positive non-zero decimal integer")]
    #[case("z", "expected positive non-zero decimal integer")]
    fn epoch_parse_failure(#[case] input: &str, #[case] err_snippet: &str) {
        let Err(Error::ParseError(err_msg)) = Epoch::from_str(input) else {
            panic!("'{input}' erroneously parsed as Epoch")
        };
        assert!(
            err_msg.contains(err_snippet),
            "Error:\n=====\n{err_msg}\n=====\nshould contain snippet:\n\n{err_snippet}"
        );
    }

    /// Make sure that we can parse valid **pkgver** strings.
    #[rstest]
    #[case("foo")]
    #[case("1.0.0")]
    // sadly, this is valid
    #[case(".xd")]
    fn valid_pkgver(#[case] pkgver: &str) {
        let parsed = PackageVersion::new(pkgver.to_string());
        assert!(parsed.is_ok(), "Expected pkgver {pkgver} to be valid.");
        assert_eq!(
            parsed.as_ref().unwrap().to_string(),
            pkgver,
            "Expected parsed PackageVersion representation '{}' to be identical to input '{}'",
            parsed.unwrap(),
            pkgver
        );
    }

    /// Ensure that invalid **pkgver**s are throwing errors.
    #[rstest]
    #[case("1:foo", "invalid pkgver character")]
    #[case("foo-1", "invalid pkgver character")]
    #[case("foo/1", "invalid pkgver character")]
    // ß is not ASCII
    #[case("ß", "invalid pkgver character")]
    #[case("1.ß", "invalid pkgver character")]
    #[case("", "invalid pkgver character")]
    fn invalid_pkgver(#[case] pkgver: &str, #[case] err_snippet: &str) {
        let Err(Error::ParseError(err_msg)) = PackageVersion::new(pkgver.to_string()) else {
            panic!("Expected pkgver {pkgver} to be invalid.")
        };
        assert!(
            err_msg.contains(err_snippet),
            "Error:\n=====\n{err_msg}\n=====\nshould contain snippet:\n\n{err_snippet}"
        );
    }

    /// Make sure that we can parse valid **pkgrel** strings.
    #[rstest]
    #[case("0")]
    #[case("1")]
    #[case("10")]
    #[case("1.0")]
    #[case("10.5")]
    #[case("0.1")]
    fn valid_pkgrel(#[case] pkgrel: &str) {
        let parsed = PackageRelease::from_str(pkgrel);
        assert!(parsed.is_ok(), "Expected pkgrel {pkgrel} to be valid.");
        assert_eq!(
            parsed.as_ref().unwrap().to_string(),
            pkgrel,
            "Expected parsed PackageRelease representation '{}' to be identical to input '{}'",
            parsed.unwrap(),
            pkgrel
        );
    }

    /// Ensure that invalid **pkgrel**s are throwing errors.
    #[rstest]
    #[case(".1", "expected positive decimal integer")]
    #[case("1.", "expected single '.' followed by positive decimal integer")]
    #[case("1..1", "expected single '.' followed by positive decimal integer")]
    #[case("-1", "expected positive decimal integer")]
    #[case("a", "expected positive decimal integer")]
    #[case("1.a", "expected single '.' followed by positive decimal integer")]
    #[case("1.0.0", "expected end of package release")]
    #[case("", "expected positive decimal integer")]
    fn invalid_pkgrel(#[case] pkgrel: &str, #[case] err_snippet: &str) {
        let Err(Error::ParseError(err_msg)) = PackageRelease::from_str(pkgrel) else {
            panic!("'{pkgrel}' erroneously parsed as PackageRelease")
        };
        assert!(
            err_msg.contains(err_snippet),
            "Error:\n=====\n{err_msg}\n=====\nshould contain snippet:\n\n{err_snippet}"
        );
    }

    /// Test that pkgrel ordering works as intended
    #[rstest]
    #[case("1", "1.0", Ordering::Less)]
    #[case("1.0", "2", Ordering::Less)]
    #[case("1", "1.1", Ordering::Less)]
    #[case("1.0", "1.1", Ordering::Less)]
    #[case("0", "1.1", Ordering::Less)]
    #[case("1", "11", Ordering::Less)]
    #[case("1", "1", Ordering::Equal)]
    #[case("1.2", "1.2", Ordering::Equal)]
    #[case("2.0", "2.0", Ordering::Equal)]
    #[case("2", "1.0", Ordering::Greater)]
    #[case("1.1", "1", Ordering::Greater)]
    #[case("1.1", "1.0", Ordering::Greater)]
    #[case("1.1", "0", Ordering::Greater)]
    #[case("11", "1", Ordering::Greater)]
    fn pkgrel_cmp(#[case] first: &str, #[case] second: &str, #[case] order: Ordering) {
        let first = PackageRelease::from_str(first).unwrap();
        let second = PackageRelease::from_str(second).unwrap();
        assert_eq!(
            first.cmp(&second),
            order,
            "{first} should be {order:?} to {second}"
        );
    }
}
