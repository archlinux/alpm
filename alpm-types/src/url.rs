//! Types for handling URLs and VCS-related information in package sources.

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use alpm_parsers::{iter_str_context, traits::ParserUntil};
use serde::{Deserialize, Serialize};
use winnow::{
    ModalResult,
    Parser,
    ascii::alpha1,
    combinator::{alt, eof, not, opt, peek, repeat_till, terminated},
    error::{ContextError, ErrMode, StrContext, StrContextValue},
    token::{any, rest},
};

use crate::Error;

/// Represents a URL.
///
/// It is used to represent the upstream URL of a package.
/// This type does not yet enforce a secure connection (e.g. HTTPS).
///
/// The `Url` type wraps the [`url::Url`] type.
///
/// ## Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::Url;
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // Create Url from &str
/// let url = Url::from_str("https://example.com/download")?;
/// assert_eq!(url.as_str(), "https://example.com/download");
///
/// // Format as String
/// assert_eq!(format!("{url}"), "https://example.com/download");
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Url(url::Url);

impl Url {
    /// Creates a new `Url` instance.
    pub fn new(url: url::Url) -> Result<Self, Error> {
        Ok(Self(url))
    }

    /// Returns a reference to the inner `url::Url` as a `&str`.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Consumes the `Url` and returns the inner `url::Url`.
    pub fn into_inner(self) -> url::Url {
        self.0
    }

    /// Returns a reference to the inner `url::Url`.
    pub fn inner(&self) -> &url::Url {
        &self.0
    }
}

impl AsRef<str> for Url {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for Url {
    type Err = Error;

    /// Creates a new `Url` instance from a string slice.
    ///
    /// ## Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::Url;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let url = Url::from_str("https://archlinux.org/")?;
    /// assert_eq!(url.as_str(), "https://archlinux.org/");
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = url::Url::parse(s).map_err(Error::InvalidUrl)?;
        Self::new(url)
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A URL for package sources.
///
/// Wraps the [`Url`] type and provides optional information on [VCS] systems.
///
/// Can be created from custom URL strings, that in part resemble the default [URL syntax], e.g.:
///
/// ```txt
/// git+https://example.org/example-project.git#tag=v1.0.0?signed
/// ```
///
/// The above example provides an overview of the custom URL syntax:
///
/// - The optional [VCS] specifier `git` is prepended, directly followed by a "+" sign as delimiter,
/// - specific URL `fragment` types such as `tag` are used to encode information about the
///   particular VCS objects to address,
/// - the URL `query` component `signed` is used to indicate that OpenPGP signature verification is
///   required for a VCS type.
///
/// ## Note
///
/// The URL format used by [`SourceUrl`] deviates from the default [URL syntax] by allowing to
/// change the order of the `query` and `fragment` component!
///
/// Refer to the [alpm-package-source] documentation for a more detailed overview of the custom URL
/// syntax.
///
/// [URL syntax]: https://en.wikipedia.org/wiki/URL#Syntax
/// [VCS]: https://en.wikipedia.org/wiki/Version_control
/// [alpm-package-source]: https://alpm.archlinux.page/specifications/alpm-package-source.7.html
///
/// ## Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::SourceUrl;
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // Create Url from &str
/// let url =
///     SourceUrl::from_str("git+https://your-vcs.org/example-project.git?signed#tag=v1.0.0")?;
/// assert_eq!(
///     &url.to_string(),
///     "git+https://your-vcs.org/example-project.git?signed#tag=v1.0.0"
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceUrl {
    /// The URL from where the sources are retrieved.
    pub url: Url,
    /// Optional data on VCS systems using the URL for the retrieval of sources.
    pub vcs_info: Option<VcsInfo>,
}

impl FromStr for SourceUrl {
    type Err = Error;

    /// Creates a new `SourceUrl` instance from a string slice.
    ///
    /// ## Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::SourceUrl;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let url =
    ///     SourceUrl::from_str("git+https://your-vcs.org/example-project.git?signed#tag=v1.0.0")?;
    /// assert_eq!(
    ///     &url.to_string(),
    ///     "git+https://your-vcs.org/example-project.git?signed#tag=v1.0.0"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser_until_eof.parse(s)?)
    }
}

impl Display for SourceUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // If there's no vcs info, print the URL and return.
        let Some(vcs_info) = &self.vcs_info else {
            return write!(f, "{}", self.url.as_str());
        };

        let mut prefix = None;
        let url = self.url.as_str();
        let mut formatted_fragment = String::new();
        let mut query = String::new();

        // Build all components of a source url, based on the protocol and provided options
        match vcs_info {
            VcsInfo::Bzr { fragment } => {
                prefix = Some(VcsProtocol::Bzr);
                if let Some(fragment) = fragment {
                    formatted_fragment = format!("#{fragment}");
                }
            }
            VcsInfo::Fossil { fragment } => {
                prefix = Some(VcsProtocol::Fossil);
                if let Some(fragment) = fragment {
                    formatted_fragment = format!("#{fragment}");
                }
            }
            VcsInfo::Git { fragment, signed } => {
                // Only add the protocol prefix if the URL doesn't already encode the protocol
                if !url.starts_with("git://") {
                    prefix = Some(VcsProtocol::Git);
                }
                if *signed {
                    query = "?signed".to_string();
                }
                if let Some(fragment) = fragment {
                    formatted_fragment = format!("#{fragment}");
                }
            }
            VcsInfo::Hg { fragment } => {
                prefix = Some(VcsProtocol::Hg);
                if let Some(fragment) = fragment {
                    formatted_fragment = format!("#{fragment}");
                }
            }
            VcsInfo::Svn { fragment } => {
                // Only add the prefix if the URL doesn't already encode the protocol
                if !url.starts_with("svn://") {
                    prefix = Some(VcsProtocol::Svn);
                }
                if let Some(fragment) = fragment {
                    formatted_fragment = format!("#{fragment}");
                }
            }
        }

        let prefix = if let Some(prefix) = prefix {
            format!("{prefix}+")
        } else {
            String::new()
        };

        write!(f, "{prefix}{url}{query}{formatted_fragment}",)
    }
}

/// For SourceUrl, we only define a [`ParserUntil`] trait and not the `AlpmParser` trait, as we
/// don't provide the [`Url`] type parser ourselves. Hence, the indicator for its supposed "end"
/// must be provided by the caller of the parser.
impl ParserUntil for SourceUrl {
    /// Recognizes an [`SourceUrl`] in an input string until a given `delimiter`.
    ///
    /// # Errors
    ///
    /// Returns an error if the immediate start of `input` does not a contain a valid `SourceUrl`,
    /// followed by the specified delimiter.
    fn parser_until<'a, P>(delimiter: P) -> impl Parser<&'a str, Self, ErrMode<ContextError>>
    where
        P: Parser<&'a str, &'a str, ErrMode<ContextError>>,
    {
        // Define the actual parser closure.
        // The delimiter is moved into the closure and borrowed via `by_ref()` on each call.
        let mut delimiter = delimiter;
        move |input: &mut &'a str| -> ModalResult<Self> {
            // Check if we should use a VCS for this URL.
            let vcs = opt(VcsProtocol::parser).parse_next(input)?;

            let Some(vcs) = vcs else {
                // If there's no VCS, simply interpret the rest of the string as a URL.
                //
                // We explicitly don't look for ALPM related fragments or queries, as the fragment
                // and query might be a part of the inner URL string for retrieving
                // the sources.
                let url = rest
                    .try_map(Url::from_str)
                    .context(StrContext::Label("url"))
                    .parse_next(input)?;
                return Ok(SourceUrl {
                    url,
                    vcs_info: None,
                });
            };

            // We now know that we look at a URL that's supposed to be used by a VCS.
            // Get the URL first, error if we cannot find it.
            // Recognizes a URL in an alpm-package-source string.
            //
            // Considers all chars until a special char or the EOF is encountered:
            // - `#` character that indicates a fragment
            // - `?` character indicates a query
            // - `EOF` we reached the end of the string.
            //
            // All of the above indicate that the end of the URL has been reached.
            // The `#` or `?` are not consumed, so that an outer parser may continue parsing
            // afterwards.
            let url = repeat_till(0.., any, peek(alt(("#", "?", delimiter.by_ref()))))
                .map(|((), _): ((), &str)| ())
                .take()
                .try_map(|url: &str| Url::from_str(url))
                .context(StrContext::Label("url"))
                .parse_next(input)?;

            let vcs_info = VcsInfo::parser(vcs).parse_next(input)?;

            // Produce a special error message for unconsumed query parameters.
            // The unused result with error type are necessary to please the type checker.
            not("?")
                .context(StrContext::Label(
                    "or duplicate query parameter for detected VCS.",
                ))
                .parse_next(input)?;

            delimiter
                .by_ref()
                .context(StrContext::Label("unexpected trailing content in URL."))
                .context(StrContext::Expected(StrContextValue::Description(
                    "end of input.",
                )))
                .parse_next(input)?;

            Ok(SourceUrl {
                url,
                vcs_info: Some(vcs_info),
            })
        }
    }
}

/// Information on Version Control Systems (VCS) using a URL.
///
/// Several different VCS systems can be used in the context of a [`SourceUrl`].
/// Each system supports addressing different types of objects and may optionally require signature
/// verification for those objects.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "protocol", rename_all = "lowercase")]
pub enum VcsInfo {
    /// Bazaar/Breezy VCS information.
    Bzr {
        /// Optional URL fragment information.
        fragment: Option<BzrFragment>,
    },
    /// Fossil VCS information.
    Fossil {
        /// Optional URL fragment information.
        fragment: Option<FossilFragment>,
    },
    /// Git VCS information.
    Git {
        /// Optional URL fragment information.
        fragment: Option<GitFragment>,
        /// Whether OpenPGP signature verification is required.
        signed: bool,
    },
    /// Mercurial VCS information.
    Hg {
        /// Optional URL fragment information.
        fragment: Option<HgFragment>,
    },
    /// Apache Subversion VCS information.
    Svn {
        /// Optional URL fragment information.
        fragment: Option<SvnFragment>,
    },
}

impl VcsInfo {
    /// Recognizes VCS-specific URL fragment and query based on a [`VcsProtocol`].
    ///
    /// As the parser is parameterized due to the earlier detected [`VcsProtocol`], it returns a
    /// new stateful parser closure.
    fn parser(vcs: VcsProtocol) -> impl FnMut(&mut &str) -> ModalResult<VcsInfo> {
        move |input: &mut &str| match vcs {
            VcsProtocol::Bzr => {
                let fragment = BzrFragment::parser.parse_next(input)?;
                Ok(VcsInfo::Bzr { fragment })
            }
            VcsProtocol::Fossil => {
                let fragment = FossilFragment::parser.parse_next(input)?;
                Ok(VcsInfo::Fossil { fragment })
            }
            VcsProtocol::Git => {
                // Pacman actually allows a parameter **after** the fragment, which is
                // theoretically an invalid URL.
                // Hence, we have to check for the parameter before and after the url.
                let mut signed = git_query(input)?;
                let fragment = GitFragment::parser.parse_next(input)?;
                if !signed {
                    // Check for the theoretically invalid query after the fragment if it wasn't
                    // already at the front.
                    signed = git_query(input)?;
                }
                Ok(VcsInfo::Git { fragment, signed })
            }
            VcsProtocol::Hg => {
                let fragment = HgFragment::parser.parse_next(input)?;
                Ok(VcsInfo::Hg { fragment })
            }
            VcsProtocol::Svn => {
                let fragment = SvnFragment::parser.parse_next(input)?;
                Ok(VcsInfo::Svn { fragment })
            }
        }
    }
}

/// A VCS protocol
///
/// This identifier is only used during parsing to have some static representation of the detected
/// VCS.
/// This is necessary as the fragment and the query are parsed at a later step and we have to
/// keep track of the VCS somehow.
#[derive(strum::Display, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
enum VcsProtocol {
    Bzr,
    Fossil,
    Git,
    Hg,
    Svn,
}

impl VcsProtocol {
    /// Parses the start of an alpm-package-source string to determine the VCS protocol in use.
    ///
    /// VCS protocol information is used in [`SourceUrl`]s and can be detected in the following
    /// ways:
    ///
    /// - An explicit VCS protocol identifier, followed by a literal `+`. E.g. `git+https://...`, `svn+https://...`
    /// - Some VCS (i.e. git and svn) support URLs in which their protocol type is exposed in the
    ///   `scheme` component of the URL itself:
    ///    - `git://...`
    ///    - `svn://...`
    fn parser(input: &mut &str) -> ModalResult<VcsProtocol> {
        // Check for an explicit vcs definition like `git+` first.
        let protocol =
            opt(terminated(alpha1.try_map(VcsProtocol::from_str), "+")).parse_next(input)?;

        if let Some(protocol) = protocol {
            return Ok(protocol);
        }

        // We didn't find any explicit identifiers.
        // Now see if we find any vcs protocol at the start of the URL.
        // Make sure to **not** consume anything from inside URL!
        //
        // If this doesn't find anything, it backtracks to the parent function.
        let protocol = peek(alt(("git://", "svn://"))).parse_next(input)?;

        match protocol {
            "git://" => Ok(VcsProtocol::Git),
            "svn://" => Ok(VcsProtocol::Svn),
            _ => unreachable!(),
        }
    }
}

/// Parses the value of a URL fragment from an alpm-package-source string.
///
/// Parsing is attempted after the URL fragment type has been determined.
///
/// E.g. `tag=v1.0.0`
///           ^^^^^^
///          This part
fn fragment_value(input: &mut &str) -> ModalResult<String> {
    // Error if we don't find the separator
    let _ = "="
        .context(StrContext::Label("fragment separator"))
        .context(StrContext::Expected(StrContextValue::Description(
            "a literal '='",
        )))
        .parse_next(input)?;

    // Get the value of the fragment.
    let (value, _) = repeat_till(0.., any, peek(alt(("?", "#", eof)))).parse_next(input)?;

    Ok(value)
}

/// The available URL fragments and their values when using the Breezy VCS in a [`SourceUrl`].
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BzrFragment {
    /// A specific revision in the repository.
    Revision(String),
}

impl Display for BzrFragment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BzrFragment::Revision(revision) => write!(f, "revision={revision}"),
        }
    }
}

impl BzrFragment {
    /// Recognizes URL fragments and values specific to Breezy VCS.
    ///
    /// This parser considers all variants of [`BzrFragment`] (including a leading `#` character).
    fn parser(input: &mut &str) -> ModalResult<Option<BzrFragment>> {
        // Check for the `#` fragment start first. If it isn't here, there's no fragment.
        let exists = opt("#").parse_next(input)?;
        if exists.is_none() {
            return Ok(None);
        }

        // Expect the only allowed revision keyword.
        "revision"
            .context(StrContext::Label("bzr revision type"))
            .context(StrContext::Expected(StrContextValue::Description(
                "revision keyword",
            )))
            .parse_next(input)?;

        let value = fragment_value.parse_next(input)?;

        Ok(Some(BzrFragment::Revision(value)))
    }
}

/// The available URL fragments and their values when using the Fossil VCS in a [`SourceUrl`].
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FossilFragment {
    /// A specific branch in the repository.
    Branch(String),
    /// A specific commit in the repository.
    Commit(String),
    /// A specific tag in the repository.
    Tag(String),
}

impl Display for FossilFragment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FossilFragment::Branch(revision) => write!(f, "branch={revision}"),
            FossilFragment::Commit(revision) => write!(f, "commit={revision}"),
            FossilFragment::Tag(revision) => write!(f, "tag={revision}"),
        }
    }
}

impl FossilFragment {
    /// Recognizes URL fragments and values specific to Fossil VCS.
    ///
    /// This parser considers all variants of [`FossilFragment`] as fragments in an
    /// alpm-package-source string (including the leading `#` character).
    fn parser(input: &mut &str) -> ModalResult<Option<FossilFragment>> {
        // Check for the `#` fragment start first. If it isn't here, there's no fragment.
        let exists = opt("#").parse_next(input)?;
        if exists.is_none() {
            return Ok(None);
        }

        // Error if we don't find one of the expected fossil revision types.
        let version_keywords = ["branch", "commit", "tag"];
        let version_type = alt(version_keywords)
            .context(StrContext::Label("fossil revision type"))
            .context_with(iter_str_context!([version_keywords]))
            .parse_next(input)?;

        let value = fragment_value.parse_next(input)?;

        let fragment = match version_type {
            "branch" => FossilFragment::Branch(value.to_string()),
            "commit" => FossilFragment::Commit(value.to_string()),
            "tag" => FossilFragment::Tag(value.to_string()),
            _ => unreachable!(),
        };

        Ok(Some(fragment))
    }
}

/// The available URL fragments and their values when using the Git VCS in a [`SourceUrl`].
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GitFragment {
    /// A specific branch in the repository.
    Branch(String),
    /// A specific commit in the repository.
    Commit(String),
    /// A specific tag in the repository.
    Tag(String),
}

impl Display for GitFragment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GitFragment::Branch(revision) => write!(f, "branch={revision}"),
            GitFragment::Commit(revision) => write!(f, "commit={revision}"),
            GitFragment::Tag(revision) => write!(f, "tag={revision}"),
        }
    }
}

impl GitFragment {
    /// Recognizes URL fragments and values specific to the Git VCS.
    ///
    /// This parser considers all variants of [`GitFragment`] as fragments in an alpm-package-source
    /// string (including the leading `#` character).
    fn parser(input: &mut &str) -> ModalResult<Option<GitFragment>> {
        // Check for the `#` fragment start first. If it isn't here, there's no fragment.
        let exists = opt("#").parse_next(input)?;
        if exists.is_none() {
            return Ok(None);
        }

        // Error if we don't find one of the expected git revision types.
        let version_keywords = ["branch", "commit", "tag"];
        let version_type = alt(version_keywords)
            .context(StrContext::Label("git revision type"))
            .context_with(iter_str_context!([version_keywords]))
            .parse_next(input)?;

        let value = fragment_value.parse_next(input)?;

        let fragment = match version_type {
            "branch" => GitFragment::Branch(value.to_string()),
            "commit" => GitFragment::Commit(value.to_string()),
            "tag" => GitFragment::Tag(value.to_string()),
            _ => unreachable!(),
        };

        Ok(Some(fragment))
    }
}

/// Recognizes URL queries specific to the Git VCS.
///
/// This parser considers the `?signed` URL query in an alpm-package-source string.
fn git_query(input: &mut &str) -> ModalResult<bool> {
    let query = opt("?signed").parse_next(input)?;
    Ok(query.is_some())
}

/// An optional version specification used in a [`SourceUrl`] for the Hg VCS.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HgFragment {
    /// A specific branch in the repository.
    Branch(String),
    /// A specific revision in the repository.
    Revision(String),
    /// A specific tag in the repository.
    Tag(String),
}

impl Display for HgFragment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HgFragment::Branch(revision) => write!(f, "branch={revision}"),
            HgFragment::Revision(revision) => write!(f, "revision={revision}"),
            HgFragment::Tag(revision) => write!(f, "tag={revision}"),
        }
    }
}

impl HgFragment {
    /// Recognizes URL fragments and values specific to the Mercurial VCS.
    ///
    /// This parser considers all variants of [`HgFragment`] as fragments in an alpm-package-source
    /// string (including the leading `#` character).
    fn parser(input: &mut &str) -> ModalResult<Option<HgFragment>> {
        // Check for the `#` fragment start first. If it isn't here, there's no fragment.
        let exists = opt("#").parse_next(input)?;
        if exists.is_none() {
            return Ok(None);
        }

        // Error if we don't find one of the expected git revision types.
        let version_keywords = ["branch", "revision", "tag"];
        let version_type = alt(version_keywords)
            .context(StrContext::Label("hg revision type"))
            .context_with(iter_str_context!([version_keywords]))
            .parse_next(input)?;

        let value = fragment_value.parse_next(input)?;

        let fragment = match version_type {
            "branch" => HgFragment::Branch(value.to_string()),
            "revision" => HgFragment::Revision(value.to_string()),
            "tag" => HgFragment::Tag(value.to_string()),
            _ => unreachable!(),
        };

        Ok(Some(fragment))
    }
}

/// The available URL fragments and their values when using Apache Subversion in a [`SourceUrl`].
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SvnFragment {
    /// A specific revision in the repository.
    Revision(String),
}

impl Display for SvnFragment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SvnFragment::Revision(revision) => write!(f, "revision={revision}"),
        }
    }
}

impl SvnFragment {
    /// Recognizes URL fragments and values specific to Apache Subversion.
    ///
    /// This parser considers all variants of [`SvnFragment`] as fragments in an alpm-package-source
    /// string (including the leading `#` character).
    fn parser(input: &mut &str) -> ModalResult<Option<SvnFragment>> {
        // Check for the `#` fragment start first. If it isn't here, there's no fragment.
        let exists = opt("#").parse_next(input)?;
        if exists.is_none() {
            return Ok(None);
        }

        // Expect the only allowed revision keyword.
        "revision"
            .context(StrContext::Label("svn revision type"))
            .context(StrContext::Expected(StrContextValue::Description(
                "revision keyword",
            )))
            .parse_next(input)?;

        let value = fragment_value.parse_next(input)?;

        Ok(Some(SvnFragment::Revision(value)))
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use rstest::rstest;
    use testresult::TestResult;

    use super::*;
    use crate::configure_insta;

    #[rstest]
    #[case("https://example.com/", Ok("https://example.com/"))]
    #[case(
        "https://example.com/path?query=1",
        Ok("https://example.com/path?query=1")
    )]
    #[case("ftp://example.com/", Ok("ftp://example.com/"))]
    #[case("not-a-url", Err(url::ParseError::RelativeUrlWithoutBase.into()))]
    fn test_url_parsing(#[case] input: &str, #[case] expected: Result<&str, Error>) {
        let result = input.parse::<Url>();
        assert_eq!(
            result.as_ref().map(|v| v.to_string()),
            expected.as_ref().map(|v| v.to_string())
        );

        if let Ok(url) = result {
            assert_eq!(url.as_str(), input);
        }
    }

    #[rstest]
    #[case(
        "git+https://example/project#tag=v1.0.0?signed",
        Some("git+https://example/project?signed#tag=v1.0.0"),
        SourceUrl {
            url: Url::from_str("https://example/project").unwrap(),
            vcs_info: Some(VcsInfo::Git {
                fragment: Some(GitFragment::Tag("v1.0.0".to_string())),
                signed: true
            })
        }
    )]
    #[case(
        "git+https://example/project?signed#tag=v1.0.0",
        None,
        SourceUrl {
            url: Url::from_str("https://example/project").unwrap(),
            vcs_info: Some(VcsInfo::Git {
                fragment: Some(GitFragment::Tag("v1.0.0".to_string())),
                signed: true
            })
        }
    )]
    #[case(
        "git://example/project#commit=a51720b",
        None,
        SourceUrl {
            url: Url::from_str("git://example/project").unwrap(),
            vcs_info: Some(VcsInfo::Git {
                fragment: Some(GitFragment::Commit("a51720b".to_string())),
                signed: false
            })
        }
    )]
    #[case(
        "svn+https://example/project#revision=a51720b",
        None,
        SourceUrl {
            url: Url::from_str("https://example/project").unwrap(),
            vcs_info: Some(VcsInfo::Svn {
                fragment: Some(SvnFragment::Revision("a51720b".to_string())),
            })
        }
    )]
    #[case(
        "bzr+https://example/project#revision=a51720b",
        None,
        SourceUrl {
            url: Url::from_str("https://example/project").unwrap(),
            vcs_info: Some(VcsInfo::Bzr {
                fragment: Some(BzrFragment::Revision("a51720b".to_string())),
            })
        }
    )]
    #[case(
        "hg+https://example/project#branch=feature",
        None,
        SourceUrl {
            url: Url::from_str("https://example/project").unwrap(),
            vcs_info: Some(VcsInfo::Hg {
                fragment: Some(HgFragment::Branch("feature".to_string())),
            })
        }
    )]
    #[case(
        "fossil+https://example/project#branch=feature",
        None,
        SourceUrl {
            url: Url::from_str("https://example/project").unwrap(),
            vcs_info: Some(VcsInfo::Fossil {
                fragment: Some(FossilFragment::Branch("feature".to_string())),
            })
        }
    )]
    #[case(
        "https://example/project#branch=feature?signed",
        None,
        SourceUrl {
            url: Url::from_str("https://example/project#branch=feature?signed").unwrap(),
            vcs_info: None,
        }
    )]
    fn test_source_url_parsing_success(
        #[case] input: &str,
        #[case] expected_to_string: Option<&str>,
        #[case] expected: SourceUrl,
    ) -> TestResult {
        let source_url = SourceUrl::from_str(input)?;
        assert_eq!(
            source_url, expected,
            "Parsed source_url should resemble the expected output."
        );

        // Some representations are shortened or brought into the proper representation, hence we
        // have a slightly different ToString output than input.
        let expected_to_string = expected_to_string.unwrap_or(input);
        assert_eq!(
            source_url.to_string(),
            expected_to_string,
            "Parsed and displayed source_url should resemble original."
        );

        Ok(())
    }

    /// Run the parser for SourceUrl and ensure that the expected parse error messages show up.
    #[rstest]
    #[case("git+https://example/project#revision=v1.0.0?signed")]
    #[case("git+https://example/project#branch=feature#branch=feature")]
    #[case("git+https://example/project#branch=feature?signed?signed")]
    #[case("bzr+https://example/project#branch=feature")]
    #[case("svn+https://example/project#branch=feature")]
    #[case("hg+https://example/project#commit=154021a")]
    #[case("hg+https://example/project#branch=feature?signed")]
    fn test_source_url_parsing_failure(#[case] input: &str) {
        let Err(Error::ParseError(err_msg)) = SourceUrl::from_str(input) else {
            panic!("'{input}' erroneously parsed as a SourceUrl")
        };

        let (test_name, _guard) = configure_insta();
        assert_snapshot!(test_name, err_msg.to_string());
    }
}
