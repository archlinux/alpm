use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
    iter::Peekable,
    num::NonZeroUsize,
    str::{CharIndices, Chars, FromStr},
};

use lazy_regex::{lazy_regex, Lazy};
use regex::Regex;
use semver::Version as SemverVersion;

use crate::error::Error;
use crate::Architecture;

pub(crate) static PKGREL_REGEX: Lazy<Regex> = lazy_regex!(r"^[1-9]+[0-9]*(|[.]{1}[1-9]+[0-9]*)$");
pub(crate) static PKGVER_REGEX: Lazy<Regex> = lazy_regex!(r"^([[:alnum:]][[:alnum:]_+.]*)$");

/// The version and architecture of a build tool
///
/// `BuildToolVer` is used in conjunction with `BuildTool` to denote the specific build tool a
/// package is built with. A `BuildToolVer` wraps a `Version` (that is guaranteed to have a
/// `Pkgrel`) and an `Architecture`.
///
/// ## Examples
/// ```
/// use alpm_types::BuildToolVer;
///
/// assert!(BuildToolVer::new("1-1-any").is_ok());
/// assert!(BuildToolVer::new("1").is_err());
/// assert!(BuildToolVer::new("1-1-foo").is_err());
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BuildToolVer {
    version: Version,
    architecture: Architecture,
}

impl BuildToolVer {
    /// Create a new BuildToolVer and return it in a Result
    pub fn new(buildtoolver: &str) -> Result<Self, Error> {
        const VERSION_DELIMITER: char = '-';
        match buildtoolver.rsplit_once(VERSION_DELIMITER) {
            Some((version, architecture)) => match Architecture::from_str(architecture) {
                Ok(architecture) => Ok(BuildToolVer {
                    version: Version::with_pkgrel(version)?,
                    architecture,
                }),
                Err(e) => Err(e.into()),
            },
            None => Err(Error::DelimiterNotFound {
                delimiter: VERSION_DELIMITER,
            }),
        }
    }

    /// Return a reference to the Architecture
    pub fn architecture(&self) -> &Architecture {
        &self.architecture
    }

    /// Return a reference to the Version
    pub fn version(&self) -> &Version {
        &self.version
    }
}

impl FromStr for BuildToolVer {
    type Err = Error;
    /// Create an BuildToolVer from a string and return it in a Result
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        BuildToolVer::new(input)
    }
}

impl Display for BuildToolVer {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}-{}", self.version, self.architecture)
    }
}

/// An epoch of a package
///
/// Epoch is used to indicate the downgrade of a package and is prepended to a version, delimited by
/// a `":"` (e.g. `1:` is added to `0.10.0-1` to form `1:0.10.0-1` which then orders newer than
/// `1.0.0-1`).
///
/// An Epoch wraps a usize that is guaranteed to be greater than `0`.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::Epoch;
///
/// assert!(Epoch::new("1").is_ok());
/// assert!(Epoch::new("0").is_err());
/// ```
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Epoch(pub NonZeroUsize);

impl Epoch {
    /// Create a new Epoch from a string and return it in a Result
    pub fn new(input: &str) -> Result<Self, Error> {
        match input.parse() {
            Ok(epoch) => Ok(Epoch(epoch)),
            Err(source) => Err(Error::InvalidInteger {
                kind: source.kind().clone(),
            }),
        }
    }
}

impl FromStr for Epoch {
    type Err = Error;
    /// Create an Epoch from a string and return it in a Result
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Epoch::new(input)
    }
}

impl Display for Epoch {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

/// A pkgrel of a package
///
/// Pkgrel is used to indicate the build version of a package and is appended to a version,
/// delimited by a `"-"` (e.g. `-2` is added to `1.0.0` to form `1.0.0-2` which then orders newer
/// than `1.0.0-1`).
///
/// A Pkgrel wraps a String which is guaranteed to not start with a `"0"`, to contain only numeric
/// characters (optionally delimited by a single `"."`, which must be followed by at least one
/// non-`"0"` numeric character).
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
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
        if PKGREL_REGEX.is_match(pkgrel.as_str()) {
            Ok(Pkgrel(pkgrel))
        } else {
            Err(Error::RegexDoesNotMatch {
                regex: PKGREL_REGEX.to_string(),
            })
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
/// A Pkgver wraps a `String`, which is guaranteed to only contain alphanumeric characters, `"_"`,
/// `"+"` or `"."`, but to not start with a `"_"`, a `"+"` or a `"."` character and to be at least
/// one char long.
///
/// NOTE: This implementation of Pkgver is stricter than that of libalpm/pacman. It does not allow
/// empty strings `""`, or chars that are not in the allowed set, or `"."` as the first character.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
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
pub struct Pkgver(pub(crate) String);

impl Pkgver {
    /// Create a new Pkgver from a string and return it in a Result
    pub fn new(pkgver: String) -> Result<Self, Error> {
        if PKGVER_REGEX.is_match(pkgver.as_str()) {
            Ok(Pkgver(pkgver))
        } else {
            Err(Error::RegexDoesNotMatch {
                regex: PKGVER_REGEX.to_string(),
            })
        }
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &str {
        &self.0
    }

    /// Return an iterator over all segments of this version.
    pub fn segments(&self) -> VersionSegments {
        VersionSegments::new(&self.0)
    }
}

/// This struct represents a single segment in a version string.
/// `VersionSegment`s are returned by the [VersionSegments] iterator, which is responsible for
/// splitting a version string into its segments.
///
/// Version strings are split according to the following rules:
/// - Non-alphanumeric characters always count as delimiters (`.`, `-`, `$`, etc.).
/// - Each segment also contains the info about the amount of leading delimiters for that segment.
///   Leading delimiters that directly follow after one another are grouped together. The length of
///   the delimiters is important, as it plays a crucial role in the algorithm that determines which
///   version is newer.
///
///   `1...a` would be represented as:
///
///   ```text
///   [
///     (segment: "1", delimiters: 0),
///     (segment: "a", delimiters: 3)
///   ]
///   ```
/// - There's no differentiation between different delimiters. `'$$$' == '...' == '.$-'`
/// - Alphanumeric strings are also split into individual sub-segments. This is done by walking over
///   the string and splitting it every time a switch from alphabetic to numeric is detected or vice
///   versa.
///
///   `1.1asdf123.0` would be represented as:
///
///   ```text
///   [
///     (segment: "1", delimiters: 0),
///     (segment: "1", delimiters: 1)
///     (segment: "asdf", delimiters: 0)
///     (segment: "123", delimiters: 0)
///     (segment: "0", delimiters: 1)
///   ]
///   ```
/// - Trailing delimiters are encoded as an empty string.
///
///   `1...` would be represented as:
///
///   ```text
///   [
///     (segment: "1", delimiters: 0),
///     (segment: "", delimiters: 3),
///   ]
///   ```
#[derive(Debug, Clone, PartialEq)]
pub struct VersionSegment<'a> {
    /// The string representation of the next segment
    pub segment: &'a str,
    /// The amount of leading delimiters that were found for this segment
    pub delimiters: usize,
}

impl<'a> VersionSegment<'a> {
    /// Create a new instance of a VersionSegment consisting of the segment's string and the amount
    /// of leading delimiters.
    pub fn new(segment: &'a str, delimiters: usize) -> Self {
        Self {
            segment,
            delimiters,
        }
    }

    /// Passhrough to `self.segment.is_empty()` for convenience purposes.
    pub fn is_empty(&self) -> bool {
        self.segment.is_empty()
    }

    /// Passhrough to `self.segment.chars()` for convenience purposes.
    pub fn chars(&self) -> Chars<'a> {
        self.segment.chars()
    }

    /// Passhrough `self.segment.parse()` for convenience purposes.
    pub fn parse<T: FromStr>(&self) -> Result<T, T::Err> {
        FromStr::from_str(self.segment)
    }

    /// Compare the `self`'s segment string with segment string of `other`.
    pub fn str_cmp(&self, other: &VersionSegment) -> Ordering {
        self.segment.cmp(other.segment)
    }
}

/// An [Iterator] over all [VersionSegment]s of an upstream version string.
/// Check the documentation on [VersionSegment] to see how a string is split into segments.
///
/// Important note:
/// Trailing delimiters will also produce a trailing [VersionSegment] with an empty string.
///
/// This iterator is capable of handling utf-8 strings.
/// However, non alphanumeric chars are still interpreted as delimiters.
pub struct VersionSegments<'a> {
    /// The original version string. We need that reference so we can get some string
    /// slices based on indices later on.
    version: &'a str,
    /// An iterator over the version's chars and their respective start byte's index.
    version_chars: Peekable<CharIndices<'a>>,
}

impl<'a> VersionSegments<'a> {
    /// Create a new instance of a VersionSegments iterator.
    pub fn new(version: &'a str) -> Self {
        VersionSegments {
            version,
            version_chars: version.char_indices().peekable(),
        }
    }
}

impl<'a> Iterator for VersionSegments<'a> {
    type Item = VersionSegment<'a>;

    /// Get the next [VersionSegment] of this version string.
    fn next(&mut self) -> Option<VersionSegment<'a>> {
        // Used to track the number of delimiters the next segment is prefixed with.
        let mut delimiter_count = 0;

        // First up, get the delimiters out of the way.
        // Peek at the next char, if it's a delimiter, consume it and increase the delimiter count.
        while let Some((_, char)) = self.version_chars.peek() {
            // An alphanumeric char indicates that we reached the next segment.
            if char.is_alphanumeric() {
                break;
            }

            self.version_chars.next();
            delimiter_count += 1;
            continue;
        }

        // Get the next char. If there's no further char, we reached the end of the version string.
        let Some((first_index, first_char)) = self.version_chars.next() else {
            // We're at the end of the string and now have to differentiate between two cases:

            // 1. There are no trailing delimiters. We can just return `None` as we truly reached
            //    the end.
            if delimiter_count == 0 {
                return None;
            }

            // 2. There's no further segment, but there were some trailing delimiters. The
            //    comparison algorithm considers this case which is why we have to somehow encode
            //    it. We do so by returning an empty segment.
            return Some(VersionSegment::new("", delimiter_count));
        };

        // Cache the last valid char + index that was checked. We need this to
        // calculate the offset in case the last char is a multi-byte UTF-8 char.
        let mut last_char = first_char;
        let mut last_char_index = first_index;

        // The following section now handles the splitting of an alphanumeric string into its
        // sub-segments. As described in the [VersionSegment] docs, the string needs to be split
        // every time a switch from alphabetic to numeric or vice versa is detected.

        let is_numeric = first_char.is_numeric();

        if is_numeric {
            // Go through chars until we hit a non-numeric char or reached the end of the string.
            #[allow(clippy::while_let_on_iterator)]
            while let Some((index, next_char)) =
                self.version_chars.next_if(|(_, peek)| peek.is_numeric())
            {
                last_char_index = index;
                last_char = next_char;
            }
        } else {
            // Go through chars until we hit a non-alphabetic char or reached the end of the string.
            #[allow(clippy::while_let_on_iterator)]
            while let Some((index, next_char)) =
                self.version_chars.next_if(|(_, peek)| peek.is_alphabetic())
            {
                last_char_index = index;
                last_char = next_char;
            }
        }

        // Create a subslice based on the indices of the first and last char.
        // The last char might be multi-byte, which is why we add its length.
        let segment_slice = &self.version[first_index..(last_char_index + last_char.len_utf8())];

        Some(VersionSegment::new(segment_slice, delimiter_count))
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
    /// This block implements the logic to determine which of two package versions is newer or
    /// whether they're considered equal.
    ///
    /// This logic is surprisingly complex as it mirrors the current C-alpmlib implementation for
    /// backwards compatibility reasons.
    /// <https://gitlab.archlinux.org/pacman/pacman/-/blob/a2d029388c7c206f5576456f91bfbea2dca98c96/lib/libalpm/version.c#L83-217>
    fn cmp(&self, other: &Self) -> Ordering {
        // Equal strings are considered equal versions.
        if self.inner() == other.inner() {
            return Ordering::Equal;
        }

        let mut self_segments = self.segments().peekable();
        let mut other_segments = other.segments().peekable();

        // Loop through both versions' segments and compare them.
        loop {
            // Try to get the next segments
            let self_segment = self_segments.next();
            let other_segment = other_segments.next();

            // Make sure that there's a next segment for both versions.
            let (self_segment, other_segment) = match (self_segment, other_segment) {
                // Both segments exist, we continue after match.
                (Some(self_seg), Some(other_seg)) => (self_seg, other_seg),

                // Both versions reached their end and are thereby equal.
                (None, None) => return Ordering::Equal,

                // One version is longer than the other.
                // Sadly, this isn't trivial to handle.
                //
                // The rules are as follows:
                // Versions with at least two additional segments are always newer.
                // -> `1.a.0` > `1`
                //        ⤷ Two more segment, include one delimiter
                // -> `1.a0` > `1`
                //        ⤷ Two more segment, thereby an alphanumerical string.
                //
                // If one version is exactly one segment and has a delimiter, it's also considered
                // newer.
                // -> `1.0` > `1`
                // -> `1.a` > `1`
                //      ⤷ Delimiter exists, thereby newer
                //
                // If one version is exactly one segment longer and that segment is
                // purely alphabetic **without** a leading delimiter, that segment is considered
                // older. The reason for this is to handle pre-releases (e.g. alpha/beta).
                // -> `1.0alpha` > `1.0`
                //          ⤷ Purely alphabetic last segment, without delimiter and thereby older.
                (Some(seg), None) => {
                    // There's at least one more segment, making `Self` effectively newer.
                    // It's either an alphanumeric string or another segment separated with a
                    // delimiter.
                    if self_segments.next().is_some() {
                        return Ordering::Greater;
                    }

                    // We now know that this is also the last segment of `self`.
                    // If the current segment has a leading delimiter, it's also considered newer.
                    if seg.delimiters > 0 {
                        return Ordering::Greater;
                    }

                    // If all chars are alphabetic, `self` is consider older.
                    if !seg.is_empty() && seg.chars().all(char::is_alphabetic) {
                        return Ordering::Less;
                    }

                    return Ordering::Greater;
                }

                // This is the same logic as above, but inverted.
                (None, Some(seg)) => {
                    if other_segments.next().is_some() {
                        return Ordering::Less;
                    }
                    if seg.delimiters > 0 {
                        return Ordering::Less;
                    }
                    if !seg.is_empty() && seg.chars().all(char::is_alphabetic) {
                        return Ordering::Greater;
                    }
                    return Ordering::Less;
                }
            };

            // Special case:
            // One or both of the segments is empty. That means that the end of the version string
            // has been reached, but there were some trailing delimiters.
            // Possible examples of how this might look:
            // `1.0.` < `1.0.0`
            // `1.0.` == `1.0.`
            // `1.0.alpha` < `1.0.`
            if other_segment.is_empty() && self_segment.is_empty() {
                // Both reached the end of their version with a trailing delimiter.
                // Counterintuitively, the trailing delimiter count is not considered and both
                // versions are considered equal
                // `1.0....` == `1.0.`
                return Ordering::Equal;
            } else if self_segment.is_empty() {
                // Check if there's at least one other segment on the `other` version.
                // If so, that one is always considered newer.
                // `1.0.1.1` > `1.0.`
                // `1.0.alpha1` > `1.0.`
                // `1.0.alpha.1` > `1.0.`
                //           ⤷ More segments and thereby always newer
                if other_segments.peek().is_some() {
                    return Ordering::Less;
                }

                // In case there's no further segment, both versions reached the last segment.
                // We now have to consider the special case where `other` is purely alphabetic.
                // If that's the case, `self` will be considered newer, as the alphabetic string
                // indicates a pre-release,
                // `1.0.` > `1.0.alpha`.
                //                   ⤷ Purely alphabetic last segment and thereby older.
                //
                // Also, we know that `other_segment` isn't empty at this point.
                if other_segment.chars().all(char::is_alphabetic) {
                    return Ordering::Greater;
                }

                // In all other cases, `other` is newer.
                return Ordering::Less;
            } else if other_segment.is_empty() {
                // Check docs above, as it's the same logic as above, just inverted.
                if self_segments.peek().is_some() {
                    return Ordering::Greater;
                }

                if self_segment.chars().all(char::is_alphabetic) {
                    return Ordering::Less;
                }

                return Ordering::Greater;
            }

            // We finally reached the end handling special cases when the version string ended.
            // From now on, we know that we have two actual segments that might be prefixed by
            // some delimiters.

            // Special case:
            // If one of the segments has more leading delimiters as the other, it's considered
            // newer.
            // `1..0.0` > `1.2.0`
            //         ⤷ Two delimiters, thereby always newer.
            // `1..0.0` < `1..2.0`
            //                ⤷ Same amount of delimiters, now `2 > 0`
            if self_segment.delimiters != other_segment.delimiters {
                return self_segment.delimiters.cmp(&other_segment.delimiters);
            }

            // Check whether any of the segments are numeric.
            // Numeric segments are always considered newer than non-numeric segments.
            // E.g. `1.0.0` > `1.lol.0`
            //         ⤷ `0` vs `lol`. `0` is purely numeric and bigger than a alphanumeric one.
            let self_is_numeric =
                !self_segment.is_empty() && self_segment.chars().all(char::is_numeric);
            let other_is_numeric =
                !other_segment.is_empty() && other_segment.chars().all(char::is_numeric);

            if self_is_numeric && !other_is_numeric {
                return Ordering::Greater;
            } else if !self_is_numeric && other_is_numeric {
                return Ordering::Less;
            }

            // In case both are numeric, we do a number comparison.
            // We can parse the string as we know that they only consist of digits, hence the
            // unwrap.
            //
            // Trailing zeroes are to be ignored, which is automatically done by Rust's number
            // parser. E.g. `1.0001.1` == `1.1.1`
            //                  ⤷ `000` is ignored in comparison.
            if self_is_numeric && other_is_numeric {
                let ordering = self_segment
                    .parse::<usize>()
                    .unwrap()
                    .cmp(&other_segment.parse::<usize>().unwrap());
                match ordering {
                    Ordering::Less => return Ordering::Less,
                    Ordering::Equal => (),
                    Ordering::Greater => return Ordering::Greater,
                }

                // However, there is a special case that needs to be handled when both numbers are
                // considered equal.
                //
                // To have a name for the following edge-case, let's call these "higher-level
                // segments". Higher-level segments are string segments that aren't separated with
                // a delimiter. E.g. on `1.10test11` the string `10test11` would be a
                // higher-level segment that's returned as segments of:
                //
                // `['10', 'test', '11']`
                //
                // The rule is:
                // Pure numeric higher-level segments are superior to mixed alphanumeric segments.
                // -> `1.10` > `1.11a1`
                // -> `1.10` > `1.11a1.2`
                //                  ⤷ `11a1` is alphanumeric and smaller than pure numerics.
                //
                // The current higher-level segment is considered purely numeric if the current
                // segment is numeric and the next segment is split via delimiter,
                // which indicates that a new higher-level segment has started. A
                // follow-up alphabetic segment in the same higher-level
                // segment wouldn't have a delimiter.
                //
                // If there's no further segment, we reached the end of the version string, also
                // indicating a purely numeric string.
                let other_is_pure_numeric = other_segments
                    .peek()
                    .map(|seg| seg.delimiters > 0)
                    .unwrap_or(true);
                let self_is_pure_numeric = self_segments
                    .peek()
                    .map(|seg| seg.delimiters > 0)
                    .unwrap_or(true);

                // One is purely numeric, the other isn't. We can return early.
                if self_is_pure_numeric && !other_is_pure_numeric {
                    return Ordering::Greater;
                } else if !self_is_pure_numeric && other_is_pure_numeric {
                    return Ordering::Less;
                }

                // Now we know that both are either numeric or alphanumeric and can take a look at
                // the next segment.
                continue;
            }
            // At this point, we know that the segments are alphabetic.
            // We do a simple string comparison to determine the newer version.
            // If the strings are equal, we check the next segments.
            match self_segment.str_cmp(&other_segment) {
                Ordering::Less => return Ordering::Less,
                Ordering::Equal => continue,
                Ordering::Greater => return Ordering::Greater,
            }
        }
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
/// However, for backwards compatibility reasons it is possible to initialize a `SchemaVersion`
/// using a non-semver compatible string, *if* it can be parsed to a single `u64` (e.g. `"1"`).
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
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
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SchemaVersion(SemverVersion);

impl SchemaVersion {
    /// Create a new SchemaVersion from a string
    ///
    /// When providing a non-semver string with only a number (i.e. no minor or patch version), the
    /// number is treated as the major version (e.g. `"23"` -> `"23.0.0"`).
    pub fn new(version: &str) -> Result<SchemaVersion, Error> {
        if !version.contains('.') {
            match version.parse() {
                Ok(major) => Ok(SchemaVersion(SemverVersion::new(major, 0, 0))),
                Err(e) => Err(Error::InvalidInteger {
                    kind: e.kind().clone(),
                }),
            }
        } else {
            match SemverVersion::parse(version) {
                Ok(version) => Ok(SchemaVersion(version)),
                Err(e) => Err(Error::InvalidSemver {
                    kind: e.to_string(),
                }),
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
        write!(fmt, "{}", self.0)
    }
}

/// A version of a package
///
/// A `Version` tracks an optional `Epoch`, a `Pkgver` and an optional `Pkgrel`.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Epoch, Pkgrel, Pkgver, Version};
///
/// let version = Version::new("1:2-3").unwrap();
/// assert_eq!(version.epoch, Some(Epoch::new("1").unwrap()));
/// assert_eq!(version.pkgver, Pkgver::new("2".to_string()).unwrap());
/// assert_eq!(version.pkgrel, Some(Pkgrel::new("3".to_string()).unwrap()));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub pkgver: Pkgver,
    pub epoch: Option<Epoch>,
    pub pkgrel: Option<Pkgrel>,
}

impl Version {
    /// Create a new Version from a string and return it in a Result
    pub fn new(version: &str) -> Result<Self, Error> {
        // try to split off epoch
        let (epoch, pkgver_pkgrel) = version.split_once(':').unzip();
        // if there's no epoch, the entire thing is pkgver and maybe pkgrel
        let pkgver_pkgrel = pkgver_pkgrel.unwrap_or(version);

        // try to split off pkgrel
        let (pkgver, pkgrel) = pkgver_pkgrel.split_once('-').unzip();
        // if there's no pkgrel, the entire thing is the pkgver
        let pkgver = pkgver.unwrap_or(pkgver_pkgrel);

        Ok(Version {
            pkgver: pkgver.parse()?,
            epoch: if let Some(s) = epoch {
                Some(s.parse()?)
            } else {
                None
            },
            pkgrel: if let Some(s) = pkgrel {
                Some(s.parse()?)
            } else {
                None
            },
        })
    }

    /// Create a new Version, which is guaranteed to have a Pkgrel
    pub fn with_pkgrel(version: &str) -> Result<Self, Error> {
        match Version::new(version) {
            Ok(version) => {
                if version.pkgrel.is_some() {
                    Ok(version)
                } else {
                    Err(Error::MissingComponent {
                        component: "pkgrel",
                    })
                }
            }
            Err(e) => Err(e),
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
    /// use alpm_types::Version;
    ///
    /// assert_eq!(
    ///     Version::vercmp(
    ///         &Version::new("1.0.0").unwrap(),
    ///         &Version::new("0.1.0").unwrap()
    ///     ),
    ///     1
    /// );
    /// assert_eq!(
    ///     Version::vercmp(
    ///         &Version::new("1.0.0").unwrap(),
    ///         &Version::new("1.0.0").unwrap()
    ///     ),
    ///     0
    /// );
    /// assert_eq!(
    ///     Version::vercmp(
    ///         &Version::new("0.1.0").unwrap(),
    ///         &Version::new("1.0.0").unwrap()
    ///     ),
    ///     -1
    /// );
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
        if let Some(epoch) = self.epoch {
            write!(fmt, "{}:", epoch)?;
        }

        write!(fmt, "{}", self.pkgver)?;

        if let Some(pkgrel) = &self.pkgrel {
            write!(fmt, "-{}", pkgrel)?;
        }

        Ok(())
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionComparison {
    Less,
    LessOrEqual,
    Equal,
    GreaterOrEqual,
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
}

impl FromStr for VersionComparison {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "<" => Ok(VersionComparison::Less),
            "<=" => Ok(VersionComparison::LessOrEqual),
            "=" => Ok(VersionComparison::Equal),
            ">=" => Ok(VersionComparison::GreaterOrEqual),
            ">" => Ok(VersionComparison::Greater),
            _ => Err(strum::ParseError::VariantNotFound.into()),
        }
    }
}

/// A version requirement, e.g. for a dependency package.
///
/// It consists of a target version and a comparison function. A version requirement of `>=1.5` has
/// a target version of `1.5` and a comparison function of [`VersionComparison::GreaterOrEqual`].
///
/// ## Examples
///
/// ```
/// use alpm_types::{Version, VersionComparison, VersionRequirement};
///
/// let requirement = VersionRequirement::new(">=1.5").unwrap();
///
/// assert_eq!(requirement.comparison, VersionComparison::GreaterOrEqual);
/// assert_eq!(requirement.version, Version::new("1.5").unwrap());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionRequirement {
    pub comparison: VersionComparison,
    pub version: Version,
}

impl VersionRequirement {
    /// Parses a version requirement from a string.
    ///
    /// ## Errors
    ///
    /// Returns an error if the comparison function or version are malformed.
    pub fn new(s: &str) -> Result<Self, Error> {
        fn is_comparison_char(c: char) -> bool {
            matches!(c, '<' | '=' | '>')
        }

        let comparison_end = s
            .find(|c| !is_comparison_char(c))
            .ok_or(Error::MissingComponent {
                component: "operator",
            })?;

        let (comparison, version) = s.split_at(comparison_end);

        let comparison = comparison.parse()?;
        let version = version.parse()?;

        Ok(VersionRequirement {
            comparison,
            version,
        })
    }

    /// Returns `true` if the requirement is satisfied by the given package version.
    ///
    /// ## Examples
    ///
    /// ```
    /// use alpm_types::{Version, VersionRequirement};
    ///
    /// let requirement = VersionRequirement::new(">=1.5-3").unwrap();
    ///
    /// assert!(!requirement.is_satisfied_by(&Version::new("1.5").unwrap()));
    /// assert!(requirement.is_satisfied_by(&Version::new("1.5-3").unwrap()));
    /// assert!(requirement.is_satisfied_by(&Version::new("1.6").unwrap()));
    /// assert!(requirement.is_satisfied_by(&Version::new("2:1.0").unwrap()));
    /// assert!(!requirement.is_satisfied_by(&Version::new("1.0").unwrap()));
    /// ```
    pub fn is_satisfied_by(&self, ver: &Version) -> bool {
        self.comparison.is_compatible_with(ver.cmp(&self.version))
    }
}

impl FromStr for VersionRequirement {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use std::num::IntErrorKind;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("1.0.0", Ok(SchemaVersion(SemverVersion::new(1, 0, 0))))]
    #[case("1", Ok(SchemaVersion(SemverVersion::new(1, 0, 0))))]
    #[case("-1.0.0", Err(Error::InvalidSemver { kind: String::from("unexpected character '-' while parsing major version number") }))]
    fn schema_version(#[case] version: &str, #[case] result: Result<SchemaVersion, Error>) {
        assert_eq!(result, SchemaVersion::new(version))
    }

    #[rstest]
    #[case(
        "1.0.0-1-any",
        Ok(BuildToolVer{version: Version::new("1.0.0-1").unwrap(), architecture: Architecture::from_str("any").unwrap()}),
    )]
    #[case(
        "1:1.0.0-1-any",
        Ok(BuildToolVer{version: Version::new("1:1.0.0-1").unwrap(), architecture: Architecture::from_str("any").unwrap()}),
    )]
    #[case(
        "1.0.0",
        Err(Error::DelimiterNotFound { delimiter: '-' }),
    )]
    #[case(
        "1.0.0-any",
        Err(Error::MissingComponent { component: "pkgrel" }),
    )]
    #[case(
        ".1.0.0-1-any",
        Err(Error::RegexDoesNotMatch { regex: PKGVER_REGEX.to_string() }),
    )]
    #[case(
        "1.0.0-1-foo",
        Err(strum::ParseError::VariantNotFound.into()),
    )]
    fn buildtoolver_new(#[case] buildtoolver: &str, #[case] result: Result<BuildToolVer, Error>) {
        assert_eq!(BuildToolVer::new(buildtoolver), result);
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
            epoch: Some(Epoch::new("1").unwrap()),
            pkgrel: Some(Pkgrel::new("1".to_string()).unwrap()),
        }),
    )]
    #[case(
        "1:foo",
        Ok(Version{
            pkgver: Pkgver::new("foo".to_string()).unwrap(),
            epoch: Some(Epoch::new("1").unwrap()),
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
    #[case("-1foo:1", Err(Error::InvalidInteger { kind: IntErrorKind::InvalidDigit }))]
    #[case("1-foo:1", Err(Error::InvalidInteger { kind: IntErrorKind::InvalidDigit }))]
    #[case("1:1:foo-1", Err(Error::RegexDoesNotMatch { regex: PKGVER_REGEX.to_string() }))]
    #[case("1:foo-1-1", Err(Error::RegexDoesNotMatch { regex: PKGREL_REGEX.to_string() }))]
    #[case("", Err(Error::RegexDoesNotMatch { regex: PKGVER_REGEX.to_string() }))]
    #[case(":", Err(Error::RegexDoesNotMatch { regex: PKGVER_REGEX.to_string() }))]
    #[case(".", Err(Error::RegexDoesNotMatch { regex: PKGVER_REGEX.to_string() }))]
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
    #[case(
        "1.0.0-1",
        Ok(Version{
            pkgver: Pkgver::new("1.0.0".to_string()).unwrap(),
            pkgrel: Some(Pkgrel::new("1".to_string()).unwrap()),
            epoch: None,
        })
    )]
    #[case("1.0.0", Err(Error::MissingComponent { component: "pkgrel" }))]
    fn version_with_pkgrel(#[case] version: &str, #[case] result: Result<Version, Error>) {
        assert_eq!(result, Version::with_pkgrel(version));
    }

    #[rstest]
    #[case("1", Ok(Epoch(NonZeroUsize::new(1).unwrap())))]
    #[case("0", Err(Error::InvalidInteger { kind: IntErrorKind::Zero }))]
    #[case("-0", Err(Error::InvalidInteger { kind: IntErrorKind::InvalidDigit }))]
    #[case("z", Err(Error::InvalidInteger { kind: IntErrorKind::InvalidDigit }))]
    fn epoch(#[case] version: &str, #[case] result: Result<Epoch, Error>) {
        assert_eq!(result, Epoch::new(version));
    }

    #[rstest]
    #[case("foo".to_string(), Ok(Pkgver::new("foo".to_string()).unwrap()))]
    #[case("1.0.0".to_string(), Ok(Pkgver::new("1.0.0".to_string()).unwrap()))]
    #[case("1:foo".to_string(), Err(Error::RegexDoesNotMatch { regex: PKGVER_REGEX.to_string() }))]
    #[case("foo-1".to_string(), Err(Error::RegexDoesNotMatch { regex: PKGVER_REGEX.to_string() }))]
    #[case("foo,1".to_string(), Err(Error::RegexDoesNotMatch { regex: PKGVER_REGEX.to_string() }))]
    #[case(".foo".to_string(), Err(Error::RegexDoesNotMatch { regex: PKGVER_REGEX.to_string() }))]
    #[case("_foo".to_string(), Err(Error::RegexDoesNotMatch { regex: PKGVER_REGEX.to_string() }))]
    // ß is not in [:alnum:]
    #[case("ß".to_string(), Err(Error::RegexDoesNotMatch { regex: PKGVER_REGEX.to_string() }))]
    #[case("1.ß".to_string(), Err(Error::RegexDoesNotMatch { regex: PKGVER_REGEX.to_string() }))]
    fn pkgver(#[case] version: String, #[case] result: Result<Pkgver, Error>) {
        assert_eq!(result, Pkgver::new(version));
    }

    #[rstest]
    #[case("1".to_string(), Ok(Pkgrel::new("1".to_string()).unwrap()))]
    #[case("1.1".to_string(), Ok(Pkgrel::new("1.1".to_string()).unwrap()))]
    #[case("0.1".to_string(), Err(Error::RegexDoesNotMatch { regex: PKGREL_REGEX.to_string() }))]
    #[case("0".to_string(), Err(Error::RegexDoesNotMatch { regex: PKGREL_REGEX.to_string() }))]
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
    #[case(Version::new("1"), Version::new("1"), Ordering::Equal)]
    #[case(Version::new("2"), Version::new("1"), Ordering::Greater)]
    #[case(Version::new("1"), Version::new("2"), Ordering::Less)]
    #[case(Version::new("1"), Version::new("1.1"), Ordering::Less)]
    #[case(Version::new("1.1"), Version::new("1"), Ordering::Greater)]
    #[case(Version::new("1.1"), Version::new("1.1"), Ordering::Equal)]
    #[case(Version::new("1.2"), Version::new("1.1"), Ordering::Greater)]
    #[case(Version::new("1.1"), Version::new("1.2"), Ordering::Less)]
    #[case(Version::new("1+2"), Version::new("1+1"), Ordering::Greater)]
    #[case(Version::new("1+1"), Version::new("1+2"), Ordering::Less)]
    #[case(Version::new("1.1"), Version::new("1.1a"), Ordering::Greater)]
    #[case(Version::new("1.1a"), Version::new("1.1"), Ordering::Less)]
    #[case(Version::new("1.1"), Version::new("1.1a1"), Ordering::Greater)]
    #[case(Version::new("1.1a1"), Version::new("1.1"), Ordering::Less)]
    #[case(Version::new("1.1"), Version::new("1.11a"), Ordering::Less)]
    #[case(Version::new("1.11a"), Version::new("1.1"), Ordering::Greater)]
    #[case(Version::new("1.1_a"), Version::new("1.1"), Ordering::Greater)]
    #[case(Version::new("1.1"), Version::new("1.1_a"), Ordering::Less)]
    #[case(Version::new("1.1"), Version::new("1.1.a"), Ordering::Less)]
    #[case(Version::new("1.1.a"), Version::new("1.1"), Ordering::Greater)]
    #[case(Version::new("1.a"), Version::new("1.1"), Ordering::Less)]
    #[case(Version::new("1.1"), Version::new("1.a"), Ordering::Greater)]
    #[case(Version::new("1.a1"), Version::new("1.1"), Ordering::Less)]
    #[case(Version::new("1.1"), Version::new("1.a1"), Ordering::Greater)]
    #[case(Version::new("1.a11"), Version::new("1.1"), Ordering::Less)]
    #[case(Version::new("1.1"), Version::new("1.a11"), Ordering::Greater)]
    #[case(Version::new("a.1"), Version::new("1.1"), Ordering::Less)]
    #[case(Version::new("1.1"), Version::new("a.1"), Ordering::Greater)]
    #[case(Version::new("foo"), Version::new("1.1"), Ordering::Less)]
    #[case(Version::new("1.1"), Version::new("foo"), Ordering::Greater)]
    #[case(Version::new("a1a"), Version::new("a1b"), Ordering::Less)]
    #[case(Version::new("a1b"), Version::new("a1a"), Ordering::Greater)]
    #[case(Version::new("20220102"), Version::new("20220202"), Ordering::Less)]
    #[case(Version::new("20220202"), Version::new("20220102"), Ordering::Greater)]
    #[case(Version::new("1.0.."), Version::new("1.0."), Ordering::Equal)]
    #[case(Version::new("1.0."), Version::new("1.0"), Ordering::Greater)]
    #[case(Version::new("1..0"), Version::new("1.0"), Ordering::Greater)]
    #[case(Version::new("1..0"), Version::new("1..0"), Ordering::Equal)]
    #[case(Version::new("1..1"), Version::new("1..0"), Ordering::Greater)]
    #[case(Version::new("1..0"), Version::new("1..1"), Ordering::Less)]
    #[case(Version::new("1+0"), Version::new("1.0"), Ordering::Equal)]
    #[case(Version::new("1.111"), Version::new("1.1a1"), Ordering::Greater)]
    #[case(Version::new("1.1a1"), Version::new("1.111"), Ordering::Less)]
    #[case(Version::new("01"), Version::new("1"), Ordering::Equal)]
    #[case(Version::new("001a"), Version::new("1a"), Ordering::Equal)]
    #[case(Version::new("1.a001a.1"), Version::new("1.a1a.1"), Ordering::Equal)]
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
    }

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
    #[case("")]
    #[case("<<")]
    #[case("==")]
    #[case("!=")]
    #[case(" =")]
    #[case("= ")]
    #[case("<1")]
    fn invalid_version_comparison(#[case] comparison: &str) {
        assert_eq!(
            comparison.parse::<VersionComparison>(),
            Err(strum::ParseError::VariantNotFound.into())
        );
    }

    /// Test successful parsing for version requirement strings.
    #[rstest]
    #[case("=1", Ok(VersionRequirement {
        comparison: VersionComparison::Equal,
        version: Version::new("1").unwrap(),
    }))]
    #[case("<=42:abcd-2.4", Ok(VersionRequirement {
        comparison: VersionComparison::LessOrEqual,
        version: Version::new("42:abcd-2.4").unwrap(),
    }))]
    #[case(">3.1", Ok(VersionRequirement {
        comparison: VersionComparison::Greater,
        version: Version::new("3.1").unwrap(),
    }))]
    #[case("<=", Err(Error::MissingComponent { component: "operator" }))]
    #[case("<>3.1", Err(strum::ParseError::VariantNotFound.into()))]
    #[case("3.1", Err(strum::ParseError::VariantNotFound.into()))]
    #[case("=>3.1", Err(strum::ParseError::VariantNotFound.into()))]
    #[case("<3.1>3.2", Err(Error::RegexDoesNotMatch { regex: PKGVER_REGEX.to_string() }))]
    fn version_requirement(
        #[case] requirement: &str,
        #[case] result: Result<VersionRequirement, Error>,
    ) {
        assert_eq!(requirement.parse(), result);
    }

    #[rstest]
    #[case("=1", "1", true)]
    #[case("=1", "1.0", false)]
    #[case("=1", "1-1", false)]
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
    #[case("1.0.0", vec![("1", 0), ("0", 1), ("0", 1)])]
    #[case("1..0", vec![("1", 0), ("0", 2)])]
    #[case("1.0.", vec![("1", 0), ("0", 1), ("", 1)])]
    #[case("1..", vec![("1", 0), ("", 2)])]
    #[case("1.🗻lol.0", vec![("1", 0), ("lol", 2), ("0", 1)])]
    #[case("1.🗻lol.", vec![("1", 0), ("lol", 2), ("", 1)])]
    #[case("20220202", vec![("20220202", 0)])]
    #[case("some_string", vec![("some", 0), ("string", 1)])]
    #[case("alpha7654numeric321", vec![("alpha", 0), ("7654", 0), ("numeric", 0), ("321", 0)])]
    fn version_segment_iterator(
        #[case] version: &str,
        #[case] expected_segments: Vec<(&'static str, usize)>,
    ) {
        let version = Pkgver(version.to_string());
        // Convert the simplified definition above into actual VersionSegment instances.
        let expected = expected_segments
            .into_iter()
            .map(|(segment, delimiters)| VersionSegment::new(segment, delimiters))
            .collect::<Vec<VersionSegment>>();

        let mut segments_iter = version.segments();
        let mut expected_iter = expected.clone().into_iter();

        // Iterate over both iterators.
        // We do it manually to ensure that they both end at the same time.
        loop {
            let next_segment = segments_iter.next();
            assert_eq!(
                next_segment,
                expected_iter.next(),
                "Failed for segment {next_segment:?} in version string {version}:\nsegments: {:?}\n expected: {:?}",
                version.segments().collect::<Vec<VersionSegment>>(),
                expected,
            );
            if next_segment.is_none() {
                break;
            }
        }
    }
}
