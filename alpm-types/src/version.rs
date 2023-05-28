// SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
// SPDX-License-Identifier: LGPL-3.0-or-later
use std::cmp::Ordering;
use std::fmt::Display;
use std::fmt::Formatter;
use std::num::NonZeroUsize;
use std::str::FromStr;

use semver::Version as SemverVersion;

use crate::regex_once;
use crate::Error;

/// An epoch of a package
///
/// Epoch is used to indicate the downgrade of a package and is prepended to a version, delimited by a `":"` (e.g. `1:`
/// is added to `0.10.0-1` to form `1:0.10.0-1` which then orders newer than `1.0.0-1`).
///
/// An Epoch wraps a usize that is guaranteed to be greater than `0`.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
/// use alpm_types::Epoch;
///
/// assert!(Epoch::new("1".to_string()).is_ok());
/// assert!(Epoch::new("0".to_string()).is_err());
/// ```
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Epoch(NonZeroUsize);

impl Epoch {
    /// Create a new Epoch from a string and return it in a Result
    pub fn new(epoch: String) -> Result<Self, Error> {
        match epoch.parse() {
            Ok(epoch) => Ok(Epoch(epoch)),
            Err(_) => Err(Error::InvalidEpoch(epoch)),
        }
    }

    // Return a reference to the inner type
    pub fn inner(&self) -> NonZeroUsize {
        self.0
    }
}

impl FromStr for Epoch {
    type Err = Error;
    /// Create an Epoch from a string and return it in a Result
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Epoch::new(input.to_string())
    }
}

impl Display for Epoch {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

/// A pkgrel of a package
///
/// Pkgrel is used to indicate the build version of a package and is appended to a version, delimited by a `"-"` (e.g.
/// `-2` is added to `1.0.0` to form `1.0.0-2` which then orders newer than `1.0.0-1`).
///
/// A Pkgrel wraps a String which is guaranteed to not start with a `"0"`, to contain only numeric characters
/// (optionally delimited by a single `"."`, which must be followed by at least one non-`"0"` numeric character).
///
/// ## Examples
/// ```
/// use std::str::FromStr;
/// use alpm_types::Pkgrel;
///
/// assert!(Pkgrel::new("1".to_string()).is_ok());
/// assert!(Pkgrel::new("1.1".to_string()).is_ok());
/// assert!(Pkgrel::new("0".to_string()).is_err());
/// assert!(Pkgrel::new("0.1".to_string()).is_err());
/// assert!(Pkgrel::new("1.0".to_string()).is_err());
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Pkgrel(String);

impl Pkgrel {
    /// Create a new Pkgrel from a string and return it in a Result
    pub fn new(pkgrel: String) -> Result<Self, Error> {
        if regex_once!(r"^[1-9]+[0-9]*(|[.]{1}[1-9]+[0-9]*)$").is_match(pkgrel.as_str()) {
            Ok(Pkgrel(pkgrel))
        } else {
            Err(Error::InvalidPkgrel(pkgrel))
        }
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &str {
        &self.0
    }
}

impl FromStr for Pkgrel {
    type Err = Error;
    /// Create a Pkgrel from a string and return it in a Result
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Pkgrel::new(input.to_string())
    }
}

impl Display for Pkgrel {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

/// A pkgver of a package
///
/// Pkgver is used to denote the upstream version of a package.
///
/// A Pkgver wraps a `String`, which is guaranteed to only contain alphanumeric characters, `"_"`, `"+"` or `"."`, but
/// to not start with a `"_"`, a `"+"` or a `"."` character and to be at least one char long.
///
/// NOTE: This implementation of Pkgver is stricter than that of libalpm/pacman. It does not allow empty strings `""`,
/// or chars that are not in the allowed set, or `"."` as the first character.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
/// use alpm_types::Pkgver;
///
/// assert!(Pkgver::new("1".to_string()).is_ok());
/// assert!(Pkgver::new("1.1".to_string()).is_ok());
/// assert!(Pkgver::new("foo".to_string()).is_ok());
/// assert!(Pkgver::new("0".to_string()).is_ok());
/// assert!(Pkgver::new(".0.1".to_string()).is_err());
/// assert!(Pkgver::new("_1.0".to_string()).is_err());
/// assert!(Pkgver::new("+1.0".to_string()).is_err());
/// ```
#[derive(Clone, Debug, Eq)]
pub struct Pkgver(String);

impl Pkgver {
    /// Create a new Pkgver from a string and return it in a Result
    pub fn new(pkgver: String) -> Result<Self, Error> {
        if regex_once!(r"^([^_+.][[:alnum:]_+.]*)$").is_match(pkgver.as_str()) {
            Ok(Pkgver(pkgver))
        } else {
            Err(Error::InvalidPkgver(pkgver))
        }
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &str {
        &self.0
    }
}

impl FromStr for Pkgver {
    type Err = Error;
    /// Create a Pkgver from a string and return it in a Result
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Pkgver::new(input.to_string())
    }
}

impl Display for Pkgver {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

impl Ord for Pkgver {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_inner = self.inner();
        let other_inner = other.inner();

        // easy comparison to see if versions are identical
        if self_inner == other_inner {
            return Ordering::Equal;
        }

        // Strings for temporarily holding leftovers when comparing
        let mut self_leftover;
        let mut other_leftover;
        // Indices used as left hand pointers for section starts when comparing self and other
        let mut self_left_index = 0;
        let mut other_left_index = 0;
        // Indices used as right hand pointers for section ends when comparing self and other
        let mut self_right_index = 0;
        let mut other_right_index = 0;

        // loop through each version segment of a and b and compare them
        while self_left_index < self_inner.len() && other_left_index < other_inner.len() {
            // set self_left_index to the location of the last alphanumeric char in one
            while self_left_index < self_inner.len()
                && !self_inner
                    .chars()
                    .nth(self_left_index)
                    .unwrap()
                    .is_alphanumeric()
            {
                self_left_index += 1;
            }
            // set other_left_index to the location of the last alphanumeric char in two
            while other_left_index < other_inner.len()
                && !other
                    .inner()
                    .chars()
                    .nth(other_left_index)
                    .unwrap()
                    .is_alphanumeric()
            {
                other_left_index += 1;
            }

            // If we ran to the end of either, we are finished with the loop
            if self_left_index >= self_inner.len() || other_left_index >= other_inner.len() {
                break;
            }

            // If the separator lengths were different, we are finished
            if (self_left_index - self_right_index) != (other_left_index - other_right_index) {
                return if (self_left_index - self_right_index)
                    < (other_left_index - other_right_index)
                {
                    Ordering::Less
                } else {
                    Ordering::Greater
                };
            }

            // adjust left side pointer to current segment start
            self_right_index = self_left_index;
            other_right_index = other_left_index;
            self_leftover = if let Some(leftover) = self_inner.get(self_left_index..) {
                leftover.to_string()
            } else {
                "".to_string()
            };
            other_leftover = if let Some(leftover) = other_inner.get(other_left_index..) {
                leftover.to_string()
            } else {
                "".to_string()
            };

            // grab first completely alpha or completely numeric segment leave one and two pointing to the start of the
            // alpha or numeric segment and walk self_right_index and other_right_index to end of segment
            let isnum = if !self_leftover.is_empty()
                && self_leftover.chars().next().unwrap().is_numeric()
            {
                self_right_index += self_leftover.chars().take_while(|x| x.is_numeric()).count();
                other_right_index += other_leftover
                    .chars()
                    .take_while(|x| x.is_numeric())
                    .count();
                true
            } else {
                self_right_index += self_leftover
                    .chars()
                    .take_while(|x| x.is_alphabetic())
                    .count();
                other_right_index += other_leftover
                    .chars()
                    .take_while(|x| x.is_alphabetic())
                    .count();
                false
            };

            // adjust current segment end with the updated right side pointer
            self_leftover =
                if let Some(leftover) = self_inner.get(self_left_index..self_right_index) {
                    leftover.to_string()
                } else {
                    "".to_string()
                };
            other_leftover =
                if let Some(leftover) = other_inner.get(other_left_index..other_right_index) {
                    leftover.to_string()
                } else {
                    "".to_string()
                };

            // take care of the case where the two version segments are different types: one numeric, the other alpha
            // (i.e. empty) numeric segments are always newer than alpha segments
            if other_leftover.is_empty() {
                return if isnum {
                    Ordering::Greater
                } else {
                    Ordering::Less
                };
            }

            if isnum {
                // throw away any leading zeros - it's a number, right?
                self_leftover = self_leftover.trim_start_matches('0').to_string();
                other_leftover = other_leftover.trim_start_matches('0').to_string();

                // whichever number has more digits wins (discard leading zeros)
                match (self_leftover.len(), other_leftover.len()) {
                    (one_len, two_len) if one_len > two_len => return Ordering::Greater,
                    (one_len, two_len) if one_len < two_len => return Ordering::Less,
                    (_, _) => {}
                }
            }

            // strcmp will return which one is greater - even if the two segments are alpha or if they are numeric.
            // don't return if they are equal because there might be more segments to compare
            if self_leftover.cmp(&other_leftover).is_ne() {
                return self_leftover.cmp(&other_leftover);
            }

            // advance left side pointer to current right side pointer
            self_left_index = self_right_index;
            other_left_index = other_right_index;
        }

        // set leftover using the left side pointer once the segment loop finished
        self_leftover = if let Some(leftover) = self_inner.get(self_left_index..) {
            leftover.to_string()
        } else {
            "".to_string()
        };
        other_leftover = if let Some(leftover) = other_inner.get(other_left_index..) {
            leftover.to_string()
        } else {
            "".to_string()
        };

        // this catches the case where all numeric and alpha segments have compared identically but the segment
        // separating characters were different
        if self_leftover.is_empty() && other_leftover.is_empty() {
            return Ordering::Equal;
        }

        // the final showdown. we never want a remaining alpha string to beat an empty string. the logic is a bit weird,
        // but:
        // - if one is empty and two is not an alpha, two is newer.
        // - if one is an alpha, two is newer.
        // - otherwise one is newer.
        if (self_leftover.is_empty() && !other_leftover.chars().next().unwrap().is_alphabetic())
            || (!self_leftover.is_empty() && self_leftover.chars().next().unwrap().is_alphabetic())
        {
            return Ordering::Less;
        }

        Ordering::Greater
    }
}

impl PartialOrd for Pkgver {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Pkgver {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

/// The schema version of a type
///
/// A `SchemaVersion` wraps a `semver::Version`, which means that the tracked version should follow [semver](https://semver.org).
/// However, for backwards compatibility reasons it is possible to initialize a `SchemaVersion` using a non-semver
/// compatible string, *if* it can be parsed to a single `u64` (e.g. `"1"`).
///
/// ## Examples
/// ```
/// use std::str::FromStr;
/// use alpm_types::SchemaVersion;
///
/// // create SchemaVersion from str
/// let version_one = SchemaVersion::from_str("1.0.0").unwrap();
/// let version_also_one = SchemaVersion::new("1").unwrap();
/// assert_eq!(version_one, version_also_one);
///
/// // format as String
/// assert_eq!("1.0.0", format!("{}", version_one));
/// assert_eq!("1.0.0", format!("{}", version_also_one));
/// ```
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SchemaVersion(SemverVersion);

impl SchemaVersion {
    /// Create a new SchemaVersion from a string
    ///
    /// When providing a non-semver string with only a number (i.e. no minor or patch version), the number is treated as
    /// the major version (e.g. `"23"` -> `"23.0.0"`).
    pub fn new(version: &str) -> Result<SchemaVersion, Error> {
        if !version.contains('.') {
            match version.parse() {
                Ok(major) => Ok(SchemaVersion(SemverVersion::new(major, 0, 0))),
                Err(_) => Err(Error::InvalidVersion(version.to_string())),
            }
        } else {
            match SemverVersion::parse(version) {
                Ok(version) => Ok(SchemaVersion(version)),
                Err(_) => Err(Error::InvalidVersion(version.to_string())),
            }
        }
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &SemverVersion {
        &self.0
    }
}

impl FromStr for SchemaVersion {
    type Err = Error;
    /// Create a SchemaVersion from a string
    fn from_str(input: &str) -> Result<SchemaVersion, Self::Err> {
        SchemaVersion::new(input)
    }
}

impl Display for SchemaVersion {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

/// A version of a package
///
/// A `Version` tracks an optional `Epoch`, a `Pkgver` and an optional `Pkgrel`.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
/// use alpm_types::{Epoch, Pkgrel, Pkgver, Version};
///
/// let version = Version::new("1:1-1").unwrap();
/// assert_eq!(version.epoch(), Some(&Epoch::new("1".to_string()).unwrap()));
/// assert_eq!(version.pkgver(), &Pkgver::new("1".to_string()).unwrap());
/// assert_eq!(version.pkgrel(), Some(&Pkgrel::new("1".to_string()).unwrap()));
/// ```
#[derive(Debug, Eq)]
pub struct Version {
    pkgver: Pkgver,
    epoch: Option<Epoch>,
    pkgrel: Option<Pkgrel>,
}

impl Version {
    /// Create a new Version from a string and return it in a Result
    pub fn new(version: &str) -> Result<Self, Error> {
        let mut epoch_split = vec![];
        let mut pkgrel_split = vec![];
        for (i, char) in version.chars().enumerate() {
            match char {
                ':' => epoch_split.push(i),
                '-' => pkgrel_split.push(i),
                _ => {}
            }
        }

        Ok(Version {
            pkgver: match (epoch_split.len(), pkgrel_split.len()) {
                // pkgrel occurs before epoch
                (1, 1) if epoch_split[0] > pkgrel_split[0] => {
                    return Err(Error::InvalidVersion(version.to_string()))
                }
                // pkgver in between epoch and pkgrel
                (1, 1) => Pkgver::new(version[epoch_split[0] + 1..pkgrel_split[0]].to_string())?,
                // pkgver before pkgrel
                (0, 1) => Pkgver::new(version[..pkgrel_split[0]].to_string())?,
                // only pkgver
                (0, 0) => Pkgver::new(version.to_string())?,
                // pkgver after epoch
                (1, 0) => Pkgver::new(version[epoch_split[0] + 1..].to_string())?,
                // more than one epoch or pkgrel
                (_, _) => return Err(Error::InvalidVersion(version.to_string())),
            },
            epoch: if epoch_split.len() == 1 {
                Some(Epoch::new(version[..epoch_split[0]].to_string())?)
            } else {
                None
            },
            pkgrel: if pkgrel_split.len() == 1 {
                Some(Pkgrel::new(version[pkgrel_split[0] + 1..].to_string())?)
            } else {
                None
            },
        })
    }

    /// Return the optional reference to the Epoch of the Version
    pub fn epoch(&self) -> Option<&Epoch> {
        self.epoch.as_ref()
    }

    /// Return a reference to Pkgver of the Version
    pub fn pkgver(&self) -> &Pkgver {
        &self.pkgver
    }

    /// Return the optional reference to the Pkgrel of the Version
    pub fn pkgrel(&self) -> Option<&Pkgrel> {
        self.pkgrel.as_ref()
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
    /// use alpm_types::Version;
    ///
    /// assert_eq!(Version::vercmp(&Version::new("1.0.0").unwrap(), &Version::new("0.1.0").unwrap()), 1);
    /// assert_eq!(Version::vercmp(&Version::new("1.0.0").unwrap(), &Version::new("1.0.0").unwrap()), 0);
    /// assert_eq!(Version::vercmp(&Version::new("0.1.0").unwrap(), &Version::new("1.0.0").unwrap()), -1);
    /// ```
    pub fn vercmp(a: &Version, b: &Version) -> i8 {
        match a.cmp(b) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }
}

impl FromStr for Version {
    type Err = Error;
    /// Create a SchemaVersion from a string
    fn from_str(input: &str) -> Result<Version, Self::Err> {
        Version::new(input)
    }
}

impl Display for Version {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{}{}{}",
            if let Some(epoch) = self.epoch() {
                format!("{}:", epoch)
            } else {
                "".to_string()
            },
            self.pkgver(),
            if let Some(pkgrel) = self.pkgrel() {
                format!("-{}", pkgrel)
            } else {
                "".to_string()
            }
        )
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.epoch, other.epoch) {
            (Some(self_epoch), Some(other_epoch)) if self_epoch.cmp(&other_epoch).is_ne() => {
                return self_epoch.cmp(&other_epoch)
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

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.epoch == other.epoch
            && self.pkgver.cmp(&other.pkgver).is_eq()
            && self.pkgrel == other.pkgrel
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("1.0.0", Ok(SchemaVersion(SemverVersion::new(1, 0, 0))))]
    #[case("1", Ok(SchemaVersion(SemverVersion::new(1, 0, 0))))]
    #[case("-1.0.0", Err(Error::InvalidVersion("-1.0.0".to_string())))]
    fn schema_version(#[case] version: &str, #[case] result: Result<SchemaVersion, Error>) {
        assert_eq!(result, SchemaVersion::new(version))
    }

    #[rstest]
    #[case(
        SchemaVersion(SemverVersion::new(1, 0, 0)),
        SchemaVersion(SemverVersion::new(0, 1, 0))
    )]
    fn compare_schema_version(#[case] version_a: SchemaVersion, #[case] version_b: SchemaVersion) {
        assert!(version_a > version_b);
    }

    #[rstest]
    #[case("foo", Ok(Version{epoch: None, pkgver: Pkgver::new("foo".to_string()).unwrap(), pkgrel: None}))]
    #[case(
        "1:foo-1",
        Ok(Version{
            pkgver: Pkgver::new("foo".to_string()).unwrap(),
            epoch: Some(Epoch::new("1".to_string()).unwrap()),
            pkgrel: Some(Pkgrel::new("1".to_string()).unwrap()),
        }),
    )]
    #[case(
        "1:foo",
        Ok(Version{
            pkgver: Pkgver::new("foo".to_string()).unwrap(),
            epoch: Some(Epoch::new("1".to_string()).unwrap()),
            pkgrel: None,
        }),
    )]
    #[case(
        "foo-1",
        Ok(Version{
            pkgver: Pkgver::new("foo".to_string()).unwrap(),
            epoch: None,
            pkgrel: Some(Pkgrel::new("1".to_string()).unwrap())
        })
    )]
    #[case("-1foo:1", Err(Error::InvalidVersion("-1foo:1".to_string())))]
    #[case("1-foo:1", Err(Error::InvalidVersion("1-foo:1".to_string())))]
    #[case("1:1:foo-1", Err(Error::InvalidVersion("1:1:foo-1".to_string())))]
    #[case("1:foo-1-1", Err(Error::InvalidVersion("1:foo-1-1".to_string())))]
    #[case("", Err(Error::InvalidPkgver("".to_string())))]
    #[case(":", Err(Error::InvalidPkgver("".to_string())))]
    #[case(".", Err(Error::InvalidPkgver(".".to_string())))]
    fn version_from_string(#[case] version: &str, #[case] result: Result<Version, Error>) {
        if result.is_ok() {
            assert_eq!(result.as_ref().unwrap(), &Version::new(version).unwrap())
        } else {
            assert_eq!(
                result.as_ref().expect_err("Should be an Err"),
                &Version::new(version).expect_err("Should be an Err")
            )
        }
    }

    #[rstest]
    #[case("1".to_string(), Ok(Epoch(NonZeroUsize::new(1).unwrap())))]
    #[case("0".to_string(), Err(Error::InvalidEpoch("0".to_string())))]
    #[case("-0".to_string(), Err(Error::InvalidEpoch("-0".to_string())))]
    #[case("z".to_string(), Err(Error::InvalidEpoch("z".to_string())))]
    fn epoch(#[case] version: String, #[case] result: Result<Epoch, Error>) {
        assert_eq!(result, Epoch::new(version));
    }

    #[rstest]
    #[case("foo".to_string(), Ok(Pkgver::new("foo".to_string()).unwrap()))]
    #[case("1.0.0".to_string(), Ok(Pkgver::new("1.0.0".to_string()).unwrap()))]
    #[case("1:foo".to_string(), Err(Error::InvalidPkgver("1:foo".to_string())))]
    #[case("foo-1".to_string(), Err(Error::InvalidPkgver("foo-1".to_string())))]
    #[case("foo,1".to_string(), Err(Error::InvalidPkgver("foo,1".to_string())))]
    #[case(".foo".to_string(), Err(Error::InvalidPkgver(".foo".to_string())))]
    #[case("_foo".to_string(), Err(Error::InvalidPkgver("_foo".to_string())))]
    fn pkgver(#[case] version: String, #[case] result: Result<Pkgver, Error>) {
        assert_eq!(result, Pkgver::new(version));
    }

    #[rstest]
    #[case("1".to_string(), Ok(Pkgrel::new("1".to_string()).unwrap()))]
    #[case("1.1".to_string(), Ok(Pkgrel::new("1.1".to_string()).unwrap()))]
    #[case("0.1".to_string(), Err(Error::InvalidPkgrel("0.1".to_string())))]
    #[case("0".to_string(), Err(Error::InvalidPkgrel("0".to_string())))]
    fn pkgrel(#[case] version: String, #[case] result: Result<Pkgrel, Error>) {
        assert_eq!(result, Pkgrel::new(version));
    }

    #[rstest]
    #[case(Pkgrel::new("1".to_string()).unwrap(), Pkgrel::new("2".to_string()).unwrap())]
    #[case(Pkgrel::new("1".to_string()).unwrap(), Pkgrel::new("1.1".to_string()).unwrap())]
    #[case(Pkgrel::new("1".to_string()).unwrap(), Pkgrel::new("11".to_string()).unwrap())]
    fn pkgrel_cmp(#[case] pkgrel_a: Pkgrel, #[case] pkgrel_b: Pkgrel) {
        assert!(pkgrel_a.lt(&pkgrel_b));
    }

    #[rstest]
    #[case(Version::new("1:1-1").unwrap(), "1:1-1")]
    #[case(Version::new("1-1").unwrap(), "1-1")]
    #[case(Version::new("1").unwrap(), "1")]
    #[case(Version::new("1:1").unwrap(), "1:1")]
    fn version_to_string(#[case] version: Version, #[case] to_str: &str) {
        assert_eq!(format!("{}", version), to_str);
    }

    #[rstest]
    #[case(Version::new("1").unwrap(), Version::new("1").unwrap(), Ordering::Equal, 0)]
    #[case(Version::new("2").unwrap(), Version::new("1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1").unwrap(), Version::new("2").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1").unwrap(), Version::new("1.1").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1.1").unwrap(), Ordering::Equal, 0)]
    #[case(Version::new("1.2").unwrap(), Version::new("1.1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1.2").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1+2").unwrap(), Version::new("1+1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1+1").unwrap(), Version::new("1+2").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1.1a").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.1a").unwrap(), Version::new("1.1").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1.1a1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.1a1").unwrap(), Version::new("1.1").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1.11a").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.11a").unwrap(), Version::new("1.1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.1_a").unwrap(), Version::new("1.1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1.1_a").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1.1.a").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1.a").unwrap(), Version::new("1.1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.a").unwrap(), Version::new("1.1").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1.a").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.a1").unwrap(), Version::new("1.1").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1.a1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.a11").unwrap(), Version::new("1.1").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1.a11").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("a.1").unwrap(), Version::new("1.1").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1").unwrap(), Version::new("a.1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("foo").unwrap(), Version::new("1.1").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1").unwrap(), Version::new("foo").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("a1a").unwrap(), Version::new("a1b").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("a1b").unwrap(), Version::new("a1a").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("20220102").unwrap(), Version::new("20220202").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("20220202").unwrap(), Version::new("20220102").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.0..").unwrap(), Version::new("1.0.").unwrap(), Ordering::Equal, 0)]
    #[case(Version::new("1.0.").unwrap(), Version::new("1.0").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1..0").unwrap(), Version::new("1.0").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1..0").unwrap(), Version::new("1..0").unwrap(), Ordering::Equal, 0)]
    #[case(Version::new("1..1").unwrap(), Version::new("1..0").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1..0").unwrap(), Version::new("1..1").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1+0").unwrap(), Version::new("1.0").unwrap(), Ordering::Equal, 0)]
    #[case(Version::new("1.111").unwrap(), Version::new("1.1a1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.1a1").unwrap(), Version::new("1.111").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("01").unwrap(), Version::new("1").unwrap(), Ordering::Equal, 0)]
    #[case(Version::new("001a").unwrap(), Version::new("1a").unwrap(), Ordering::Equal, 0)]
    #[case(Version::new("1.a001a.1").unwrap(), Version::new("1.a1a.1").unwrap(), Ordering::Equal, 0)]
    fn version_cmp(
        #[case] version_a: Version,
        #[case] version_b: Version,
        #[case] ordering: Ordering,
        #[case] vercmp_result: i8,
    ) {
        assert_eq!(version_a.cmp(&version_b), ordering);
        assert_eq!(Version::vercmp(&version_a, &version_b), vercmp_result);
    }
}