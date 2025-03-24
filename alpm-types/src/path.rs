use std::{
    fmt::{Display, Formatter},
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::Error;

/// A representation of an absolute path
///
/// AbsolutePath wraps a `PathBuf`, that is guaranteed to be absolute.
///
/// ## Examples
/// ```
/// use std::{path::PathBuf, str::FromStr};
///
/// use alpm_types::{AbsolutePath, Error};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // Create AbsolutePath from &str
/// assert_eq!(
///     AbsolutePath::from_str("/"),
///     AbsolutePath::new(PathBuf::from("/"))
/// );
/// assert_eq!(
///     AbsolutePath::from_str("./"),
///     Err(Error::PathNotAbsolute(PathBuf::from("./")))
/// );
///
/// // Format as String
/// assert_eq!("/", format!("{}", AbsolutePath::from_str("/")?));
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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
    /// # Errors
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
/// use std::str::FromStr;
///
/// use alpm_types::{Error, BuildDirectory};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // Create BuildDirectory from &str and format it
/// assert_eq!(
///     "/etc",
///     BuildDirectory::from_str("/etc")?.to_string()
/// );
/// # Ok(())
/// # }
pub type BuildDirectory = AbsolutePath;

/// An absolute path used as start directory in a package build environment
///
/// This is a type alias for [`AbsolutePath`]
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Error, StartDirectory};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // Create StartDirectory from &str and format it
/// assert_eq!(
///     "/etc",
///     StartDirectory::from_str("/etc")?.to_string()
/// );
/// # Ok(())
/// # }
pub type StartDirectory = AbsolutePath;

/// A representation of a relative file path
///
/// `RelativePath` wraps a `PathBuf` that is guaranteed to represent a
/// relative file path (i.e. it does not end with a `/`).
///
/// ## Examples
///
/// ```
/// use std::{path::PathBuf, str::FromStr};
///
/// use alpm_types::{Error, RelativePath};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // Create RelativePath from &str
/// assert_eq!(
///     RelativePath::from_str("etc/test.conf"),
///     RelativePath::new(PathBuf::from("etc/test.conf"))
/// );
/// assert_eq!(
///     RelativePath::from_str("/etc/test.conf"),
///     Err(Error::PathNotRelative(PathBuf::from("/etc/test.conf")))
/// );
///
/// // Format as String
/// assert_eq!(
///     "test/test.txt",
///     RelativePath::from_str("test/test.txt")?.to_string()
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RelativePath(PathBuf);

impl RelativePath {
    /// Create a new `RelativePath`
    pub fn new(path: PathBuf) -> Result<RelativePath, Error> {
        match path.is_relative()
            && !path
                .to_string_lossy()
                .ends_with(std::path::MAIN_SEPARATOR_STR)
        {
            true => Ok(RelativePath(path)),
            false => Err(Error::PathNotRelative(path)),
        }
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &Path {
        &self.0
    }
}

impl FromStr for RelativePath {
    type Err = Error;

    /// Parses a relative path from a string
    ///
    /// # Errors
    ///
    /// Returns an error if the path is not relative
    fn from_str(s: &str) -> Result<RelativePath, Self::Err> {
        Self::new(PathBuf::from(s))
    }
}

impl Display for RelativePath {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner().display())
    }
}

/// The path of a packaged file that should be preserved during package operations
///
/// This is a type alias for [`RelativePath`]
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::Backup;
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // Create Backup from &str and format it
/// assert_eq!(
///     "etc/test.conf",
///     Backup::from_str("etc/test.conf")?.to_string()
/// );
/// # Ok(())
/// # }
pub type Backup = RelativePath;

/// A special install script that is to be included in the package
///
/// This is a type alias for [RelativePath`]
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Error, Install};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // Create Install from &str and format it
/// assert_eq!(
///     "scripts/setup.install",
///     Install::from_str("scripts/setup.install")?.to_string()
/// );
/// # Ok(())
/// # }
pub type Install = RelativePath;

/// The relative path to a changelog file that may be included in a package
///
/// This is a type alias for [`RelativePath`]
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Error, Changelog};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // Create Changelog from &str and format it
/// assert_eq!(
///     "changelog.md",
///     Changelog::from_str("changelog.md")?.to_string()
/// );
/// # Ok(())
/// # }
pub type Changelog = RelativePath;

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("/home", BuildDirectory::new(PathBuf::from("/home")))]
    #[case("./", Err(Error::PathNotAbsolute(PathBuf::from("./"))))]
    #[case("~/", Err(Error::PathNotAbsolute(PathBuf::from("~/"))))]
    #[case("foo.txt", Err(Error::PathNotAbsolute(PathBuf::from("foo.txt"))))]
    fn build_dir_from_string(#[case] s: &str, #[case] result: Result<BuildDirectory, Error>) {
        assert_eq!(BuildDirectory::from_str(s), result);
    }

    #[rstest]
    #[case("/start", StartDirectory::new(PathBuf::from("/start")))]
    #[case("./", Err(Error::PathNotAbsolute(PathBuf::from("./"))))]
    #[case("~/", Err(Error::PathNotAbsolute(PathBuf::from("~/"))))]
    #[case("foo.txt", Err(Error::PathNotAbsolute(PathBuf::from("foo.txt"))))]
    fn startdir_from_str(#[case] s: &str, #[case] result: Result<StartDirectory, Error>) {
        assert_eq!(StartDirectory::from_str(s), result);
    }

    #[rstest]
    #[case("etc/test.conf", RelativePath::new(PathBuf::from("etc/test.conf")))]
    #[case(
        "/etc/test.conf",
        Err(Error::PathNotRelative(PathBuf::from("/etc/test.conf")))
    )]
    #[case("etc/", Err(Error::PathNotRelative(PathBuf::from("etc/"))))]
    #[case("etc", RelativePath::new(PathBuf::from("etc")))]
    #[case(
        "../etc/test.conf",
        RelativePath::new(PathBuf::from("../etc/test.conf"))
    )]
    fn relative_path_from_str(#[case] s: &str, #[case] result: Result<RelativePath, Error>) {
        assert_eq!(RelativePath::from_str(s), result);
    }
}
