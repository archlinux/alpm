// SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
// SPDX-License-Identifier: LGPL-3.0-or-later
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

/// The Error that can occur when using types
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// An invalid build date (in seconds since the epoch)
    InvalidBuildDate(String),
    /// An invalid compressed file size (in bytes)
    InvalidCompressedSize(String),
    /// An invalid installed package size (in bytes)
    InvalidInstalledSize(String),
    /// an unknown CPU architecture has been encountered
    UnknownArchitecture(String),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        write!(
            fmt,
            "{}",
            match self {
                Error::InvalidBuildDate(reason) => format!("Invalid build date: {}", reason),
                Error::InvalidCompressedSize(reason) =>
                    format!("Invalid compressed size: {}", reason),
                Error::InvalidInstalledSize(reason) =>
                    format!("Invalid installed size: {}", reason),
                Error::UnknownArchitecture(reason) =>
                    format!("Invalid CPU architecture: {}", reason),
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_format_string() {
        assert_eq!(
            "Invalid CPU architecture: foo",
            format!("{}", Error::UnknownArchitecture(String::from("foo")))
        );
        assert_eq!(
            "Invalid build date: -1",
            format!("{}", Error::InvalidBuildDate(String::from("-1")))
        );
        assert_eq!(
            "Invalid compressed size: -1",
            format!("{}", Error::InvalidCompressedSize(String::from("-1")))
        );
        assert_eq!(
            "Invalid installed size: -1",
            format!("{}", Error::InvalidInstalledSize(String::from("-1")))
        );
    }
}
