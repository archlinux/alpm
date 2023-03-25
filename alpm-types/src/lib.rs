// SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
// SPDX-License-Identifier: LGPL-3.0-or-later
use chrono::DateTime;
use chrono::Utc;

use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;
use std::string::ToString;

use strum_macros::Display;
use strum_macros::EnumString;

mod error;
pub use error::Error;

/// CPU architecture
///
/// Members of the Architecture enum can be created from `&str`.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
/// use alpm_types::Architecture;
///
/// // create Architecture from str
/// assert_eq!(Architecture::from_str("aarch64"), Ok(Architecture::Aarch64));
///
/// // format as String
/// assert_eq!("aarch64", format!("{}", Architecture::Aarch64));
/// assert_eq!("any", format!("{}", Architecture::Any));
/// assert_eq!("arm", format!("{}", Architecture::Arm));
/// assert_eq!("armv6h", format!("{}", Architecture::Armv6h));
/// assert_eq!("armv7h", format!("{}", Architecture::Armv7h));
/// assert_eq!("i486", format!("{}", Architecture::I486));
/// assert_eq!("i686", format!("{}", Architecture::I686));
/// assert_eq!("pentium4", format!("{}", Architecture::Pentium4));
/// assert_eq!("riscv32", format!("{}", Architecture::Riscv32));
/// assert_eq!("riscv64", format!("{}", Architecture::Riscv64));
/// assert_eq!("x86_64", format!("{}", Architecture::X86_64));
/// assert_eq!("x86_64_v2", format!("{}", Architecture::X86_64V2));
/// assert_eq!("x86_64_v3", format!("{}", Architecture::X86_64V3));
/// assert_eq!("x86_64_v4", format!("{}", Architecture::X86_64V4));
/// ```
#[derive(Debug, Display, EnumString, Eq, PartialEq)]
#[non_exhaustive]
pub enum Architecture {
    #[strum(to_string = "aarch64")]
    Aarch64,
    #[strum(to_string = "any")]
    Any,
    #[strum(to_string = "arm")]
    Arm,
    #[strum(to_string = "armv6h")]
    Armv6h,
    #[strum(to_string = "armv7h")]
    Armv7h,
    #[strum(to_string = "i486")]
    I486,
    #[strum(to_string = "i686")]
    I686,
    #[strum(to_string = "pentium4")]
    Pentium4,
    #[strum(to_string = "riscv32")]
    Riscv32,
    #[strum(to_string = "riscv64")]
    Riscv64,
    #[strum(to_string = "x86_64")]
    X86_64,
    #[strum(to_string = "x86_64_v2")]
    X86_64V2,
    #[strum(to_string = "x86_64_v3")]
    X86_64V3,
    #[strum(to_string = "x86_64_v4")]
    X86_64V4,
}

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
pub struct BuildDate {
    date: i64,
}

impl BuildDate {
    /// Create a new BuildDate
    pub fn new(date: i64) -> BuildDate {
        BuildDate { date }
    }
}

impl From<DateTime<Utc>> for BuildDate {
    fn from(input: DateTime<Utc>) -> BuildDate {
        let date = input.timestamp();
        BuildDate { date }
    }
}

impl FromStr for BuildDate {
    type Err = Error;
    /// Create a BuildDate from a string
    fn from_str(input: &str) -> Result<BuildDate, Self::Err> {
        match input.parse::<i64>() {
            Ok(date) => Ok(BuildDate { date }),
            _ => Err(Error::InvalidBuildDate(input.to_string())),
        }
    }
}

impl Display for BuildDate {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.date)
    }
}

/// Compressed size of a file (in bytes)
///
/// ## Examples
/// ```
/// use alpm_types::{CompressedSize, Error};
/// use std::str::FromStr;
///
/// // create CompressedSize from &str
/// assert_eq!(
///     CompressedSize::from_str("1"),
///     Ok(CompressedSize::new(1))
/// );
/// assert_eq!(
///     CompressedSize::from_str("-1"),
///     Err(Error::InvalidCompressedSize(String::from("-1")))
/// );
///
/// // format as String
/// assert_eq!("1", format!("{}", CompressedSize::new(1)));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompressedSize {
    size: u64,
}

impl CompressedSize {
    /// Create a new CompressedSize
    pub fn new(size: u64) -> CompressedSize {
        CompressedSize { size }
    }
}

impl FromStr for CompressedSize {
    type Err = Error;
    /// Create a CompressedSize from a string
    fn from_str(input: &str) -> Result<CompressedSize, Self::Err> {
        match input.parse::<u64>() {
            Ok(size) => Ok(CompressedSize { size }),
            _ => Err(Error::InvalidCompressedSize(input.to_string())),
        }
    }
}

impl Display for CompressedSize {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.size)
    }
}

/// Installed size of a package (in bytes)
///
/// ## Examples
/// ```
/// use alpm_types::{InstalledSize, Error};
/// use std::str::FromStr;
///
/// // create InstalledSize from &str
/// assert_eq!(InstalledSize::from_str("1"), Ok(InstalledSize::new(1)));
/// assert_eq!(
///     InstalledSize::from_str("-1"),
///     Err(Error::InvalidInstalledSize(String::from("-1")))
/// );
///
/// // format as String
/// assert_eq!("1", format!("{}", InstalledSize::new(1)));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InstalledSize {
    size: u64,
}

impl InstalledSize {
    /// Create a new InstalledSize
    pub fn new(size: u64) -> InstalledSize {
        InstalledSize { size }
    }
}

impl FromStr for InstalledSize {
    type Err = Error;
    /// Create a InstalledSize from a string
    fn from_str(input: &str) -> Result<InstalledSize, Self::Err> {
        match input.parse::<u64>() {
            Ok(size) => Ok(InstalledSize { size }),
            _ => Err(Error::InvalidInstalledSize(input.to_string())),
        }
    }
}

impl Display for InstalledSize {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use strum::ParseError;

    #[test]
    fn architecture_from_string() {
        assert_eq!(Architecture::from_str("aarch64"), Ok(Architecture::Aarch64));
        assert_eq!(Architecture::from_str("any"), Ok(Architecture::Any));
        assert_eq!(Architecture::from_str("arm"), Ok(Architecture::Arm));
        assert_eq!(Architecture::from_str("armv6h"), Ok(Architecture::Armv6h));
        assert_eq!(Architecture::from_str("armv7h"), Ok(Architecture::Armv7h));
        assert_eq!(Architecture::from_str("i486"), Ok(Architecture::I486));
        assert_eq!(Architecture::from_str("i686"), Ok(Architecture::I686));
        assert_eq!(
            Architecture::from_str("pentium4"),
            Ok(Architecture::Pentium4)
        );
        assert_eq!(Architecture::from_str("riscv32"), Ok(Architecture::Riscv32));
        assert_eq!(Architecture::from_str("riscv64"), Ok(Architecture::Riscv64));
        assert_eq!(Architecture::from_str("x86_64"), Ok(Architecture::X86_64));
        assert_eq!(
            Architecture::from_str("x86_64_v2"),
            Ok(Architecture::X86_64V2)
        );
        assert_eq!(
            Architecture::from_str("x86_64_v3"),
            Ok(Architecture::X86_64V3)
        );
        assert_eq!(
            Architecture::from_str("x86_64_v4"),
            Ok(Architecture::X86_64V4)
        );
        assert_eq!(
            Architecture::from_str("foo"),
            Err(ParseError::VariantNotFound)
        );
    }

    #[test]
    fn architecture_format_string() {
        assert_eq!("aarch64", format!("{}", Architecture::Aarch64));
        assert_eq!("any", format!("{}", Architecture::Any));
        assert_eq!("arm", format!("{}", Architecture::Arm));
        assert_eq!("armv6h", format!("{}", Architecture::Armv6h));
        assert_eq!("armv7h", format!("{}", Architecture::Armv7h));
        assert_eq!("i486", format!("{}", Architecture::I486));
        assert_eq!("i686", format!("{}", Architecture::I686));
        assert_eq!("pentium4", format!("{}", Architecture::Pentium4));
        assert_eq!("riscv32", format!("{}", Architecture::Riscv32));
        assert_eq!("riscv64", format!("{}", Architecture::Riscv64));
        assert_eq!("x86_64", format!("{}", Architecture::X86_64));
        assert_eq!("x86_64_v2", format!("{}", Architecture::X86_64V2));
        assert_eq!("x86_64_v3", format!("{}", Architecture::X86_64V3));
        assert_eq!("x86_64_v4", format!("{}", Architecture::X86_64V4));
    }

    #[test]
    fn builddate_from_string() {
        assert_eq!(BuildDate::from_str("1"), Ok(BuildDate { date: 1 }));
        assert_eq!(
            BuildDate::from_str("foo"),
            Err(Error::InvalidBuildDate(String::from("foo")))
        );
    }

    #[test]
    fn builddate_format_string() {
        assert_eq!("1", format!("{}", BuildDate::new(1)));
    }

    #[test]
    fn datetime_into_builddate() {
        let builddate = BuildDate { date: 1 };
        let datetime: BuildDate =
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(1, 0).unwrap(), Utc).into();
        assert_eq!(builddate, datetime);
    }

    #[test]
    fn compressedsize_from_string() {
        assert_eq!(CompressedSize::from_str("1"), Ok(CompressedSize::new(1)));
        assert_eq!(
            CompressedSize::from_str("-1"),
            Err(Error::InvalidCompressedSize(String::from("-1")))
        );
    }

    #[test]
    fn compressedsize_format_string() {
        assert_eq!("1", format!("{}", CompressedSize::new(1)));
    }

    #[test]
    fn installedsize_from_string() {
        assert_eq!(InstalledSize::from_str("1"), Ok(InstalledSize::new(1)));
        assert_eq!(
            InstalledSize::from_str("-1"),
            Err(Error::InvalidInstalledSize(String::from("-1")))
        );
    }

    #[test]
    fn installedsize_format_string() {
        assert_eq!("1", format!("{}", InstalledSize::new(1)));
    }
}
