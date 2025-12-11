use std::{
    fmt::{Display, Formatter},
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use winnow::{
    ModalResult,
    Parser,
    combinator::{alt, cut_err, eof, peek, repeat_till},
    error::{StrContext, StrContextValue},
    token::{any, rest},
};

use crate::{Error, SharedLibraryPrefix};

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
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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

/// A representation of a relative path
///
/// [`RelativePath`] wraps a [`PathBuf`] that is guaranteed to represent a relative path, regardless
/// of whether it points to a file or a directory.
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
///     RelativePath::from_str("etc/"),
///     RelativePath::new(PathBuf::from("etc/"))
/// );
/// assert_eq!(
///     RelativePath::from_str("/etc/test.conf"),
///     Err(Error::PathNotRelative(PathBuf::from("/etc/test.conf")))
/// );
///
/// // Format as String
/// assert_eq!("test/", RelativePath::from_str("test/")?.to_string());
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct RelativePath(PathBuf);

impl RelativePath {
    /// Create a new [`RelativePath`]
    pub fn new(path: PathBuf) -> Result<RelativePath, Error> {
        if !path.is_relative() {
            return Err(Error::PathNotRelative(path));
        }
        Ok(RelativePath(path))
    }

    /// Consume `self` and return the inner [`PathBuf`]
    pub fn into_inner(self) -> PathBuf {
        self.0
    }
}

impl AsRef<Path> for RelativePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl FromStr for RelativePath {
    type Err = Error;

    /// Parses a relative path from a string
    ///
    /// # Errors
    ///
    /// Returns an error if the path is not relative.
    fn from_str(s: &str) -> Result<RelativePath, Self::Err> {
        Self::new(PathBuf::from(s))
    }
}

impl Display for RelativePath {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.as_ref().display())
    }
}

/// A representation of a relative file path
///
/// `RelativeFilePath` wraps a `PathBuf` that is guaranteed to represent a
/// relative file path (i.e. it does not end with a `/`).
///
/// ## Examples
///
/// ```
/// use std::{path::PathBuf, str::FromStr};
///
/// use alpm_types::{Error, RelativeFilePath};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // Create RelativeFilePath from &str
/// assert_eq!(
///     RelativeFilePath::from_str("etc/test.conf"),
///     RelativeFilePath::new(PathBuf::from("etc/test.conf"))
/// );
/// assert_eq!(
///     RelativeFilePath::from_str("/etc/test.conf"),
///     Err(Error::PathNotRelative(PathBuf::from("/etc/test.conf")))
/// );
///
/// // Format as String
/// assert_eq!(
///     "test/test.txt",
///     RelativeFilePath::from_str("test/test.txt")?.to_string()
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct RelativeFilePath(PathBuf);

impl RelativeFilePath {
    /// Create a new `RelativeFilePath`
    pub fn new(path: PathBuf) -> Result<RelativeFilePath, Error> {
        if path
            .to_string_lossy()
            .ends_with(std::path::MAIN_SEPARATOR_STR)
        {
            return Err(Error::PathIsNotAFile(path));
        }
        if !path.is_relative() {
            return Err(Error::PathNotRelative(path));
        }
        Ok(RelativeFilePath(path))
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &Path {
        &self.0
    }
}

impl FromStr for RelativeFilePath {
    type Err = Error;

    /// Parses a relative path from a string
    ///
    /// # Errors
    ///
    /// Returns an error if the path is not relative
    fn from_str(s: &str) -> Result<RelativeFilePath, Self::Err> {
        Self::new(PathBuf::from(s))
    }
}

impl Display for RelativeFilePath {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner().display())
    }
}

/// The path of a packaged file that should be preserved during package operations
///
/// This is a type alias for [`RelativeFilePath`]
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
pub type Backup = RelativeFilePath;

/// A special install script that is to be included in the package
///
/// This is a type alias for [RelativeFilePath`]
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
pub type Install = RelativeFilePath;

/// The relative path to a changelog file that may be included in a package
///
/// This is a type alias for [`RelativeFilePath`]
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
pub type Changelog = RelativeFilePath;

/// A lookup directory for shared object files.
///
/// Follows the [alpm-sonamev2] format, which encodes a `prefix` and a `directory`.
/// The same `prefix` is later used to identify the location of a **soname**, see
/// [`SonameV2`][crate::SonameV2].
///
/// [alpm-sonamev2]: https://alpm.archlinux.page/specifications/alpm-sonamev2.7.html
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SonameLookupDirectory {
    /// The lookup prefix for shared objects.
    pub prefix: SharedLibraryPrefix,
    /// The directory to look for shared objects in.
    pub directory: AbsolutePath,
}

impl SonameLookupDirectory {
    /// Creates a new lookup directory with a prefix and a directory.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_types::SonameLookupDirectory;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// SonameLookupDirectory::new("lib".parse()?, "/usr/lib".parse()?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(prefix: SharedLibraryPrefix, directory: AbsolutePath) -> Self {
        Self { prefix, directory }
    }

    /// Parses a [`SonameLookupDirectory`] from a string slice.
    ///
    /// Consumes all of its input.
    ///
    /// See [`SonameLookupDirectory::from_str`] for more details.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        // Parse until the first `:`, which separates the prefix from the directory.
        let prefix = cut_err(
            repeat_till(1.., any, peek(alt((":", eof))))
                .try_map(|(name, _): (String, &str)| SharedLibraryPrefix::from_str(&name)),
        )
        .context(StrContext::Label("prefix for a shared object lookup path"))
        .parse_next(input)?;

        // Take the delimiter.
        cut_err(":")
            .context(StrContext::Label("shared library prefix delimiter"))
            .context(StrContext::Expected(StrContextValue::Description(
                "shared library prefix `:`",
            )))
            .parse_next(input)?;

        // Parse the rest as a directory.
        let directory = rest
            .verify(|s: &str| !s.is_empty())
            .try_map(AbsolutePath::from_str)
            .context(StrContext::Label("directory"))
            .context(StrContext::Expected(StrContextValue::Description(
                "directory for a shared object lookup path",
            )))
            .parse_next(input)?;

        Ok(Self { prefix, directory })
    }
}

impl Display for SonameLookupDirectory {
    /// Converts the [`SonameLookupDirectory`] to a string.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.prefix, self.directory)
    }
}

impl FromStr for SonameLookupDirectory {
    type Err = Error;

    /// Creates a [`SonameLookupDirectory`] from a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` can not be converted into a [`SonameLookupDirectory`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::SonameLookupDirectory;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let dir = SonameLookupDirectory::from_str("lib:/usr/lib")?;
    /// assert_eq!(dir.to_string(), "lib:/usr/lib");
    /// assert!(SonameLookupDirectory::from_str(":/usr/lib").is_err());
    /// assert!(SonameLookupDirectory::from_str(":/usr/lib").is_err());
    /// assert!(SonameLookupDirectory::from_str("lib:").is_err());
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use testresult::TestResult;

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
    #[case("etc/", RelativePath::new(PathBuf::from("etc/")))]
    #[case(
        "/etc/test.conf",
        Err(Error::PathNotRelative(PathBuf::from("/etc/test.conf")))
    )]
    #[case(
        "../etc/test.conf",
        RelativePath::new(PathBuf::from("../etc/test.conf"))
    )]
    fn relative_path_from_str(#[case] s: &str, #[case] result: Result<RelativePath, Error>) {
        assert_eq!(RelativePath::from_str(s), result);
    }

    #[rstest]
    #[case("etc/test.conf", RelativeFilePath::new(PathBuf::from("etc/test.conf")))]
    #[case(
        "/etc/test.conf",
        Err(Error::PathNotRelative(PathBuf::from("/etc/test.conf")))
    )]
    #[case("etc/", Err(Error::PathIsNotAFile(PathBuf::from("etc/"))))]
    #[case("etc", RelativeFilePath::new(PathBuf::from("etc")))]
    #[case(
        "../etc/test.conf",
        RelativeFilePath::new(PathBuf::from("../etc/test.conf"))
    )]
    fn relative_file_path_from_str(
        #[case] s: &str,
        #[case] result: Result<RelativeFilePath, Error>,
    ) {
        assert_eq!(RelativeFilePath::from_str(s), result);
    }

    #[rstest]
    #[case("lib:/usr/lib", SonameLookupDirectory {
        prefix: "lib".parse()?,
        directory: AbsolutePath::from_str("/usr/lib")?,
    })]
    #[case("lib32:/usr/lib32", SonameLookupDirectory {
        prefix: "lib32".parse()?,
        directory: AbsolutePath::from_str("/usr/lib32")?,
    })]
    fn soname_lookup_directory_from_string(
        #[case] input: &str,
        #[case] expected_result: SonameLookupDirectory,
    ) -> TestResult {
        let lookup_directory = SonameLookupDirectory::from_str(input)?;
        assert_eq!(expected_result, lookup_directory);
        assert_eq!(input, lookup_directory.to_string());
        Ok(())
    }

    #[rstest]
    #[case("lib", "invalid shared library prefix delimiter")]
    #[case("lib:", "invalid directory")]
    #[case(":/usr/lib", "invalid first character of package name")]
    fn invalid_soname_lookup_directory_parser(#[case] input: &str, #[case] error_snippet: &str) {
        let result = SonameLookupDirectory::from_str(input);
        assert!(result.is_err(), "Expected LookupDirectory parsing to fail");
        let err = result.unwrap_err();
        let pretty_error = err.to_string();
        assert!(
            pretty_error.contains(error_snippet),
            "Error:\n=====\n{pretty_error}\n=====\nshould contain snippet:\n\n{error_snippet}"
        );
    }
}
