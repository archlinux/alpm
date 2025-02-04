use time::OffsetDateTime;

/// A build date in seconds since the epoch
///
/// This is a type alias for [`i64`].
///
/// # Examples
/// ```
/// use std::{num::IntErrorKind, str::FromStr};
///
/// use alpm_types::{BuildDate, Error, FromOffsetDateTime};
/// use time::OffsetDateTime;
///
/// // create BuildDate from OffsetDateTime
/// let datetime = BuildDate::from_offset_datetime(OffsetDateTime::from_unix_timestamp(1).unwrap());
/// assert_eq!(1, datetime);
///
/// // create BuildDate from &str
/// assert_eq!(BuildDate::from_str("1"), Ok(1));
/// assert!(BuildDate::from_str("foo").is_err());
/// ```
pub type BuildDate = i64;

/// A trait for allowing conversion from an [`OffsetDateTime`] to a type.
pub trait FromOffsetDateTime {
    /// Converts an [`OffsetDateTime`] into a type.
    fn from_offset_datetime(input: OffsetDateTime) -> Self;
}

impl FromOffsetDateTime for BuildDate {
    /// Converts a [`OffsetDateTime`] into a [`BuildDate`].
    ///
    /// Uses the unix timestamp of the [`OffsetDateTime`].
    fn from_offset_datetime(input: OffsetDateTime) -> Self {
        input.unix_timestamp()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn datetime_into_builddate() {
        let builddate = 1;
        let offset_datetime = OffsetDateTime::from_unix_timestamp(1).unwrap();
        let datetime: BuildDate = BuildDate::from_offset_datetime(offset_datetime);
        assert_eq!(builddate, datetime);
    }
}
