//! Types for handling URLs and VCS-related information in package sources.

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use alpm_parsers::iter_str_context;
use serde::{Deserialize, Serialize};
use winnow::{
    ModalResult,
    Parser,
    ascii::{alpha1, space0},
    combinator::{alt, cut_err, eof, fail, opt, peek, repeat_till, terminated},
    error::{StrContext, StrContextValue},
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
        Ok(Self::parser.parse(s)?)
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

impl SourceUrl {
    /// Parses a full [`SourceUrl`] from a string slice.
    fn parser(input: &mut &str) -> ModalResult<SourceUrl> {
        // Check if we should use a VCS for this URL.
        let vcs = opt(VcsProtocol::parser).parse_next(input)?;

        let Some(vcs) = vcs else {
            // If there's no VCS, simply interpret the rest of the string as a URL.
            //
            // We explicitly don't look for ALPM related fragments or queries, as the fragment and
            // query might be a part of the inner URL string for retrieving the sources.
            let url = cut_err(rest.try_map(Url::from_str))
                .context(StrContext::Label("url"))
                .parse_next(input)?;
            return Ok(SourceUrl {
                url,
                vcs_info: None,
            });
        };

        // We now know that we look at a URL that's supposed to be used by a VCS.
        // Get the URL first, error if we cannot find it.
        let url = cut_err(SourceUrl::inner_url_parser.try_map(|url| Url::from_str(&url)))
            .context(StrContext::Label("url"))
            .parse_next(input)?;

        let vcs_info = VcsInfo::parser(vcs).parse_next(input)?;

        // Produce a special error message for unconsumed query parameters.
        // The unused result with error type are necessary to please the type checker.
        let _: Option<String> =
            opt(("?", rest)
                .take()
                .and_then(cut_err(fail.context(StrContext::Label(
                    "or duplicate query parameter for detected VCS.",
                )))))
            .parse_next(input)?;

        cut_err((space0, eof))
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

    /// Recognizes a URL in an alpm-package-source string.
    ///
    /// Considers all chars until a special char or the EOF is encountered:
    /// - `#` character that indicates a fragment
    /// - `?` character indicates a query
    /// - `EOF` we reached the end of the string.
    ///
    /// All of the above indicate that the end of the URL has been reached.
    /// The `#` or `?` are not consumed, so that an outer parser may continue parsing afterwards.
    fn inner_url_parser(input: &mut &str) -> ModalResult<String> {
        let (url, _) = repeat_till(0.., any, peek(alt(("#", "?", eof)))).parse_next(input)?;
        Ok(url)
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
                let fragment = opt(BzrFragment::parser).parse_next(input)?;
                Ok(VcsInfo::Bzr { fragment })
            }
            VcsProtocol::Fossil => {
                let fragment = opt(FossilFragment::parser).parse_next(input)?;
                Ok(VcsInfo::Fossil { fragment })
            }
            VcsProtocol::Git => {
                // Pacman actually allows a parameter **after** the fragment, which is
                // theoretically an invalid URL.
                // Hence, we have to check for the parameter before and after the url.
                let mut signed = git_query(input)?;
                let fragment = opt(GitFragment::parser).parse_next(input)?;
                if !signed {
                    // Check for the theoretically invalid query after the fragment if it wasn't
                    // already at the front.
                    signed = git_query(input)?;
                }
                Ok(VcsInfo::Git { fragment, signed })
            }
            VcsProtocol::Hg => {
                let fragment = opt(HgFragment::parser).parse_next(input)?;
                Ok(VcsInfo::Hg { fragment })
            }
            VcsProtocol::Svn => {
                let fragment = opt(SvnFragment::parser).parse_next(input)?;
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
    let _ = cut_err("=")
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
    fn parser(input: &mut &str) -> ModalResult<BzrFragment> {
        // Check for the `#` fragment start first. If it isn't here, backtrack.
        let _ = "#".parse_next(input)?;

        // Expect the only allowed revision keyword.
        cut_err("revision")
            .context(StrContext::Label("bzr revision type"))
            .context(StrContext::Expected(StrContextValue::Description(
                "revision keyword",
            )))
            .parse_next(input)?;

        let value = fragment_value.parse_next(input)?;

        Ok(BzrFragment::Revision(value))
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
    fn parser(input: &mut &str) -> ModalResult<FossilFragment> {
        // Check for the `#` fragment start first. If it isn't here, backtrack.
        let _ = "#".parse_next(input)?;

        // Error if we don't find one of the expected fossil revision types.
        let version_keywords = ["branch", "commit", "tag"];
        let version_type = cut_err(alt(version_keywords))
            .context(StrContext::Label("fossil revision type"))
            .context_with(iter_str_context!([version_keywords]))
            .parse_next(input)?;

        let value = fragment_value.parse_next(input)?;

        match version_type {
            "branch" => Ok(FossilFragment::Branch(value.to_string())),
            "commit" => Ok(FossilFragment::Commit(value.to_string())),
            "tag" => Ok(FossilFragment::Tag(value.to_string())),
            _ => unreachable!(),
        }
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
    fn parser(input: &mut &str) -> ModalResult<GitFragment> {
        // Check for the `#` fragment start first. If it isn't here, backtrack.
        let _ = "#".parse_next(input)?;

        // Error if we don't find one of the expected git revision types.
        let version_keywords = ["branch", "commit", "tag"];
        let version_type = cut_err(alt(version_keywords))
            .context(StrContext::Label("git revision type"))
            .context_with(iter_str_context!([version_keywords]))
            .parse_next(input)?;

        let value = fragment_value.parse_next(input)?;

        match version_type {
            "branch" => Ok(GitFragment::Branch(value.to_string())),
            "commit" => Ok(GitFragment::Commit(value.to_string())),
            "tag" => Ok(GitFragment::Tag(value.to_string())),
            _ => unreachable!(),
        }
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
    fn parser(input: &mut &str) -> ModalResult<HgFragment> {
        // Check for the `#` fragment start first. If it isn't here, backtrack.
        let _ = "#".parse_next(input)?;

        // Error if we don't find one of the expected git revision types.
        let version_keywords = ["branch", "revision", "tag"];
        let version_type = cut_err(alt(version_keywords))
            .context(StrContext::Label("hg revision type"))
            .context_with(iter_str_context!([version_keywords]))
            .parse_next(input)?;

        let value = fragment_value.parse_next(input)?;

        match version_type {
            "branch" => Ok(HgFragment::Branch(value.to_string())),
            "revision" => Ok(HgFragment::Revision(value.to_string())),
            "tag" => Ok(HgFragment::Tag(value.to_string())),
            _ => unreachable!(),
        }
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
    fn parser(input: &mut &str) -> ModalResult<SvnFragment> {
        // Check for the `#` fragment start first. If it isn't here, backtrack.
        let _ = "#".parse_next(input)?;

        // Expect the only allowed revision keyword.
        cut_err("revision")
            .context(StrContext::Label("svn revision type"))
            .context(StrContext::Expected(StrContextValue::Description(
                "revision keyword",
            )))
            .parse_next(input)?;

        let value = fragment_value.parse_next(input)?;

        Ok(SvnFragment::Revision(value))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use testresult::TestResult;

    use super::*;

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
    #[case(
        "git+https://example/project#revision=v1.0.0?signed",
        "invalid git revision type\nexpected `branch`, `commit`, `tag`"
    )]
    #[case(
        "git+https://example/project#branch=feature#branch=feature",
        "invalid unexpected trailing content in URL."
    )]
    #[case(
        "git+https://example/project#branch=feature?signed?signed",
        "invalid or duplicate query parameter for detected VCS."
    )]
    #[case(
        "bzr+https://example/project#branch=feature",
        "invalid bzr revision type\nexpected revision keyword"
    )]
    #[case(
        "svn+https://example/project#branch=feature",
        "invalid svn revision type\nexpected revision keyword"
    )]
    #[case(
        "hg+https://example/project#commit=154021a",
        "invalid hg revision type\nexpected `branch`, `revision`, `tag`"
    )]
    #[case(
        "hg+https://example/project#branch=feature?signed",
        "invalid or duplicate query parameter for detected VCS."
    )]
    fn test_source_url_parsing_failure(#[case] input: &str, #[case] error_snippet: &str) {
        let result = SourceUrl::from_str(input);
        assert!(result.is_err(), "Invalid source_url should fail to parse.");
        let err = result.unwrap_err();
        let pretty_error = err.to_string();
        assert!(
            pretty_error.contains(error_snippet),
            "Error:\n=====\n{pretty_error}\n=====\nshould contain snippet:\n\n{error_snippet}"
        );
    }
}
