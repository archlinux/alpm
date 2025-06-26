use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
    iter::Peekable,
    num::NonZeroUsize,
    str::{CharIndices, Chars, FromStr},
};

use alpm_parsers::{iter_char_context, iter_str_context};
use semver::Version as SemverVersion;
use serde::{Deserialize, Serialize};
use strum::VariantNames;
use winnow::{
    ModalResult,
    Parser,
    ascii::{dec_uint, digit1},
    combinator::{Repeat, alt, cut_err, eof, fail, opt, preceded, repeat, seq, terminated},
    error::{StrContext, StrContextValue},
    token::{one_of, take_till, take_while},
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
        let special_tail_character = ['_', '+', '.'];
        let tail_character = one_of((alnum, special_tail_character));

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
                .context_with(iter_char_context!(special_tail_character)),
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

/// This enum represents a single segment in a version string.
/// [`VersionSegment`]s are returned by the [`VersionSegments`] iterator, which is responsible for
/// splitting a version string into its segments.
///
/// Version strings are split according to the following rules:
///
/// - Non-alphanumeric characters always count as delimiters (`.`, `-`, `$`, etc.).
/// - There's no differentiation between delimiters represented by different characters (e.g. `'$$$'
///   == '...' == '.$-'`).
/// - Each segment contains the info about the amount of leading delimiters for that segment.
///   Leading delimiters that directly follow after one another are grouped together. The length of
///   the delimiters is important, as it plays a crucial role in the algorithm that determines which
///   version is newer.
///
///   `1...a` would be represented as:
///
///   ```
///   use alpm_types::VersionSegment::*;
///   vec![
///     Segment { text: "1", delimiter_count: 0},
///     Segment { text: "a", delimiter_count: 3},
///   ];
///   ```
/// - Alphanumeric strings are also split into individual sub-segments. This is done by walking over
///   the string and splitting it every time a switch from alphabetic to numeric is detected or vice
///   versa.
///
///   `1.1foo123.0` would be represented as:
///
///   ```
///   use alpm_types::VersionSegment::*;
///   vec![
///     Segment { text: "1", delimiter_count: 0},
///     Segment { text: "1", delimiter_count: 1},
///     SubSegment { text: "foo" },
///     SubSegment { text: "123" },
///     Segment { text: "0", delimiter_count: 1},
///   ];
///   ```
/// - Trailing delimiters are encoded as an empty string.
///
///   `1...` would be represented as:
///
///   ```
///   use alpm_types::VersionSegment::*;
///   vec![
///     Segment { text: "1", delimiter_count: 0},
///     Segment { text: "", delimiter_count: 3},
///   ];
///   ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VersionSegment<'a> {
    /// The start of a new segment.
    /// If the current segment can be split into multiple sub-segments, this variant only contains
    /// the **first** sub-segment.
    ///
    /// To figure out whether this is sub-segment, peek at the next element in the
    /// [`VersionSegments`] iterator, whether it's a [`VersionSegment::SubSegment`].
    Segment {
        /// The string representation of this segment
        text: &'a str,
        /// The amount of leading delimiters that were found for this segment
        delimiter_count: usize,
    },
    /// A sub-segment of a version string's segment.
    ///
    /// Note that the first sub-segment of a segment that can be split into sub-segments is
    /// counterintuitively represented by [VersionSegment::Segment]. This implementation detail
    /// is due to the way the comparison algorithm works, as it does not always differentiate
    /// between segments and sub-segments.
    SubSegment {
        /// The string representation of this sub-segment
        text: &'a str,
    },
}

impl<'a> VersionSegment<'a> {
    /// Returns the inner string slice independent of [`VersionSegment`] variant.
    pub fn text(&self) -> &str {
        match self {
            VersionSegment::Segment { text, .. } | VersionSegment::SubSegment { text } => text,
        }
    }

    /// Returns whether the inner string slice is empty, independent of [`VersionSegment`] variant
    pub fn is_empty(&self) -> bool {
        match self {
            VersionSegment::Segment { text, .. } | VersionSegment::SubSegment { text } => {
                text.is_empty()
            }
        }
    }

    /// Returns an iterator over the chars of the inner string slice.
    pub fn chars(&self) -> Chars<'a> {
        match self {
            VersionSegment::Segment { text, .. } | VersionSegment::SubSegment { text } => {
                text.chars()
            }
        }
    }

    /// Creates a type `T` from the inner string slice by relying on `T`'s [`FromStr::from_str`]
    /// implementation.
    pub fn parse<T: FromStr>(&self) -> Result<T, T::Err> {
        match self {
            VersionSegment::Segment { text, .. } | VersionSegment::SubSegment { text } => {
                FromStr::from_str(text)
            }
        }
    }

    /// Compares the inner string slice with that of another [`VersionSegment`].
    pub fn str_cmp(&self, other: &VersionSegment) -> Ordering {
        match self {
            VersionSegment::Segment { text, .. } | VersionSegment::SubSegment { text } => {
                text.cmp(&other.text())
            }
        }
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
#[derive(Debug)]
pub struct VersionSegments<'a> {
    /// The original version string. We need that reference so we can get some string
    /// slices based on indices later on.
    version: &'a str,
    /// An iterator over the version's chars and their respective start byte's index.
    version_chars: Peekable<CharIndices<'a>>,
    /// Check if the cursor is currently in a segment.
    /// This is necessary to detect whether the next segment should be a sub-segment or a new
    /// segment.
    in_segment: bool,
}

impl<'a> VersionSegments<'a> {
    /// Create a new instance of a VersionSegments iterator.
    pub fn new(version: &'a str) -> Self {
        VersionSegments {
            version,
            version_chars: version.char_indices().peekable(),
            in_segment: false,
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

            // As soon as we hit a delimiter, we know that a new segment is about to start.
            self.in_segment = false;
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
            return Some(VersionSegment::Segment {
                text: "",
                delimiter_count,
            });
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

        if !self.in_segment {
            // Any further segments should be sub-segments, unless we hit a delimiter in which
            // case this variable will reset to false.
            self.in_segment = true;
            Some(VersionSegment::Segment {
                text: segment_slice,
                delimiter_count,
            })
        } else {
            Some(VersionSegment::SubSegment {
                text: segment_slice,
            })
        }
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

        let mut self_segments = self.segments();
        let mut other_segments = other.segments();

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

                // One version is longer than the other and both are equal until now.
                //
                // ## Case 1
                //
                // The longer version is one or more **segment**s longer.
                // In this case, the longer version is always considered newer.
                //   `1.0` > `1`
                // `1.0.0` > `1.0`
                // `1.0.a` > `1.0`
                //     ⤷ New segment exists, thereby newer
                //
                // ## Case 2
                //
                // The current **segment** has one or more sub-segments and the next sub-segment is
                // alphabetic.
                // In this case, the shorter version is always newer.
                // The reason for this is to handle pre-releases (e.g. alpha/beta).
                // `1.0alpha` < `1.0`
                // `1.0alpha.0` < `1.0`
                // `1.0alpha12.0` < `1.0`
                //     ⤷ Next sub-segment is alphabetic.
                //
                // ## Case 3
                //
                // The current **segment** has one or more sub-segments and the next sub-segment is
                // numeric. In this case, the longer version is always newer.
                // `1.alpha0` > `1.alpha`
                // `1.alpha0.1` > `1.alpha`
                //         ⤷ Next sub-segment is numeric.
                (Some(seg), None) => {
                    // If the current segment is the start of a segment, it's always considered
                    // newer.
                    let text = match seg {
                        VersionSegment::Segment { .. } => return Ordering::Greater,
                        VersionSegment::SubSegment { text } => text,
                    };

                    // If it's a sub-segment, we have to check for the edge-case explained above
                    // If all chars are alphabetic, `self` is consider older.
                    if !text.is_empty() && text.chars().all(char::is_alphabetic) {
                        return Ordering::Less;
                    }

                    return Ordering::Greater;
                }

                // This is the same logic as above, but inverted.
                (None, Some(seg)) => {
                    let text = match seg {
                        VersionSegment::Segment { .. } => return Ordering::Less,
                        VersionSegment::SubSegment { text } => text,
                    };
                    if !text.is_empty() && text.chars().all(char::is_alphabetic) {
                        return Ordering::Greater;
                    }
                    if !text.is_empty() && text.chars().all(char::is_alphabetic) {
                        return Ordering::Greater;
                    }

                    return Ordering::Less;
                }
            };

            // At this point, we have two sub-/segments.
            //
            // We start with the special case where one or both of the segments are empty.
            // That means that the end of the version string has been reached, but there were one
            // or more trailing delimiters, e.g.:
            //
            // `1.0.`
            // `1.0...`
            if other_segment.is_empty() && self_segment.is_empty() {
                // Both reached the end of their version with a trailing delimiter.
                // Counterintuitively, the trailing delimiter count is not considered and both
                // versions are considered equal
                // `1.0....` == `1.0.`
                //       ⤷ Length of delimiters is ignored.
                return Ordering::Equal;
            } else if self_segment.is_empty() {
                // Now we have to consider the special case where `other` is alphabetic.
                // If that's the case, `self` will be considered newer, as the alphabetic string
                // indicates a pre-release,
                // `1.0.` > `1.0alpha0`
                // `1.0.` > `1.0.alpha.0`
                //                ⤷ Alphabetic sub-/segment and thereby always older.
                //
                // Also, we know that `other_segment` isn't empty at this point.
                // It's noteworthy that this logic does not differentiated between segments and
                // sub-segments.
                if other_segment.chars().all(char::is_alphabetic) {
                    return Ordering::Greater;
                }

                // In all other cases, `other` is newer.
                // `1.0.` < `1.0.0`
                // `1.0.` < `1.0.2.0`
                return Ordering::Less;
            } else if other_segment.is_empty() {
                // Check docs above, as it's the same logic as above, just inverted.
                if self_segment.chars().all(char::is_alphabetic) {
                    return Ordering::Less;
                }

                return Ordering::Greater;
            }

            // We finally reached the end handling special cases when the version string ended.
            // From now on, we know that we have two actual sub-/segments that might be prefixed by
            // some delimiters.
            //
            // However, it is possible that one version has a segment and while the other has a
            // sub-segment. This special case is what is handled next.
            //
            // We purposefully give up ownership of both segments.
            // This is to ensure that following this match block, we finally only have to
            // consider the actual text of the segments, as we'll know that both sub-/segments are
            // of the same type.
            let (self_text, other_text) = match (self_segment, other_segment) {
                (
                    VersionSegment::Segment {
                        delimiter_count: self_count,
                        text: self_text,
                    },
                    VersionSegment::Segment {
                        delimiter_count: other_count,
                        text: other_text,
                    },
                ) => {
                    // Special case:
                    // If one of the segments has more leading delimiters than the other, it is
                    // always considered newer, no matter what follows after the delimiters.
                    // `1..0.0` > `1.2.0`
                    //    ⤷ Two delimiters, thereby always newer.
                    // `1..0.0` < `1..2.0`
                    //               ⤷ Same amount of delimiters, now `2 > 0`
                    if self_count != other_count {
                        return self_count.cmp(&other_count);
                    }
                    (self_text, other_text)
                }
                // If one is the start of a new segment, while the other is still a sub-segment,
                // we can return early as a new segment always overrules a sub-segment.
                // `1.alpha0.0` < `1.alpha.0`
                //         ⤷ sub-segment  ⤷ segment
                //         In the third iteration there's a sub-segment on the left side while
                //         there's a segment on the right side.
                (VersionSegment::Segment { .. }, VersionSegment::SubSegment { .. }) => {
                    return Ordering::Greater;
                }
                (VersionSegment::SubSegment { .. }, VersionSegment::Segment { .. }) => {
                    return Ordering::Less;
                }
                (
                    VersionSegment::SubSegment { text: self_text },
                    VersionSegment::SubSegment { text: other_text },
                ) => (self_text, other_text),
            };

            // At this point, we know that we are dealing with two identical types of sub-/segments.
            // Thereby, we now only have to compare the text of those sub-/segments.

            // Check whether any of the texts are numeric.
            // Numeric sub-/segments are always considered newer than non-numeric sub-/segments.
            // E.g.: `1.0.0` > `1.foo.0`
            //          ⤷ `0` vs `foo`.
            //            `0` is numeric and therebynewer than a alphanumeric one.
            let self_is_numeric = !self_text.is_empty() && self_text.chars().all(char::is_numeric);
            let other_is_numeric =
                !other_text.is_empty() && other_text.chars().all(char::is_numeric);

            if self_is_numeric && !other_is_numeric {
                return Ordering::Greater;
            } else if !self_is_numeric && other_is_numeric {
                return Ordering::Less;
            } else if self_is_numeric && other_is_numeric {
                // In case both are numeric, we do a number comparison.
                // We can parse the string as we know that they only consist of digits, hence the
                // unwrap.
                //
                // Preceding zeroes are to be ignored, which is automatically done by Rust's number
                // parser.
                // E.g. `1.0001.1` == `1.1.1`
                //          ⤷ `000` is ignored in the comparison.
                let ordering = self_text
                    .parse::<usize>()
                    .unwrap()
                    .cmp(&other_text.parse::<usize>().unwrap());

                match ordering {
                    Ordering::Less => return Ordering::Less,
                    Ordering::Greater => return Ordering::Greater,
                    // If both numbers are equal we check the next sub-/segment.
                    Ordering::Equal => continue,
                }
            }

            // At this point, we know that both sub-/segments are alphabetic.
            // We do a simple string comparison to determine the newer version.
            match self_text.cmp(other_text) {
                Ordering::Less => return Ordering::Less,
                Ordering::Greater => return Ordering::Greater,
                // If the strings are equal, we check the next sub-/segment.
                Ordering::Equal => continue,
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
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_satisfied_by(&self, ver: &Version) -> bool {
        self.comparison.is_compatible_with(ver.cmp(&self.version))
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
            pkgrel: Some(PackageRelease::new(1, None)),
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
    #[case("1.0.0", vec![
        VersionSegment::Segment{ text:"1", delimiter_count: 0},
        VersionSegment::Segment{ text:"0", delimiter_count: 1},
        VersionSegment::Segment{ text:"0", delimiter_count: 1},
    ])]
    #[case("1..0", vec![
        VersionSegment::Segment{ text:"1", delimiter_count: 0},
        VersionSegment::Segment{ text:"0", delimiter_count: 2},
    ])]
    #[case("1.0.", vec![
        VersionSegment::Segment{ text:"1", delimiter_count: 0},
        VersionSegment::Segment{ text:"0", delimiter_count: 1},
        VersionSegment::Segment{ text:"", delimiter_count: 1},
    ])]
    #[case("1..", vec![
        VersionSegment::Segment{ text:"1", delimiter_count: 0},
        VersionSegment::Segment{ text:"", delimiter_count: 2},
    ])]
    #[case("1...", vec![
        VersionSegment::Segment{ text:"1", delimiter_count: 0},
        VersionSegment::Segment{ text:"", delimiter_count: 3},
    ])]
    #[case("1.🗻lol.0", vec![
        VersionSegment::Segment{ text:"1", delimiter_count: 0},
        VersionSegment::Segment{ text:"lol", delimiter_count: 2},
        VersionSegment::Segment{ text:"0", delimiter_count: 1},
    ])]
    #[case("1.🗻lol.", vec![
        VersionSegment::Segment{ text:"1", delimiter_count: 0},
        VersionSegment::Segment{ text:"lol", delimiter_count: 2},
        VersionSegment::Segment{ text:"", delimiter_count: 1},
    ])]
    #[case("20220202", vec![
        VersionSegment::Segment{ text:"20220202", delimiter_count: 0},
    ])]
    #[case("some_string", vec![
        VersionSegment::Segment{ text:"some", delimiter_count: 0},
        VersionSegment::Segment{ text:"string", delimiter_count: 1}
    ])]
    #[case("alpha7654numeric321", vec![
        VersionSegment::Segment{ text:"alpha", delimiter_count: 0},
        VersionSegment::SubSegment{ text:"7654"},
        VersionSegment::SubSegment{ text:"numeric"},
        VersionSegment::SubSegment{ text:"321"},
    ])]
    fn version_segment_iterator(
        #[case] version: &str,
        #[case] expected_segments: Vec<VersionSegment>,
    ) {
        let version = PackageVersion(version.to_string());
        // Convert the simplified definition above into actual VersionSegment instances.
        let mut segments_iter = version.segments();
        let mut expected_iter = expected_segments.clone().into_iter();

        // Iterate over both iterators.
        // We do it manually to ensure that they both end at the same time.
        loop {
            let next_segment = segments_iter.next();
            assert_eq!(
                next_segment,
                expected_iter.next(),
                "Failed for segment {next_segment:?} in version string {version}:\nsegments: {:?}\n expected: {:?}",
                version.segments().collect::<Vec<VersionSegment>>(),
                expected_segments,
            );
            if next_segment.is_none() {
                break;
            }
        }
    }
}
