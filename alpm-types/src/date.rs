// SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
// SPDX-License-Identifier: LGPL-3.0-or-later
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;
use std::string::ToString;

use chrono::DateTime;
use chrono::Utc;

use crate::Error;

/// A build date in seconds since the epoch
///
/// # Examples
/// ```
/// use alpm_types::{BuildDate, Error};
/// use chrono::{DateTime, NaiveDateTime, Utc};
/// use std::str::FromStr;
///
/// // create BuildDate from DateTime<Utc>
/// let datetime: BuildDate =
/// DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(1, 0).unwrap(), Utc).into();
/// assert_eq!(BuildDate::new(1), datetime);
///
/// // create BuildDate from &str
/// assert_eq!(BuildDate::from_str("1"), Ok(BuildDate::new(1)));
/// assert_eq!(
///     BuildDate::from_str("foo"),
///     Err(Error::InvalidBuildDate(String::from("foo")))
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

impl From<DateTime<Utc>> for BuildDate {
    fn from(input: DateTime<Utc>) -> BuildDate {
        let builddate = input.timestamp();
        BuildDate(builddate)
    }
}

impl FromStr for BuildDate {
    type Err = Error;
    /// Create a BuildDate from a string
    fn from_str(input: &str) -> Result<BuildDate, Self::Err> {
        match input.parse::<i64>() {
            Ok(builddate) => Ok(BuildDate(builddate)),
            _ => Err(Error::InvalidBuildDate(input.to_string())),
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
    use super::*;
    use chrono::NaiveDateTime;
    use rstest::rstest;

    #[rstest]
    #[case("1", Ok(BuildDate(1)))]
    #[case("foo", Err(Error::InvalidBuildDate(String::from("foo"))))]
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
        let datetime: BuildDate =
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(1, 0).unwrap(), Utc).into();
        assert_eq!(builddate, datetime);
    }
}
