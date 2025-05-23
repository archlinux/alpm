use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::{Error, SourceUrl};

/// Represents the location that a source file should be retrieved from
///
/// It can be either a local file (next to the PKGBUILD) or a URL.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
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
    SourceUrl {
        /// The optional destination file name.
        filename: Option<PathBuf>,
        /// The source URL.
        source_url: SourceUrl,
    },
}

impl Source {
    /// Returns the filename of the source if it is set.
    pub fn filename(&self) -> Option<&PathBuf> {
        match self {
            Self::File { filename, .. } | Self::SourceUrl { filename, .. } => filename.as_ref(),
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
    /// # Errors
    ///
    /// This function returns an error in the following cases:
    ///
    /// - The destination file name or url/source file name are malformed.
    /// - The source file name is an absolute path.
    ///
    /// ## Examples
    ///
    /// ```
    /// use std::{path::Path, str::FromStr};
    ///
    /// use alpm_types::Source;
    /// use url::Url;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    ///
    /// // Parse from a string that represents a remote file link.
    /// let source = Source::from_str("foopkg-1.2.3.tar.gz::https://example.com/download")?;
    /// let Source::SourceUrl {
    ///     source_url,
    ///     filename,
    /// } = source
    /// else {
    ///     panic!()
    /// };
    ///
    /// assert_eq!(filename.unwrap(), Path::new("foopkg-1.2.3.tar.gz"));
    /// assert_eq!(source_url.url.inner().host_str(), Some("example.com"));
    /// assert_eq!(source_url.to_string(), "https://example.com/download");
    ///
    /// // Parse from a string that represents a local file.
    /// let source = Source::from_str("renamed-source.tar.gz::test.tar.gz")?;
    /// let Source::File { location, filename } = source else {
    ///     panic!()
    /// };
    /// assert_eq!(location, Path::new("test.tar.gz"));
    /// assert_eq!(filename.unwrap(), Path::new("renamed-source.tar.gz"));
    ///
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // First up, check if there's a filename prefix e.g. `filename::...`.
        let (filename, location) = if let Some((filename, location)) = s.split_once("::") {
            (Some(filename.into()), location)
        } else {
            (None, s)
        };

        // The following logic is a bit convoluted:
        //
        // - Check if we have a valid URL
        // - If we don't have an URL, check if we have a valid relative filename.
        // - If it is a valid URL go ahead and do the next parsing sequence into a SourceUrl.
        match location.parse::<url::Url>() {
            Ok(_) => {}
            Err(url::ParseError::RelativeUrlWithoutBase) => {
                if location.is_empty() {
                    return Err(Error::FileNameIsEmpty);
                } else if location.contains(std::path::MAIN_SEPARATOR) {
                    return Err(Error::FileNameContainsInvalidChars(
                        PathBuf::from(location),
                        std::path::MAIN_SEPARATOR,
                    ));
                } else if location.contains('\0') {
                    return Err(Error::FileNameContainsInvalidChars(
                        PathBuf::from(location),
                        '\0',
                    ));
                } else {
                    // We have a valid relative file. Return early
                    return Ok(Self::File {
                        filename,
                        location: location.into(),
                    });
                }
            }
            Err(e) => return Err(e.into()),
        }

        // Parse potential extra syntax from the URL.
        let source_url = SourceUrl::from_str(location)?;
        Ok(Self::SourceUrl {
            filename,
            source_url,
        })
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
            Self::SourceUrl {
                filename,
                source_url,
            } => {
                if let Some(filename) = filename {
                    write!(f, "{}::{}", filename.display(), source_url)
                } else {
                    write!(f, "{source_url}")
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
    #[case(
        "foo-1.2.3.tar.gz::https://example.com/download",
        Ok(Source::SourceUrl {
            filename: Some(PathBuf::from("foo-1.2.3.tar.gz")),
            source_url: SourceUrl::from_str("https://example.com/download").unwrap(),
        })
    )]
    #[case(
        "my-git-repo::git+https://example.com/project/repo.git?signed#commit=deadbeef",
        Ok(Source::SourceUrl {
            filename: Some(PathBuf::from("my-git-repo")),
            source_url: SourceUrl::from_str("git+https://example.com/project/repo.git?signed#commit=deadbeef").unwrap(),
        })
    )]
    #[case(
        "file:///somewhere/else",
        Ok(Source::SourceUrl {
            filename: None,
            source_url: SourceUrl::from_str("file:///somewhere/else").unwrap(),
        })
    )]
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
