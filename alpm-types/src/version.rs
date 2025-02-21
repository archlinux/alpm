use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
    iter::Peekable,
    num::NonZeroUsize,
    str::{CharIndices, Chars, FromStr},
};

use semver::Version as SemverVersion;
use serde::Serialize;
use winnow::{
    ModalResult,
    Parser,
    ascii::{dec_uint, digit1},
    combinator::{Repeat, cut_err, eof, opt, preceded, repeat, seq, terminated},
    error::{StrContext, StrContextValue},
    token::{one_of, take_till},
};

use crate::{Architecture, error::Error};

/// The version and architecture of a build tool
///
/// `BuildToolVersion` is used in conjunction with `BuildTool` to denote the specific build tool a
/// package is built with. A `BuildToolVersion` wraps a `Version` (that is guaranteed to have a
/// `PackageRelease`) and an `Architecture`.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::BuildToolVersion;
///
/// assert!(BuildToolVersion::from_str("1-1-any").is_ok());
/// assert!(BuildToolVersion::from_str("1").is_ok());
/// assert!(BuildToolVersion::from_str("1-1").is_err());
/// assert!(BuildToolVersion::from_str("1-1-foo").is_err());
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct BuildToolVersion {
    version: Version,
    architecture: Option<Architecture>,
}

impl BuildToolVersion {
    /// Create a new BuildToolVersion
    pub fn new(version: Version, architecture: Option<Architecture>) -> Self {
        BuildToolVersion {
            version,
            architecture,
        }
    }

    /// Return a reference to the Architecture
    pub fn architecture(&self) -> &Option<Architecture> {
        &self.architecture
    }

    /// Return a reference to the Version
    pub fn version(&self) -> &Version {
        &self.version
    }
}

impl FromStr for BuildToolVersion {
    type Err = Error;
    /// Create an BuildToolVersion from a string and return it in a Result
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const VERSION_DELIMITER: char = '-';
        match s.rsplit_once(VERSION_DELIMITER) {
            Some((version, architecture)) => match Architecture::from_str(architecture) {
                Ok(architecture) => Ok(BuildToolVersion {
                    version: Version::with_pkgrel(version)?,
                    architecture: Some(architecture),
                }),
                Err(err) => Err(err.into()),
            },
            None => Ok(BuildToolVersion {
                version: Version::from_str(s)?,
                architecture: None,
            }),
        }
    }
}

impl Display for BuildToolVersion {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        if let Some(architecture) = &self.architecture {
            write!(fmt, "{}-{}", self.version, architecture)
        } else {
            write!(fmt, "{}", self.version)
        }
    }
}

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
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
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

/// A pkgrel of a package
///
/// PackageRelease is used to indicate the build version of a package and is appended to a version,
/// delimited by a `"-"` (e.g. `-2` is added to `1.0.0` to form `1.0.0-2` which then orders newer
/// than `1.0.0-1`).
///
/// A PackageRelease wraps a String which must consist of one or more numeric digits,
/// optionally followed by a period (`.`) and one or more additional numeric digits.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::PackageRelease;
///
/// assert!(PackageRelease::new("1".to_string()).is_ok());
/// assert!(PackageRelease::new("1.1".to_string()).is_ok());
/// assert!(PackageRelease::new("0".to_string()).is_ok());
/// assert!(PackageRelease::new("a".to_string()).is_err());
/// assert!(PackageRelease::new("1.a".to_string()).is_err());
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct PackageRelease(String);

impl PackageRelease {
    /// Create a new PackageRelease from a string and return it in a Result
    pub fn new(pkgrel: String) -> Result<Self, Error> {
        PackageRelease::from_str(pkgrel.as_str())
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &str {
        &self.0
    }

    /// Parses a [`PackageRelease`] from a string slice.
    ///
    /// Consumes all of its input.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        (
            digit1
                .context(StrContext::Label("package release"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "positive decimal integer",
                ))),
            opt(('.', cut_err(digit1))
                .context(StrContext::Label("package release"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "single '.' followed by positive decimal integer",
                )))),
            eof.context(StrContext::Expected(StrContextValue::Description(
                "end of package release value",
            ))),
        )
            .take()
            .map(|s: &str| Self(s.to_string()))
            .parse_next(input)
    }
}

impl FromStr for PackageRelease {
    type Err = Error;
    /// Create a PackageRelease from a string and return it in a Result
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

impl Display for PackageRelease {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

/// A pkgver of a package
///
/// PackageVersion is used to denote the upstream version of a package.
///
/// A PackageVersion wraps a `String`, which is guaranteed to only contain alphanumeric characters,
/// `"_"`, `"+"` or `"."`, but to not start with a `"_"`, a `"+"` or a `"."` character and to be at
/// least one char long.
///
/// NOTE: This implementation of PackageVersion is stricter than that of libalpm/pacman. It does not
/// allow empty strings `""`, or chars that are not in the allowed set, or `"."` as the first
/// character.
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
/// assert!(PackageVersion::new(".0.1".to_string()).is_err());
/// assert!(PackageVersion::new("_1.0".to_string()).is_err());
/// assert!(PackageVersion::new("+1.0".to_string()).is_err());
/// ```
#[derive(Clone, Debug, Eq, Serialize)]
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
    pub fn segments(&self) -> VersionSegments {
        VersionSegments::new(&self.0)
    }

    /// Recognizes a [`PackageVersion`] in a string slice.
    ///
    /// Consumes all of its input.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is not a valid _alpm-pkgrel_.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        let alnum = |c: char| c.is_ascii_alphanumeric();

        let first_character = one_of(alnum)
            .context(StrContext::Label("first pkgver character"))
            .context(StrContext::Expected(StrContextValue::Description(
                "ASCII alphanumeric character",
            )));
        let tail_character = one_of((alnum, '_', '+', '.'));

        // no error context because this is infallible due to `0..`
        // note the empty tuple collection to avoid allocation
        let tail: Repeat<_, _, _, (), _> = repeat(0.., tail_character);

        (
            first_character,
            tail,
            eof.context(StrContext::Label("pkgver character"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "ASCII alphanumeric character",
                )))
                .context(StrContext::Expected(StrContextValue::CharLiteral('_')))
                .context(StrContext::Expected(StrContextValue::CharLiteral('+')))
                .context(StrContext::Expected(StrContextValue::CharLiteral('.'))),
        )
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
#[derive(Debug, Clone, Eq, PartialEq)]
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

impl Ord for PackageVersion {
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

impl PartialOrd for PackageVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PackageVersion {
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
/// # fn main() -> Result<(), alpm_types::Error> {
/// // create SchemaVersion from str
/// let version_one = SchemaVersion::from_str("1.0.0")?;
/// let version_also_one = SchemaVersion::from_str("1")?;
/// assert_eq!(version_one, version_also_one);
///
/// // format as String
/// assert_eq!("1.0.0", format!("{}", version_one));
/// assert_eq!("1.0.0", format!("{}", version_also_one));
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SchemaVersion(SemverVersion);

impl SchemaVersion {
    /// Create a new SchemaVersion
    pub fn new(version: SemverVersion) -> Self {
        SchemaVersion(version)
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &SemverVersion {
        &self.0
    }
}

impl FromStr for SchemaVersion {
    type Err = Error;
    /// Create a new SchemaVersion from a string
    ///
    /// When providing a non-semver string with only a number (i.e. no minor or patch version), the
    /// number is treated as the major version (e.g. `"23"` -> `"23.0.0"`).
    fn from_str(s: &str) -> Result<SchemaVersion, Self::Err> {
        if !s.contains('.') {
            match s.parse() {
                Ok(major) => Ok(SchemaVersion(SemverVersion::new(major, 0, 0))),
                Err(e) => Err(Error::InvalidInteger {
                    kind: e.kind().clone(),
                }),
            }
        } else {
            match SemverVersion::parse(s) {
                Ok(version) => Ok(SchemaVersion(version)),
                Err(e) => Err(Error::InvalidSemver {
                    kind: e.to_string(),
                }),
            }
        }
    }
}

impl Display for SchemaVersion {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

/// A version of a package
///
/// A `Version` tracks an optional `Epoch`, a `PackageVersion` and an optional `PackageRelease`.
/// See [alpm-package-version] for details on the format.
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
/// assert_eq!(version.pkgrel, Some(PackageRelease::new("3".to_string())?));
/// # Ok(())
/// # }
/// ```
///
/// [alpm-package-version]: https://alpm.archlinux.page/specifications/alpm-package-version.7.html
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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

    /// Create a new Version, which is guaranteed to have a PackageRelease
    pub fn with_pkgrel(version: &str) -> Result<Self, Error> {
        let version = Version::from_str(version)?;
        if version.pkgrel.is_some() {
            Ok(version)
        } else {
            Err(Error::MissingComponent {
                component: "pkgrel",
            })
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
/// ## Note
///
/// The variants of this enum are sorted in a way, that prefers the two-letter comparators over
/// the one-letter ones.
/// This is because when splitting a string on the string representation of [`VersionComparison`]
/// variant and relying on the ordering of [`strum::EnumIter`], the two-letter comparators must be
/// checked before checking the one-letter ones to yield robust results.
#[derive(
    strum::AsRefStr,
    Clone,
    Copy,
    Debug,
    strum::Display,
    strum::EnumIter,
    strum::EnumString,
    PartialEq,
    Eq,
    strum::VariantNames,
    Serialize,
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
}

/// A version requirement, e.g. for a dependency package.
///
/// It consists of a target version and a comparison function. A version requirement of `>=1.5` has
/// a target version of `1.5` and a comparison function of [`VersionComparison::GreaterOrEqual`].
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_satisfied_by(&self, ver: &Version) -> bool {
        self.comparison.is_compatible_with(ver.cmp(&self.version))
    }
}

impl Display for VersionRequirement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.comparison, self.version)
    }
}

impl FromStr for VersionRequirement {
    type Err = Error;

    /// Parses a version requirement from a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the comparison function or version are malformed.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("1.0.0", Ok(SchemaVersion(SemverVersion::new(1, 0, 0))))]
    #[case("1", Ok(SchemaVersion(SemverVersion::new(1, 0, 0))))]
    #[case("-1.0.0", Err(Error::InvalidSemver { kind: String::from("unexpected character '-' while parsing major version number") }))]
    fn schema_version(#[case] version: &str, #[case] result: Result<SchemaVersion, Error>) {
        assert_eq!(result, SchemaVersion::from_str(version))
    }

    /// Ensure that valid buildtool version strings are parsed as expected.
    #[rstest]
    #[case(
        "1.0.0-1-any",
        BuildToolVersion::new(Version::from_str("1.0.0-1").unwrap(), Some(Architecture::from_str("any").unwrap())),
    )]
    #[case(
        "1:1.0.0-1-any",
        BuildToolVersion::new(Version::from_str("1:1.0.0-1").unwrap(), Some(Architecture::from_str("any").unwrap())),
    )]
    #[case(
        "1.0.0",
        BuildToolVersion::new(Version::from_str("1.0.0").unwrap(), None),
    )]
    fn valid_buildtoolver_new(#[case] buildtoolver: &str, #[case] expected: BuildToolVersion) {
        assert_eq!(
            BuildToolVersion::from_str(buildtoolver),
            Ok(expected),
            "Expected valid parse of buildtoolver '{buildtoolver}'"
        );
    }

    /// Ensure that invalid buildtool version strings produce the respective errors.
    #[rstest]
    #[case("1.0.0-any", Error::MissingComponent { component: "pkgrel" })]
    #[case("1.0.0-1-foo", strum::ParseError::VariantNotFound.into())]
    fn invalid_buildtoolver_new(#[case] buildtoolver: &str, #[case] expected: Error) {
        assert_eq!(
            BuildToolVersion::from_str(buildtoolver),
            Err(expected),
            "Expected error during parse of buildtoolver '{buildtoolver}'"
        );
    }

    #[rstest]
    #[case(".1.0.0-1-any", "invalid first pkgver character")]
    fn invalid_buildtoolver_badpkgver(#[case] buildtoolver: &str, #[case] err_snippet: &str) {
        let Err(Error::ParseError(err_msg)) = BuildToolVersion::from_str(buildtoolver) else {
            panic!("'{buildtoolver}' erroneously parsed as BuildToolVersion")
        };
        assert!(
            err_msg.contains(err_snippet),
            "Error:\n=====\n{err_msg}\n=====\nshould contain snippet:\n\n{err_snippet}"
        );
    }

    #[rstest]
    #[case(
        SchemaVersion(SemverVersion::new(1, 0, 0)),
        SchemaVersion(SemverVersion::new(0, 1, 0))
    )]
    fn compare_schema_version(#[case] version_a: SchemaVersion, #[case] version_b: SchemaVersion) {
        assert!(version_a > version_b);
    }

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
            epoch: Some(Epoch::from_str("1").unwrap()),
            pkgrel: Some(PackageRelease::new("1".to_string()).unwrap()),
        },
    )]
    #[case(
        "1:foo",
        Version {
            pkgver: PackageVersion::new("foo".to_string()).unwrap(),
            epoch: Some(Epoch::from_str("1").unwrap()),
            pkgrel: None,
        },
    )]
    #[case(
        "foo-1",
        Version {
            pkgver: PackageVersion::new("foo".to_string()).unwrap(),
            epoch: None,
            pkgrel: Some(PackageRelease::new("1".to_string()).unwrap())
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
    #[case::no_version(":", "invalid first pkgver character")]
    #[case::no_version(".", "invalid first pkgver character")]
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

    /// Test that version parsing works/fails for the special case where a pkgrel is expected.
    /// This is done by calling the `with_pkgrel` function directly.
    #[rstest]
    #[case(
        "1.0.0-1",
        Ok(Version{
            pkgver: PackageVersion::new("1.0.0".to_string()).unwrap(),
            pkgrel: Some(PackageRelease::new("1".to_string()).unwrap()),
            epoch: None,
        })
    )]
    #[case("1.0.0", Err(Error::MissingComponent { component: "pkgrel" }))]
    fn version_with_pkgrel(#[case] version: &str, #[case] result: Result<Version, Error>) {
        assert_eq!(result, Version::with_pkgrel(version));
    }

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
    #[case("foo,1", "invalid pkgver character")]
    #[case(".foo", "invalid first pkgver character")]
    #[case("_foo", "invalid first pkgver character")]
    // ß is not in [:alnum:]
    #[case("ß", "invalid first pkgver character")]
    #[case("1.ß", "invalid pkgver character")]
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
        let parsed = PackageRelease::new(pkgrel.to_string());
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
        let Err(Error::ParseError(err_msg)) = PackageRelease::new(pkgrel.to_string()) else {
            panic!("'{pkgrel}' erroneously parsed as PackageRelease")
        };
        assert!(
            err_msg.contains(err_snippet),
            "Error:\n=====\n{err_msg}\n=====\nshould contain snippet:\n\n{err_snippet}"
        );
    }

    /// Test that pkgrel ordering works as intended
    #[rstest]
    #[case("1", "2")]
    #[case("1", "1.1")]
    #[case("1", "11")]
    fn pkgrel_cmp(#[case] lesser: &str, #[case] bigger: &str) {
        let lesser = PackageRelease::new(lesser.to_string()).unwrap();
        let bigger = PackageRelease::new(bigger.to_string()).unwrap();
        assert!(lesser.lt(&bigger));
    }

    /// Ensure that versions are properly serialized back to their string representation.
    #[rstest]
    #[case(Version::from_str("1:1-1").unwrap(), "1:1-1")]
    #[case(Version::from_str("1-1").unwrap(), "1-1")]
    #[case(Version::from_str("1").unwrap(), "1")]
    #[case(Version::from_str("1:1").unwrap(), "1:1")]
    fn version_to_string(#[case] version: Version, #[case] to_str: &str) {
        assert_eq!(format!("{}", version), to_str);
    }

    #[rstest]
    #[case(Version::from_str("1"), Version::from_str("1"), Ordering::Equal)]
    #[case(Version::from_str("2"), Version::from_str("1"), Ordering::Greater)]
    #[case(Version::from_str("1"), Version::from_str("2"), Ordering::Less)]
    #[case(Version::from_str("1"), Version::from_str("1.1"), Ordering::Less)]
    #[case(Version::from_str("1.1"), Version::from_str("1"), Ordering::Greater)]
    #[case(Version::from_str("1.1"), Version::from_str("1.1"), Ordering::Equal)]
    #[case(Version::from_str("1.2"), Version::from_str("1.1"), Ordering::Greater)]
    #[case(Version::from_str("1.1"), Version::from_str("1.2"), Ordering::Less)]
    #[case(Version::from_str("1+2"), Version::from_str("1+1"), Ordering::Greater)]
    #[case(Version::from_str("1+1"), Version::from_str("1+2"), Ordering::Less)]
    #[case(Version::from_str("1.1"), Version::from_str("1.1a"), Ordering::Greater)]
    #[case(Version::from_str("1.1a"), Version::from_str("1.1"), Ordering::Less)]
    #[case(
        Version::from_str("1.1"),
        Version::from_str("1.1a1"),
        Ordering::Greater
    )]
    #[case(Version::from_str("1.1a1"), Version::from_str("1.1"), Ordering::Less)]
    #[case(Version::from_str("1.1"), Version::from_str("1.11a"), Ordering::Less)]
    #[case(
        Version::from_str("1.11a"),
        Version::from_str("1.1"),
        Ordering::Greater
    )]
    #[case(
        Version::from_str("1.1_a"),
        Version::from_str("1.1"),
        Ordering::Greater
    )]
    #[case(Version::from_str("1.1"), Version::from_str("1.1_a"), Ordering::Less)]
    #[case(Version::from_str("1.1"), Version::from_str("1.1.a"), Ordering::Less)]
    #[case(
        Version::from_str("1.1.a"),
        Version::from_str("1.1"),
        Ordering::Greater
    )]
    #[case(Version::from_str("1.a"), Version::from_str("1.1"), Ordering::Less)]
    #[case(Version::from_str("1.1"), Version::from_str("1.a"), Ordering::Greater)]
    #[case(Version::from_str("1.a1"), Version::from_str("1.1"), Ordering::Less)]
    #[case(Version::from_str("1.1"), Version::from_str("1.a1"), Ordering::Greater)]
    #[case(Version::from_str("1.a11"), Version::from_str("1.1"), Ordering::Less)]
    #[case(
        Version::from_str("1.1"),
        Version::from_str("1.a11"),
        Ordering::Greater
    )]
    #[case(Version::from_str("a.1"), Version::from_str("1.1"), Ordering::Less)]
    #[case(Version::from_str("1.1"), Version::from_str("a.1"), Ordering::Greater)]
    #[case(Version::from_str("foo"), Version::from_str("1.1"), Ordering::Less)]
    #[case(Version::from_str("1.1"), Version::from_str("foo"), Ordering::Greater)]
    #[case(Version::from_str("a1a"), Version::from_str("a1b"), Ordering::Less)]
    #[case(Version::from_str("a1b"), Version::from_str("a1a"), Ordering::Greater)]
    #[case(
        Version::from_str("20220102"),
        Version::from_str("20220202"),
        Ordering::Less
    )]
    #[case(
        Version::from_str("20220202"),
        Version::from_str("20220102"),
        Ordering::Greater
    )]
    #[case(Version::from_str("1.0.."), Version::from_str("1.0."), Ordering::Equal)]
    #[case(Version::from_str("1.0."), Version::from_str("1.0"), Ordering::Greater)]
    #[case(Version::from_str("1..0"), Version::from_str("1.0"), Ordering::Greater)]
    #[case(Version::from_str("1..0"), Version::from_str("1..0"), Ordering::Equal)]
    #[case(
        Version::from_str("1..1"),
        Version::from_str("1..0"),
        Ordering::Greater
    )]
    #[case(Version::from_str("1..0"), Version::from_str("1..1"), Ordering::Less)]
    #[case(Version::from_str("1+0"), Version::from_str("1.0"), Ordering::Equal)]
    #[case(
        Version::from_str("1.111"),
        Version::from_str("1.1a1"),
        Ordering::Greater
    )]
    #[case(Version::from_str("1.1a1"), Version::from_str("1.111"), Ordering::Less)]
    #[case(Version::from_str("01"), Version::from_str("1"), Ordering::Equal)]
    #[case(Version::from_str("001a"), Version::from_str("1a"), Ordering::Equal)]
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
            Err(strum::ParseError::VariantNotFound)
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

    /// Test expected parsing errors for version requirement strings.
    #[rstest]
    #[case("<=", Error::MissingComponent { component: "operator" })]
    #[case("<>3.1", strum::ParseError::VariantNotFound.into())]
    #[case("3.1", strum::ParseError::VariantNotFound.into())]
    #[case("=>3.1", strum::ParseError::VariantNotFound.into())]
    fn invalid_version_requirement(#[case] requirement: &str, #[case] expected: Error) {
        assert_eq!(
            requirement.parse::<VersionRequirement>(),
            Err(expected),
            "Expected error while parsing version requirement '{requirement}'"
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
        let version = PackageVersion(version.to_string());
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
