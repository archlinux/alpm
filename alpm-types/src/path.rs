// SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
// SPDX-License-Identifier: LGPL-3.0-or-later
use std::fmt::Display;
use std::fmt::Formatter;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use crate::Error;

/// A representation of an absolute path
///
/// AbsolutePath wraps a `PathBuf`, that is guaranteed to be absolute.
///
/// ## Examples
/// ```
/// use alpm_types::{AbsolutePath, Error};
/// use std::str::FromStr;
///
/// // create BuildDir from &str
/// assert_eq!(
///     AbsolutePath::from_str("/"),
///     Ok(AbsolutePath::new("/").unwrap())
/// );
/// assert_eq!(
///     AbsolutePath::from_str("./"),
///     Err(Error::InvalidAbsolutePath(String::from("./")))
/// );
///
/// // format as String
/// assert_eq!("/", format!("{}", AbsolutePath::new("/").unwrap()));
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbsolutePath(PathBuf);

impl AbsolutePath {
    pub fn new(input: &str) -> Result<AbsolutePath, Error> {
        match Path::new(input).is_absolute() {
            true => Ok(AbsolutePath(PathBuf::from(input))),
            false => Err(Error::InvalidAbsolutePath(input.to_string())),
        }
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &Path {
        &self.0
    }
}

impl FromStr for AbsolutePath {
    type Err = Error;
    fn from_str(input: &str) -> Result<AbsolutePath, Self::Err> {
        AbsolutePath::new(input)
    }
}

impl Display for AbsolutePath {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner().display())
    }
}

/// An absolute path used as build directory
///
/// BuildDir wraps an `AbsolutePath`
///
/// ## Examples
/// ```
/// use alpm_types::{BuildDir, Error};
/// use std::str::FromStr;
///
/// // create BuildDir from &str
/// assert_eq!(
///     BuildDir::from_str("/"),
///     Ok(BuildDir::new("/").unwrap())
/// );
/// assert_eq!(
///     BuildDir::from_str("/foo.txt"),
///     Ok(BuildDir::new("/foo.txt").unwrap())
/// );
///
/// // format as String
/// assert_eq!("/", format!("{}", BuildDir::new("/").unwrap()));
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildDir(AbsolutePath);

impl BuildDir {
    /// Create a new BuildDir
    pub fn new(absolute_path: &str) -> Result<BuildDir, Error> {
        match AbsolutePath::new(absolute_path) {
            Ok(abs_path) => Ok(BuildDir(abs_path)),
            _ => Err(Error::InvalidBuildDir(absolute_path.to_string())),
        }
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &AbsolutePath {
        &self.0
    }
}

impl FromStr for BuildDir {
    type Err = Error;
    fn from_str(absolute_path: &str) -> Result<BuildDir, Self::Err> {
        BuildDir::new(absolute_path)
    }
}

impl Display for BuildDir {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("/home", BuildDir::new("/home"))]
    #[case("./", Err(Error::InvalidBuildDir(String::from("./"))))]
    #[case("~/", Err(Error::InvalidBuildDir(String::from("~/"))))]
    #[case("foo.txt", Err(Error::InvalidBuildDir(String::from("foo.txt"))))]
    fn build_dir_from_string(#[case] from_str: &str, #[case] result: Result<BuildDir, Error>) {
        assert_eq!(BuildDir::from_str(from_str), result);
    }
}