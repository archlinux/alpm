use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::Serialize;
use winnow::ascii::space0;
use winnow::combinator::fail;
use winnow::error::StrContextValue;
use winnow::token::rest;
use winnow::{
    combinator::{alt, cut_err, eof, opt, peek, repeat_till, terminated},
    error::StrContext,
    token::any,
    ModalResult,
    Parser,
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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

/// Represents a package source related URL.
///
/// It is used to represent the source URL scheme used in the ALPM packaging processes, which
/// hijacks and extends the typical URL scheme to some degree.
///
/// There's additional packaging related information embedded into these URLs, which looks like
/// this:
///
/// ```txt
/// git+https://some.domain/example-project.git#tag=v1.0.0?signed
/// ```
///
/// - `git+` Specify whether and which VSC should be used
/// - `#tag=v1.0.0` (called fragment), specifies what version should be checked out by a VCS.
/// - `?signed` additional options that may be applied per VCS.
///
/// For full information, check out the man page of `PKGBUILD` and `alpm-package-source`.
///
/// The `SourceUrl` type wraps the [`Url`] type and extracts .
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
///     SourceUrl::from_str("git+https://your-vcs.org/example-project.git#tag=v1.0.0?signed")?;
/// assert_eq!(
///     &url.to_string(),
///     "git+https://your-vcs.org/example-project.git#tag=v1.0.0?signed"
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceUrl {
    /// The actual URL from where the sources may be pulled.
    pub url: Url,
    /// Additionally encoded data specific to
    pub vcs_info: Option<SourceVcsInfo>,
}

impl FromStr for SourceUrl {
    type Err = Error;

    /// Creates a new `Url` instance from a string slice.
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
    ///     SourceUrl::from_str("git+https://your-vcs.org/example-project.git#tag=v1.0.0?signed")?;
    /// assert_eq!(
    ///     &url.to_string(),
    ///     "git+https://your-vcs.org/example-project.git#tag=v1.0.0?signed"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match source_url.parse(s) {
            Ok(source_url) => Ok(source_url),
            Err(error) => Err(Error::ParseError(error.to_string())),
        }
    }
}

impl Display for SourceUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // If there's no vcs info, print the URL and return.
        let Some(vcs_info) = &self.vcs_info else {
            return write!(f, "{}", self.url.as_str());
        };

        let mut prefix = "";
        let url = self.url.as_str();
        let mut formatted_fragment = String::new();
        let mut query = String::new();

        // Build all components of a source url, based on the protocol and provided options
        match vcs_info {
            SourceVcsInfo::Bzr { fragment } => {
                prefix = "bzr+";
                if let Some(fragment) = fragment {
                    formatted_fragment = format!("#{fragment}");
                }
            }
            SourceVcsInfo::Fossil { fragment } => {
                prefix = "fossil+";
                if let Some(fragment) = fragment {
                    formatted_fragment = format!("#{fragment}");
                }
            }
            SourceVcsInfo::Git { fragment, signed } => {
                // Only add the prefix if the URL doesn't already encode the protocol
                if !url.starts_with("git://") {
                    prefix = "git+";
                }
                if let Some(fragment) = fragment {
                    formatted_fragment = format!("#{fragment}");
                }
                if *signed {
                    query = "?signed".to_string();
                }
            }
            SourceVcsInfo::Hg { fragment } => {
                prefix = "hg+";
                if let Some(fragment) = fragment {
                    formatted_fragment = format!("#{fragment}");
                }
            }
            SourceVcsInfo::Svn { fragment } => {
                // Only add the prefix if the URL doesn't already encode the protocol
                if !url.starts_with("svn://") {
                    prefix = "svn+";
                }
                if let Some(fragment) = fragment {
                    formatted_fragment = format!("#{fragment}");
                }
            }
        }

        write!(f, "{prefix}{url}{formatted_fragment}{query}",)
    }
}

/// Parse the actual url from a [`SourceUrl`].
/// That is all chars until a special char or the EOF is hit:
// - `#` character that indicates a fragment
// - `?` character indicates a query
// - `EOF` we reached the end of the string.
//
// All indicate that we reached the end of the URL.
// Don't consume the `#` or `?` so that an outer parser may continue parsing afterwards.
fn source_inner_url(input: &mut &str) -> ModalResult<String> {
    let (url, _) = repeat_till(0.., any, peek(alt(("#", "?", eof)))).parse_next(input)?;
    Ok(url)
}

/// Parse a full SourceUrl.
fn source_url(input: &mut &str) -> ModalResult<SourceUrl> {
    // Check if we should use a VCS for this URL.
    let vcs = opt(vcs_identifier).parse_next(input)?;

    let Some(vcs) = vcs else {
        // If there's no VCS, simply interpret the rest of the string as an URL.
        //
        // We explicitly don't look for ALPM related fragments or queries, as the fragment and
        // query might be an actual part URL to get the sources.
        let url = cut_err(rest.try_map(Url::from_str))
            .context(StrContext::Label("url"))
            .parse_next(input)?;
        return Ok(SourceUrl {
            url,
            vcs_info: None,
        });
    };

    // We now know that we look at an URL that's supposed to be used by a VCS.
    // Get the URL first, error if we cannot find it.
    let url = cut_err(source_inner_url.try_map(|url| Url::from_str(&url)))
        .context(StrContext::Label("url"))
        .parse_next(input)?;

    let vcs_info = source_vcs_info(vcs).parse_next(input)?;

    // Produce a special error message for unconsumed query parameters.
    // The unused result with error type are necessary to please the type checker.
    let _: Option<String> = opt(("?", rest).take().and_then(cut_err(
        fail.context(StrContext::Label("query parameter for detected VCS.")),
    )))
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "protocol", rename_all = "lowercase")]
pub enum SourceVcsInfo {
    Bzr {
        fragment: Option<BzrFragment>,
    },
    Fossil {
        fragment: Option<FossilFragment>,
    },
    Git {
        fragment: Option<GitFragment>,
        signed: bool,
    },
    Hg {
        fragment: Option<HgFragment>,
    },
    Svn {
        fragment: Option<SvnFragment>,
    },
}

/// Parse the VCS related fragment and query based on the vcs that was detected at the URL's start.
///
/// As the parser is parameterized due to the earlier detected [`VcsIdentifier`], it returns a new
/// stateful parser closure.
fn source_vcs_info(vcs: VcsIdentifier) -> impl FnMut(&mut &str) -> ModalResult<SourceVcsInfo> {
    move |input: &mut &str| match vcs {
        VcsIdentifier::Bzr => {
            let fragment = opt(bzr_fragment).parse_next(input)?;
            Ok(SourceVcsInfo::Bzr { fragment })
        }
        VcsIdentifier::Fossil => {
            let fragment = opt(fossil_fragment).parse_next(input)?;
            Ok(SourceVcsInfo::Fossil { fragment })
        }
        VcsIdentifier::Git => {
            // Pacman actually allows a parameter **after** the fragment, which is
            // theoretically an invalid URL.
            // Hence, we have to check for the parameter before and after the url.
            let signed_before = git_query(input)?;
            let fragment = opt(git_fragment).parse_next(input)?;
            let signed_after = git_query(input)?;
            Ok(SourceVcsInfo::Git {
                fragment,
                signed: signed_before || signed_after,
            })
        }
        VcsIdentifier::Hg => {
            let fragment = opt(hg_fragment).parse_next(input)?;
            Ok(SourceVcsInfo::Hg { fragment })
        }
        VcsIdentifier::Svn => {
            let fragment = opt(svn_fragment).parse_next(input)?;
            Ok(SourceVcsInfo::Svn { fragment })
        }
    }
}

/// This identifier is only used during parsing to have some static representation of the detected
/// VCS. This is necessary as the fragment and the query are parsed at a later step and we have to
/// keep track of the VCS somehow.
enum VcsIdentifier {
    Bzr,
    Fossil,
    Git,
    Hg,
    Svn,
}

/// Parse the start of a SourceUrl to determine the VCS that's in use.
///
/// This is done by looking at two things.
/// 1. An explicit VCS identifier, which is the name of the vcs, followed by a literal `+`. E.g. `git+https://...`,
///    `svn+https://...`
/// 2. Some VCS support their type baked into the protocol of the URL itself. These are git and svn:
///    - `git://...`
///    - `svn://...`
fn vcs_identifier(input: &mut &str) -> ModalResult<VcsIdentifier> {
    // Check for an explicit vcs definition first.
    let identifier =
        opt(terminated(alt(("bzr", "fossil", "git", "hg", "svn")), "+")).parse_next(input)?;

    if let Some(identifier) = identifier {
        match identifier {
            "bzr" => return Ok(VcsIdentifier::Bzr),
            "fossil" => return Ok(VcsIdentifier::Fossil),
            "git" => return Ok(VcsIdentifier::Git),
            "hg" => return Ok(VcsIdentifier::Hg),
            "svn" => return Ok(VcsIdentifier::Svn),
            _ => unreachable!(),
        }
    }

    // We didn't find any explicit identifiers.
    // Now see if we find any vcs protocol at the start of the URL.
    // Make sure to **not** consume anything from inside URL!
    //
    // If this doesn't find anything, it backtracks to the parent function.
    let protocol = peek(alt(("git://", "svn://"))).parse_next(input)?;

    match protocol {
        "git://" => Ok(VcsIdentifier::Git),
        "svn://" => Ok(VcsIdentifier::Svn),
        _ => unreachable!(),
    }
}

/// Get the fragment value after the version type has been determined.
/// E.g. `tag=v1.0.0`
///          ^^^^^^^
///          This part
fn fragment_value(input: &mut &str) -> ModalResult<String> {
    // Error if we don't find the separator
    let _ = cut_err("=")
        .context(StrContext::Label("fragment separator"))
        .context(StrContext::Expected(StrContextValue::Description(
            "an literal '='",
        )))
        .parse_next(input)?;

    // Get the value of the fragment.
    let (value, _) = repeat_till(0.., any, peek(alt(("?", eof)))).parse_next(input)?;

    Ok(value)
}

/// An optional version specification used in a [`SourceUrl`] for the Bzr VCS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BzrFragment {
    Revision(String),
}

impl Display for BzrFragment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BzrFragment::Revision(revision) => write!(f, "revision={revision}"),
        }
    }
}

/// Parse a Bzr URL fragment, including the preceding `#`.
fn bzr_fragment(input: &mut &str) -> ModalResult<BzrFragment> {
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

/// An optional version specification used in a [`SourceUrl`] for the Fossil VCS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FossilFragment {
    Branch(String),
    Commit(String),
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

/// Parse a Fossil URL fragment, including the preceding `#`.
fn fossil_fragment(input: &mut &str) -> ModalResult<FossilFragment> {
    // Check for the `#` fragment start first. If it isn't here, backtrack.
    let _ = "#".parse_next(input)?;

    // Error if we don't find one of the expected fossil revision types.
    let version_type = cut_err(alt(("branch", "commit", "tag")))
        .context(StrContext::Label("fossil revision type"))
        .context(StrContext::Expected(StrContextValue::Description(
            "branch, commit or tag keyword",
        )))
        .parse_next(input)?;

    let value = fragment_value.parse_next(input)?;

    match version_type {
        "branch" => Ok(FossilFragment::Branch(value.to_string())),
        "commit" => Ok(FossilFragment::Commit(value.to_string())),
        "tag" => Ok(FossilFragment::Tag(value.to_string())),
        _ => unreachable!(),
    }
}

/// An optional version specification used in a [`SourceUrl`] for the Git VCS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GitFragment {
    Branch(String),
    Commit(String),
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

/// Parse a Git URL fragment, including the preceding `#`.
fn git_fragment(input: &mut &str) -> ModalResult<GitFragment> {
    // Check for the `#` fragment start first. If it isn't here, backtrack.
    let _ = "#".parse_next(input)?;

    // Error if we don't find one of the expected git revision types.
    let version_type = cut_err(alt(("branch", "commit", "tag")))
        .context(StrContext::Label("git revision type"))
        .context(StrContext::Expected(StrContextValue::Description(
            "branch, commit or tag keyword",
        )))
        .parse_next(input)?;

    let value = fragment_value.parse_next(input)?;

    match version_type {
        "branch" => Ok(GitFragment::Branch(value.to_string())),
        "commit" => Ok(GitFragment::Commit(value.to_string())),
        "tag" => Ok(GitFragment::Tag(value.to_string())),
        _ => unreachable!(),
    }
}

/// Parse the optional Git URL `?signed` query
fn git_query(input: &mut &str) -> ModalResult<bool> {
    let query = opt("?signed").parse_next(input)?;
    Ok(query.is_some())
}

/// An optional version specification used in a [`SourceUrl`] for the Hg VCS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HgFragment {
    Branch(String),
    Revision(String),
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

/// Parse a Hg URL fragment, including the preceding `#`.
fn hg_fragment(input: &mut &str) -> ModalResult<HgFragment> {
    // Check for the `#` fragment start first. If it isn't here, backtrack.
    let _ = "#".parse_next(input)?;

    // Error if we don't find one of the expected git revision types.
    let version_type = cut_err(alt(("branch", "revision", "tag")))
        .context(StrContext::Label("hg revision type"))
        .context(StrContext::Expected(StrContextValue::Description(
            "branch, revision or tag keyword",
        )))
        .parse_next(input)?;

    let value = fragment_value.parse_next(input)?;

    match version_type {
        "branch" => Ok(HgFragment::Branch(value.to_string())),
        "revision" => Ok(HgFragment::Revision(value.to_string())),
        "tag" => Ok(HgFragment::Tag(value.to_string())),
        _ => unreachable!(),
    }
}

/// An optional version specification used in a [`SourceUrl`] for the Svn VCS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SvnFragment {
    Revision(String),
}

impl Display for SvnFragment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SvnFragment::Revision(revision) => write!(f, "revision={revision}"),
        }
    }
}

/// Parse a Svn URL fragment, including the preceding `#`.
fn svn_fragment(input: &mut &str) -> ModalResult<SvnFragment> {
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
        SourceUrl {
            url: Url::from_str("https://example/project").unwrap(),
            vcs_info: Some(SourceVcsInfo::Git {
                fragment: Some(GitFragment::Tag("v1.0.0".to_string())),
                signed: true
            })
        }
    )]
    #[case(
        "git://example/project#commit=a51720b",
        SourceUrl {
            url: Url::from_str("git://example/project").unwrap(),
            vcs_info: Some(SourceVcsInfo::Git {
                fragment: Some(GitFragment::Commit("a51720b".to_string())),
                signed: false
            })
        }
    )]
    #[case(
        "svn+https://example/project#revision=a51720b",
        SourceUrl {
            url: Url::from_str("https://example/project").unwrap(),
            vcs_info: Some(SourceVcsInfo::Svn {
                fragment: Some(SvnFragment::Revision("a51720b".to_string())),
            })
        }
    )]
    #[case(
        "bzr+https://example/project#revision=a51720b",
        SourceUrl {
            url: Url::from_str("https://example/project").unwrap(),
            vcs_info: Some(SourceVcsInfo::Bzr {
                fragment: Some(BzrFragment::Revision("a51720b".to_string())),
            })
        }
    )]
    #[case(
        "hg+https://example/project#branch=feature",
        SourceUrl {
            url: Url::from_str("https://example/project").unwrap(),
            vcs_info: Some(SourceVcsInfo::Hg {
                fragment: Some(HgFragment::Branch("feature".to_string())),
            })
        }
    )]
    #[case(
        "fossil+https://example/project#branch=feature",
        SourceUrl {
            url: Url::from_str("https://example/project").unwrap(),
            vcs_info: Some(SourceVcsInfo::Fossil {
                fragment: Some(FossilFragment::Branch("feature".to_string())),
            })
        }
    )]
    #[case(
        "https://example/project#branch=feature?signed",
        SourceUrl {
            url: Url::from_str("https://example/project#branch=feature?signed").unwrap(),
            vcs_info: None,
        }
    )]
    fn test_source_url_parsing_success(
        #[case] input: &str,
        #[case] expected: SourceUrl,
    ) -> TestResult {
        let source_url = SourceUrl::from_str(input)?;
        assert_eq!(
            source_url, expected,
            "Parsed source_url should resemble the expected output."
        );
        assert_eq!(
            source_url.to_string(),
            input,
            "Parsed and displayed source_url should resemble original."
        );

        Ok(())
    }

    /// Run the parser for SourceUrl and ensure that the expected parse error messages show up.
    #[rstest]
    #[case(
        "git+https://example/project#revision=v1.0.0?signed",
        "invalid git revision type\nexpected branch, commit or tag keyword"
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
        "invalid hg revision type\nexpected branch, revision or tag keyword"
    )]
    #[case(
        "hg+https://example/project#branch=feature?signed",
        "invalid query parameter for detected VCS."
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
