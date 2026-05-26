use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
    str::FromStr,
};

use alpm_parsers::traits::ParserUntil;
use serde::{Deserialize, Serialize};
use winnow::{
    ModalResult,
    Parser,
    combinator::{alt, peek, repeat_till},
    error::{ContextError, ErrMode, StrContext, StrContextValue},
    stream::Stream,
    token::any,
};

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

/// For Source, we only define a [`ParserUntil`] trait and not the `AlpmParser` trait, as we
/// don't provide the [`Url`](url::Url) type parser ourselves. Hence, the indicator for its supposed
/// "end" must be provided by the caller of the parser.
impl ParserUntil for Source {
    fn parser_until<'a, P>(delimiter: P) -> impl Parser<&'a str, Self, ErrMode<ContextError>>
    where
        P: Parser<&'a str, &'a str, ErrMode<ContextError>>,
    {
        // Define the actual parser closure.
        // The delimiter is moved into the closure and borrowed via `by_ref()` on each call.
        let mut delimiter_parser = delimiter;
        move |input: &mut &'a str| -> ModalResult<Self> {
            // We have to work with checkpoints here, as we cannot `peek` with a `repeat_till` +
            // `delimiter_parser`, as that would require the `delimiter_parser` to be borrowed
            // twice.
            let checkpoint = input.checkpoint();

            // First up, we handle the case that there's a filename prefix e.g. `filename::...`.
            // As such, we parse everything until either the expected end or the `::` delimiter.
            let path: &str = repeat_till::<_, _, (), _, _, _, _>(
                1..,
                any,
                peek(alt(("::", delimiter_parser.by_ref()))),
            )
            .context(StrContext::Label("source url"))
            .context(StrContext::Expected(StrContextValue::Description(
                "a filename followed by `::` or a path/url with valid end of input.",
            )))
            .take()
            .parse_next(input)?;

            let mut filename = None;
            // We now check, if we hit the `::` delimiter, in which case, the input until here is
            // treated as a filename.
            // Otherwise, we reset to the start of the string and will handle the whole expected
            // input as a URL.
            let delimiter = alt(("::", peek(delimiter_parser.by_ref()))).parse_next(input)?;
            if delimiter == "::" {
                filename = Some(path.into())
            } else {
                input.reset(&checkpoint);
            }

            // Now, take the rest until we hit the delimiter.
            let source_url =
                repeat_till::<_, _, (), _, _, _, _>(0.., any, peek(delimiter_parser.by_ref()))
                    .take()
                    .try_map(move |location: &str| {
                        // The following logic is a bit convoluted:
                        //
                        // - Check if we have a valid URL
                        // - If we don't have an URL, check if we have a valid relative filename.
                        // - If it is a valid URL go ahead and do the next parsing sequence into a
                        //   SourceUrl.
                        match location.parse::<url::Url>() {
                            Ok(_) => {
                                // Parse potential extra syntax from the URL.
                                let source_url = SourceUrl::from_str(location)?;

                                Ok(Self::SourceUrl {
                                    filename: filename.clone(),
                                    source_url,
                                })
                            }
                            Err(url::ParseError::RelativeUrlWithoutBase) => {
                                if location.is_empty() {
                                    Err(Error::FileNameIsEmpty)
                                } else if location.contains(std::path::MAIN_SEPARATOR) {
                                    Err(Error::FileNameContainsInvalidChars(
                                        PathBuf::from(location),
                                        std::path::MAIN_SEPARATOR,
                                    ))
                                } else if location.contains('\0') {
                                    Err(Error::FileNameContainsInvalidChars(
                                        PathBuf::from(location),
                                        '\0',
                                    ))
                                } else {
                                    // We have a valid relative file. Return early
                                    Ok(Self::File {
                                        filename: filename.clone(),
                                        location: location.into(),
                                    })
                                }
                            }
                            Err(e) => Err(e.into()),
                        }
                    })
                    .parse_next(input)?;

            // Now make sure we actually hit the expected delimiter.
            peek(delimiter_parser.by_ref())
                .context(StrContext::Label("source url"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "end of input.",
                )))
                .parse_next(input)?;

            Ok(source_url)
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
        Ok(Self::parser_until_eof.parse(s)?)
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
    use insta::assert_snapshot;
    use rstest::rstest;

    use super::*;
    use crate::configure_insta;

    #[rstest]
    #[case("bikeshed_colour.patch::test", Source::File {
        filename: Some(PathBuf::from("bikeshed_colour.patch")),
        location: PathBuf::from("test"),
    })]
    #[case("c:foo::test", Source::File {
        filename: Some(PathBuf::from("c:foo")),
        location: PathBuf::from("test"),
    })]
    #[case("renamed::local", Source::File {
        filename: Some(PathBuf::from("renamed")),
        location: PathBuf::from("local"),
    })]
    #[case("bikeshed_colour.patch",Source::File {
        filename: None,
        location: PathBuf::from("bikeshed_colour.patch"),
    })]
    #[case(
        "foo-1.2.3.tar.gz::https://example.com/download",
        Source::SourceUrl {
            filename: Some(PathBuf::from("foo-1.2.3.tar.gz")),
            source_url: SourceUrl::from_str("https://example.com/download").unwrap(),
        }
    )]
    #[case(
        "my-git-repo::git+https://example.com/project/repo.git?signed#commit=deadbeef",
        Source::SourceUrl {
            filename: Some(PathBuf::from("my-git-repo")),
            source_url: SourceUrl::from_str("git+https://example.com/project/repo.git?signed#commit=deadbeef").unwrap(),
        }
    )]
    #[case(
        "file:///somewhere/else",
        Source::SourceUrl {
            filename: None,
            source_url: SourceUrl::from_str("file:///somewhere/else").unwrap(),
        }
    )]
    fn valid_source(#[case] input: &str, #[case] expected: Source) {
        assert_eq!(
            Source::from_str(input),
            Ok(expected),
            "Expected valid parsing for Source: {input}"
        );
    }

    #[rstest]
    #[case("./bikeshed_colour.patch")]
    #[case("")]
    #[case("with\0null")]
    #[case("/absolute/path")]
    #[case("foo:::/absolute/path")]
    fn invalid_filename(#[case] input: &str) {
        let Err(Error::ParseError(err_msg)) = Source::from_str(input) else {
            panic!("'{input}' erroneously parsed as a Source")
        };

        let (test_name, _guard) = configure_insta();
        assert_snapshot!(test_name, err_msg.to_string());
    }
}
