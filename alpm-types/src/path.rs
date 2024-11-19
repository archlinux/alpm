use std::{
    fmt::{Display, Formatter},
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::Error;

/// A representation of an absolute path
///
/// AbsolutePath wraps a `PathBuf`, that is guaranteed to be absolute.
///
/// ## Examples
/// ```
/// use std::path::PathBuf;
/// use std::str::FromStr;
///
/// use alpm_types::{AbsolutePath, Error};
///
/// // create BuildDir from &str
/// assert_eq!(
///     AbsolutePath::from_str("/"),
///     AbsolutePath::new(PathBuf::from("/"))
/// );
/// assert_eq!(
///     AbsolutePath::from_str("./"),
///     Err(Error::PathNotAbsolute(PathBuf::from("./")))
/// );
///
/// // format as String
/// assert_eq!("/", format!("{}", AbsolutePath::from_str("/").unwrap()));
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbsolutePath(PathBuf);

impl AbsolutePath {
    /// Create a new `AbsolutePath`
    pub fn new(path: PathBuf) -> Result<AbsolutePath, Error> {
        match path.is_absolute() {
            true => Ok(AbsolutePath(path)),
            false => Err(Error::PathNotAbsolute(path)),
        }
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &Path {
        &self.0
    }
}

impl FromStr for AbsolutePath {
    type Err = Error;

    /// Parses an absolute path from a string
    ///
    /// ## Errors
    ///
    /// Returns an error if the path is not absolute
    fn from_str(s: &str) -> Result<AbsolutePath, Self::Err> {
        match Path::new(s).is_absolute() {
            true => Ok(AbsolutePath(PathBuf::from(s))),
            false => Err(Error::PathNotAbsolute(PathBuf::from(s))),
        }
    }
}

impl Display for AbsolutePath {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner().display())
    }
}

/// An absolute path used as build directory
///
/// This is a type alias for [`AbsolutePath`]
///
/// ## Examples
/// ```
/// use std::path::PathBuf;
/// use std::str::FromStr;
///
/// use alpm_types::{BuildDir, Error};
///
/// // create BuildDir from &str
/// assert_eq!(BuildDir::from_str("/"), BuildDir::new(PathBuf::from("/")));
/// assert_eq!(
///     BuildDir::from_str("/foo.txt"),
///     BuildDir::new(PathBuf::from("/foo.txt"))
/// );
///
/// // format as String
/// assert_eq!("/", format!("{}", BuildDir::from_str("/").unwrap()));
/// ```
pub type BuildDir = AbsolutePath;

/// An absolute path used as start directory in a package build environment
///
/// This is a type alias for [`AbsolutePath`]
///
/// ## Examples
/// ```
/// use std::path::PathBuf;
/// use std::str::FromStr;
///
/// use alpm_types::{Error, StartDir};
///
/// // create StartDir from &str
/// assert_eq!(
///     StartDir::from_str("/").unwrap(),
///     StartDir::new(PathBuf::from("/")).unwrap()
/// );
/// assert_eq!(
///     StartDir::from_str("/foo.txt").unwrap(),
///     StartDir::new(PathBuf::from("/foo.txt")).unwrap()
/// );
///
/// // format as String
/// assert_eq!("/", format!("{}", StartDir::from_str("/").unwrap()));
/// ```
pub type StartDir = AbsolutePath;

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("/home", BuildDir::new(PathBuf::from("/home")))]
    #[case("./", Err(Error::PathNotAbsolute(PathBuf::from("./"))))]
    #[case("~/", Err(Error::PathNotAbsolute(PathBuf::from("~/"))))]
    #[case("foo.txt", Err(Error::PathNotAbsolute(PathBuf::from("foo.txt"))))]
    fn build_dir_from_string(#[case] s: &str, #[case] result: Result<BuildDir, Error>) {
        assert_eq!(BuildDir::from_str(s), result);
    }

    #[rstest]
    #[case("/start", StartDir::new(PathBuf::from("/start")))]
    #[case("./", Err(Error::PathNotAbsolute(PathBuf::from("./"))))]
    #[case("~/", Err(Error::PathNotAbsolute(PathBuf::from("~/"))))]
    #[case("foo.txt", Err(Error::PathNotAbsolute(PathBuf::from("foo.txt"))))]
    fn startdir_from_str(#[case] s: &str, #[case] result: Result<StartDir, Error>) {
        assert_eq!(StartDir::from_str(s), result);
    }
}
