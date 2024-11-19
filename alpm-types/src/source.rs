use std::{
    fmt::{Display, Formatter},
    path::{Path, PathBuf},
    str::FromStr,
};

use url::Url;

use crate::Error;

/// Represents a single (relative) filename, without any directories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filename {
    inner: PathBuf,
}

impl Filename {
    /// Creates a new `Filename`
    ///
    /// ## Errors
    ///
    /// Returns an error if the string contains directories, is absolute, or is otherwise an
    /// invalid path.
    pub fn new(path: PathBuf) -> Result<Self, Error> {
        if path.components().count() > 1 {
            Err(Error::FileNameContainsInvalidChars(
                path,
                std::path::MAIN_SEPARATOR,
            ))
        } else if path.components().any(|v| {
            v.as_os_str()
                .to_str()
                .map(|v| v.contains('\0'))
                .unwrap_or(false)
        }) {
            Err(Error::FileNameContainsInvalidChars(path, '\0'))
        } else {
            Ok(Self { inner: path })
        }
    }

    /// Returns a reference to the filename as a `&str`.
    pub fn as_str(&self) -> &str {
        // Can only be constructed from valid strings
        self.inner.as_os_str().to_str().unwrap()
    }

    /// Consumes the `Filename` and returns the filename as a `String`.
    pub fn into_string(self) -> String {
        // Can only be constructed from valid strings
        self.inner.into_os_string().into_string().unwrap()
    }

    /// Returns a reference to the filename as a `&Path`.
    pub fn inner(&self) -> &Path {
        &self.inner
    }

    /// Consumes the `Filename` and returns the filename as a `PathBuf`.
    pub fn into_inner(self) -> PathBuf {
        self.inner
    }
}

impl FromStr for Filename {
    type Err = Error;
    /// Creates a new `Filename` from a string.
    ///
    /// ## Errors
    ///
    /// Returns an error if the string contains directories, is absolute, or is otherwise an
    /// invalid path.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err(Error::FileNameIsEmpty)
        } else if s.contains(std::path::MAIN_SEPARATOR) {
            Err(Error::FileNameContainsInvalidChars(
                PathBuf::from(s),
                std::path::MAIN_SEPARATOR,
            ))
        } else if s.contains('\0') {
            Err(Error::FileNameContainsInvalidChars(PathBuf::from(s), '\0'))
        } else {
            Ok(Self { inner: s.into() })
        }
    }
}

/// Represents the location that a source file should be retrieved from - either a local file (next
/// to the PKGBUILD) or a URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceLocation {
    File(Filename),
    Url(Url),
}

impl SourceLocation {
    /// Creates a new `SourceLocation` from a local file.
    pub fn from_file(file: Filename) -> Self {
        Self::File(file)
    }

    /// Creates a new `SourceLocation` from a URL.
    pub fn from_url(url: Url) -> Self {
        Self::Url(url)
    }
}

impl FromStr for SourceLocation {
    type Err = Error;

    /// Parses a source location from a string. It must be either a valid URL or a plain
    /// filename.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse() {
            Ok(url) => Ok(Self::Url(url)),
            Err(url::ParseError::RelativeUrlWithoutBase) => {
                Filename::from_str(s).map(Self::File).map_err(Into::into)
            }
            Err(e) => Err(e.into()),
        }
    }
}

/// Represents a source directive.
///
/// Consists of an optional local filename and a [`SourceLocation`]
/// to get the file from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub filename: Option<Filename>,
    pub location: SourceLocation,
}

impl Source {
    /// Creates a new `Source` directive
    pub fn new(filename: Option<Filename>, location: SourceLocation) -> Self {
        Self { filename, location }
    }
}

impl FromStr for Source {
    type Err = Error;

    /// Parses a source directive. It is either a filename (in the same directory as the PKGBUILD)
    /// or a url, optionally prefixed by a destination file name (separated by `::`).
    ///
    /// ## Errors
    ///
    /// This function returns an error if the destination file name or url/source file name are
    /// malformed.
    ///
    /// ## Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::{Source, SourceLocation};
    /// use url::Url;
    ///
    /// let source = Source::from_str("foopkg-1.2.3.tar.gz::https://example.com/download").unwrap();
    /// assert_eq!(source.filename.unwrap().as_str(), "foopkg-1.2.3.tar.gz");
    /// let SourceLocation::Url(url) = source.location else {
    ///     panic!()
    /// };
    /// assert_eq!(url.host_str(), Some("example.com"));
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((filename, loc)) = s.split_once("::") {
            Ok(Self {
                filename: Some(filename.parse()?),
                location: loc.parse()?,
            })
        } else {
            Ok(Self {
                filename: None,
                location: s.parse()?,
            })
        }
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(filename) = &self.filename {
            write!(f, "{}::", filename.as_str())?;
        }
        match &self.location {
            SourceLocation::File(file) => write!(f, "{}", file.as_str()),
            SourceLocation::Url(u) => write!(f, "{u}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("bikeshed_colour.patch", Ok(Filename {
        inner: PathBuf::from("bikeshed_colour.patch"),
    }))]
    #[case("c:foo", Ok(Filename {
        inner: PathBuf::from("c:foo"),
    }))]
    #[case(
        "./bikeshed_colour.patch",
        Err(Error::FileNameContainsInvalidChars(PathBuf::from("./bikeshed_colour.patch"), '/'))
    )]
    #[case("", Err(Error::FileNameIsEmpty))]
    #[case(
        "with\0null",
        Err(Error::FileNameContainsInvalidChars(PathBuf::from("with\0null"), '\0'))
    )]
    fn parse_filename(#[case] input: &str, #[case] expected: Result<Filename, Error>) {
        let filename = input.parse();
        assert_eq!(filename, expected);

        if let Ok(filename) = filename {
            assert_eq!(filename.as_str(), input);
        }
    }

    #[rstest]
    #[case("bikeshed_colour.patch", Ok(Source {
        filename: None,
        location: SourceLocation::File("bikeshed_colour.patch".parse().unwrap()),
    }))]
    #[case("renamed::local", Ok(Source {
        filename: Some("renamed".parse().unwrap()),
        location: SourceLocation::File("local".parse().unwrap()),
    }))]
    #[case("foo-1.2.3.tar.gz::https://example.com/download", Ok(Source {
        filename: Some("foo-1.2.3.tar.gz".parse().unwrap()),
        location: SourceLocation::Url(Url::parse("https://example.com/download").unwrap()),
    }))]
    #[case("my-git-repo::git+https://example.com/project/repo.git#commit=deadbeef?signed", Ok(Source {
        filename: Some("my-git-repo".parse().unwrap()),
        location: SourceLocation::Url(Url::parse("git+https://example.com/project/repo.git#commit=deadbeef?signed").unwrap()),
    }))]
    #[case("file:///somewhere/else", Ok(Source {
        filename: None,
        location: SourceLocation::Url(Url::parse("file:///somewhere/else").unwrap()),
    }))]
    #[case(
        "/absolute/path",
        Err(Error::FileNameContainsInvalidChars(PathBuf::from("/absolute/path"), '/'))
    )]
    #[case(
        "foo:::/absolute/path",
        Err(Error::FileNameContainsInvalidChars(PathBuf::from(":/absolute/path"), '/'))
    )]
    fn parse_source(#[case] input: &str, #[case] expected: Result<Source, Error>) {
        let source = input.parse();
        assert_eq!(source, expected);

        if let Ok(source) = source {
            assert_eq!(source.to_string(), input);
        }
    }
}
