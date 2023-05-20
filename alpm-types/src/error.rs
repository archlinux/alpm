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
    /// An invalid build environment
    #[error("Invalid build environment string: {0}")]
    InvalidBuildEnv(String),
    /// An invalid build option
    #[error("Invalid option string: {0}")]
    InvalidBuildOption(String),
    /// An invalid BuildTool
    #[error("Invalid buildtool: {0}")]
    InvalidBuildTool(String),
    /// An invalid compressed file size (in bytes)
    #[error("Invalid compressed size: {0}")]
    InvalidCompressedSize(String),
    /// An invalid installed package size (in bytes)
    #[error("Invalid installed size: {0}")]
    InvalidInstalledSize(String),
    /// An invalid package name
    #[error("Invalid package name: {0}")]
    InvalidName(String),
    #[error("Invalid md5sum: {0}")]
    InvalidMd5Sum(String),
    #[error("Invalid packager string: {0}")]
    InvalidPackager(String),
    #[error("Invalid packager e-mail: {0}")]
    InvalidPackagerEmail(String),
    /// An invalid package option
    #[error("Invalid package option: {0}")]
    InvalidPackageOption(String),
    #[error("Invalid version string: {0}")]
    InvalidVersion(String),
    /// An invalid absolute path
    #[error("Invalid absolute path: {0}")]
    InvalidAbsolutePath(String),
    /// An invalid build directory
    #[error("Invalid build directory: {0}")]
    InvalidBuildDir(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("Invalid build date: -1", Error::InvalidBuildDate(String::from("-1")))]
    #[case(
        "Invalid compressed size: -1",
        Error::InvalidCompressedSize(String::from("-1"))
    )]
    #[case(
        "Invalid installed size: -1",
        Error::InvalidInstalledSize(String::from("-1"))
    )]
    #[case("Invalid package name: -1", Error::InvalidName(String::from("-1")))]
    #[case("Invalid md5sum: -1", Error::InvalidMd5Sum(String::from("-1")))]
    #[case(
        "Invalid packager string: foo",
        Error::InvalidPackager(String::from("foo"))
    )]
    #[case(
        "Invalid packager e-mail: foo",
        Error::InvalidPackagerEmail(String::from("foo"))
    )]
    #[case(
        "Invalid version string: -1",
        Error::InvalidVersion(String::from("-1"))
    )]
    fn error_format_string(#[case] error_str: &str, #[case] error: Error) {
        assert_eq!(error_str, format!("{}", error));
    }
}
