//! Comparison for [`PackageVersion`].
//!
//! This module implements the behavior of the [rpmvercmp algorithm in RPM version 4.8.1].
//! The current implementation is based on the [alpm-pkgver] specification and explicitly
//! handles undefined and undocumented behavior observed from `rpmvercmp`.
//!
//! The current version, including its improvements over the reference implementation are described
//! and specified in detail in [alpm-pkgver].
//!
//! [alpm-pkgver]: https://alpm.archlinux.page/specifications/alpm-pkgver.7.html
//! [rpmvercmp algorithm in RPM version 4.8.1]: https://github.com/rpm-software-management/rpm/blob/rpm-4.8.1-release/lib/rpmvercmp.c

use std::{
    cmp::Ordering,
    iter::Peekable,
    str::{CharIndices, Chars, FromStr},
};

use crate::PackageVersion;

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
    /// This logic is surprisingly complex as it mirrors the current C-alpmlib implementation's
    /// behavior for backwards compatibility reasons.
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

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
