// SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
// SPDX-License-Identifier: LGPL-3.0-or-later
use thiserror::Error;

/// The Error that can occur when using types
#[derive(Debug, Error, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// An invalid build date (in seconds since the epoch)
    #[error("Invalid build date: {0}")]
    InvalidBuildDate(String),
    /// An invalid compressed file size (in bytes)
    #[error("Invalid compressed size: {0}")]
    InvalidCompressedSize(String),
    /// An invalid installed package size (in bytes)
    #[error("Invalid installed size: {0}")]
    InvalidInstalledSize(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_format_string() {
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
