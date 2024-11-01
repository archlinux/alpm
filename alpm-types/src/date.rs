use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use time::OffsetDateTime;

use crate::error::Error;

/// A build date in seconds since the epoch
///
/// # Examples
/// ```
/// use std::num::IntErrorKind;
/// use std::str::FromStr;
///
/// use alpm_types::{BuildDate, Error};
/// use time::OffsetDateTime;
///
/// // create BuildDate from OffsetDateTime
/// let datetime: BuildDate = OffsetDateTime::from_unix_timestamp(1).unwrap().into();
/// assert_eq!(BuildDate::new(1), datetime);
///
/// // create BuildDate from &str
/// assert_eq!(BuildDate::from_str("1"), Ok(BuildDate::new(1)));
/// assert_eq!(
///     BuildDate::from_str("foo"),
///     Err(Error::InvalidInteger {
///         kind: IntErrorKind::InvalidDigit
///     })
/// );
///
/// // format as String
/// assert_eq!("1", format!("{}", BuildDate::new(1)));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildDate(i64);

impl BuildDate {
    /// Create a new BuildDate
    pub fn new(builddate: i64) -> BuildDate {
        BuildDate(builddate)
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &i64 {
        &self.0
    }
}

impl From<OffsetDateTime> for BuildDate {
    fn from(input: OffsetDateTime) -> BuildDate {
        let builddate = input.unix_timestamp();
        BuildDate(builddate)
    }
}

impl FromStr for BuildDate {
    type Err = Error;
    /// Create a BuildDate from a string
    fn from_str(input: &str) -> Result<BuildDate, Self::Err> {
        match input.parse::<i64>() {
            Ok(builddate) => Ok(BuildDate(builddate)),
            Err(e) => Err(Error::InvalidInteger {
                kind: e.kind().clone(),
            }),
        }
    }
}

impl Display for BuildDate {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

#[cfg(test)]
mod tests {
    use std::num::IntErrorKind;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("1", Ok(BuildDate(1)))]
    #[case(
        "foo",
        Err(Error::InvalidInteger { kind: IntErrorKind::InvalidDigit })
    )]
    fn builddate_from_string(#[case] from_str: &str, #[case] result: Result<BuildDate, Error>) {
        assert_eq!(BuildDate::from_str(from_str), result);
    }

    #[rstest]
    fn builddate_format_string() {
        assert_eq!("1", format!("{}", BuildDate::new(1)));
    }

    #[rstest]
    fn datetime_into_builddate() {
        let builddate = BuildDate(1);
        let datetime: BuildDate = OffsetDateTime::from_unix_timestamp(1).unwrap().into();
        assert_eq!(builddate, datetime);
    }
}
