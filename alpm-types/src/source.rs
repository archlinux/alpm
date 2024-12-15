use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
    str::FromStr,
};

use url::Url;

use crate::Error;

/// Represents the location that a source file should be retrieved from
///
/// It can be either a local file (next to the PKGBUILD) or a URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// A local file source.
    ///
    /// The location must be a pure file name, without any path components (`/`).
    /// Hence, the file must be located directly next to the PKGBUILD.
    File {
        /// The optional destination file name.
        filename: Option<PathBuf>,
        /// The source file name.
        location: PathBuf,
    },
    /// A URL source.
    Url {
        /// The optional destination file name.
        filename: Option<PathBuf>,
        /// The source URL.
        url: Url,
    },
}

impl Source {
    /// Returns the filename of the source if it is set.
    pub fn filename(&self) -> Option<&PathBuf> {
        match self {
            Self::File { filename, .. } | Self::Url { filename, .. } => filename.as_ref(),
        }
    }
}

impl FromStr for Source {
    type Err = Error;

    /// Parses a `Source` from string.
    ///
    /// It is either a filename (in the same directory as the PKGBUILD)
    /// or a url, optionally prefixed by a destination file name (separated by `::`).
    ///
    /// ## Errors
    ///
    /// This function returns an error in the following cases:
    ///
    /// - The destination file name or url/source file name are malformed.
    /// - The source file name is an absolute path.
    ///
    /// ## Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use std::str::FromStr;
    ///
    /// use alpm_types::Source;
    /// use url::Url;
    ///
    /// let source = Source::from_str("foopkg-1.2.3.tar.gz::https://example.com/download").unwrap();
    /// assert_eq!(source.filename().unwrap(), Path::new("foopkg-1.2.3.tar.gz"));
    ///
    /// let Source::Url { url, .. } = source else {
    ///     panic!()
    /// };
    /// assert_eq!(url.host_str(), Some("example.com"));
    ///
    /// let source = Source::from_str("renamed-source.tar.gz::test.tar.gz").unwrap();
    /// assert_eq!(
    ///     source.filename().unwrap(),
    ///     Path::new("renamed-source.tar.gz")
    /// );
    ///
    /// let Source::File { location, .. } = source else {
    ///     panic!()
    /// };
    /// assert_eq!(location, Path::new("test.tar.gz"));
    /// ```
    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let filename = if let Some((filename, location)) = s.split_once("::") {
            s = location;
            Some(filename.into())
        } else {
            None
        };

        match s.parse() {
            Ok(url) => Ok(Self::Url { filename, url }),
            Err(url::ParseError::RelativeUrlWithoutBase) => {
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
                    Ok(Self::File {
                        filename,
                        location: s.into(),
                    })
                }
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File { filename, location } => {
                if let Some(filename) = filename {
                    write!(f, "{}::{}", filename.display(), location.display())
                } else {
                    write!(f, "{}", location.display())
                }
            }
            Self::Url { filename, url } => {
                if let Some(filename) = filename {
                    write!(f, "{}::{}", filename.display(), url)
                } else {
                    write!(f, "{}", url)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("bikeshed_colour.patch::test", Ok(Source::File {
        filename: Some(PathBuf::from("bikeshed_colour.patch")),
        location: PathBuf::from("test"),
    }))]
    #[case("c:foo::test", Ok(Source::File {
        filename: Some(PathBuf::from("c:foo")),
        location: PathBuf::from("test"),
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
    fn parse_filename(#[case] input: &str, #[case] expected: Result<Source, Error>) {
        let source = input.parse();
        assert_eq!(source, expected);

        if let Ok(source) = source {
            assert_eq!(
                source.filename(),
                input.split("::").next().map(PathBuf::from).as_ref()
            );
        }
    }

    #[rstest]
    #[case("bikeshed_colour.patch", Ok(Source::File {
        filename: None,
        location: PathBuf::from("bikeshed_colour.patch"),
    }))]
    #[case("renamed::local", Ok(Source::File {
        filename: Some(PathBuf::from("renamed")),
        location: PathBuf::from("local"),
    }))]
    #[case("foo-1.2.3.tar.gz::https://example.com/download", Ok(Source::Url {
        filename: Some(PathBuf::from("foo-1.2.3.tar.gz")),
        url: Url::parse("https://example.com/download").unwrap(),
    }))]
    #[case("my-git-repo::git+https://example.com/project/repo.git#commit=deadbeef?signed", Ok(Source::Url {
        filename: Some(PathBuf::from("my-git-repo")),
        url: Url::parse("git+https://example.com/project/repo.git#commit=deadbeef?signed").unwrap(),
    }))]
    #[case("file:///somewhere/else", Ok(Source::Url {
        filename: None,
        url: Url::parse("file:///somewhere/else").unwrap(),
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
        let source: Result<Source, Error> = input.parse();
        assert_eq!(source, expected);

        if let Ok(source) = source {
            assert_eq!(source.to_string(), input);
        }
    }
}
