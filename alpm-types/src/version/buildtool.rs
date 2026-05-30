//! Build tool related version handling.

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use alpm_parsers::prelude::*;
use serde::Serialize;
use winnow::{
    Parser,
    combinator::opt,
    error::{ErrMode, StrContext, StrContextValue},
};

#[cfg(doc)]
use crate::BuildTool;
use crate::{Architecture, Error, FullVersion, MinimalVersion, Version};

/// The version and optional architecture of a build tool.
///
/// [`BuildToolVersion`] is used in conjunction with [`BuildTool`] to denote the specific build tool
/// a package is built with.
/// [`BuildToolVersion`] distinguishes between two types of representations:
///
/// - the one used by [makepkg], which relies on [`MinimalVersion`]
/// - and the one used by [pkgctl] (devtools), which relies on [`FullVersion`] and the
///   [`Architecture`] of the build tool.
///
/// For more information refer to the `buildtoolver` keyword in [BUILDINFOv2].
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Architecture, BuildToolVersion, FullVersion, MinimalVersion};
///
/// # fn main() -> testresult::TestResult {
/// // Representation used by makepkg
/// assert_eq!(
///     BuildToolVersion::from_str("1.0.0")?,
///     BuildToolVersion::Makepkg(MinimalVersion::from_str("1.0.0")?)
/// );
/// assert_eq!(
///     BuildToolVersion::from_str("1:1.0.0")?,
///     BuildToolVersion::Makepkg(MinimalVersion::from_str("1:1.0.0")?)
/// );
///
/// // Representation used by pkgctl
/// assert_eq!(
///     BuildToolVersion::from_str("1.0.0-1-any")?,
///     BuildToolVersion::DevTools {
///         version: FullVersion::from_str("1.0.0-1")?,
///         architecture: Architecture::from_str("any")?
///     }
/// );
/// assert_eq!(
///     BuildToolVersion::from_str("1:1.0.0-1-any")?,
///     BuildToolVersion::DevTools {
///         version: FullVersion::from_str("1:1.0.0-1")?,
///         architecture: Architecture::from_str("any")?
///     }
/// );
/// # Ok(())
/// # }
/// ```
///
/// [BUILDINFOv2]: https://alpm.archlinux.page/specifications/BUILDINFOv2.5.html
/// [makepkg]: https://man.archlinux.org/man/makepkg.8
/// [pkgctl]: https://man.archlinux.org/man/pkgctl.1
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum BuildToolVersion {
    /// The version representation used by [makepkg].
    ///
    /// [makepkg]: https://man.archlinux.org/man/makepkg.8
    Makepkg(MinimalVersion),
    /// The version representation used by [pkgctl] (devtools).
    ///
    /// [pkgctl]: https://man.archlinux.org/man/pkgctl.1
    DevTools {
        /// The (_full_ or _full with epoch_) version of the build tool.
        version: FullVersion,
        /// The architecture of the build tool.
        architecture: Architecture,
    },
}

impl BuildToolVersion {
    /// Returns the optional [`Architecture`].
    ///
    /// # Note
    ///
    /// If `self` is a [`BuildToolVersion::Makepkg`] this method always returns [`None`].
    pub fn architecture(&self) -> Option<Architecture> {
        if let Self::DevTools {
            version: _,
            architecture,
        } = self
        {
            Some(architecture.clone())
        } else {
            None
        }
    }

    /// Returns a [`Version`] that matches the underlying [`MinimalVersion`] or [`FullVersion`].
    pub fn version(&self) -> Version {
        match self {
            Self::Makepkg(version) => Version::from(version),
            Self::DevTools {
                version,
                architecture: _,
            } => Version::from(version),
        }
    }
}

impl AlpmParser for BuildToolVersion {
    /// Recognizes a [`BuildToolVersion`] in a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` does not begin with a `BuildToolVersion`.
    // TODO: Wrap this parser in a layer closure
    fn parser<'a>(input: &mut Input<'a>) -> PResult<'a, Self> {
        // The start can either be:
        // - A minimal version (no pkgrel, thereby shorter)
        // - A full version together with an `-` and an architecture.
        //
        // Since the FullVersion is longer, we can use it to determine what kind of input we can
        // expect.
        let full_version = opt(FullVersion::parser).parse_next(input)?;

        if let Some(version) = full_version {
            "-".context(StrContext::Label("buildtool version"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "'-' delimiter between full alpm-package-version and alpm-architecture",
                )))
                .parse_next(input)?;

            let architecture = Architecture::parser.parse_next(input)?;
            return Ok(BuildToolVersion::DevTools {
                version,
                architecture,
            });
        }

        let minimal_version =  MinimalVersion::parser
            .context(StrContext::Expected(StrContextValue::Description("a stand-alone minimal alpm-package-version")))
            .context(StrContext::Expected(StrContextValue::Description("or a full alpm-package-version together with a alpm-architecture, delimited by a '-'")))
            .parse_next(input)?;

        Ok(BuildToolVersion::Makepkg(minimal_version))
    }

    fn delimiter_error_context<'a, O, P>(
        parser: P,
    ) -> impl Parser<Input<'a>, O, ErrMode<ParseStack<'a>>>
    where
        P: Parser<Input<'a>, O, ErrMode<ParseStack<'a>>>,
    {
        parser
            .context(StrContext::Expected(StrContextValue::Description("a stand-alone minimal alpm-package-version")))
            .context(StrContext::Expected(StrContextValue::Description("or a full alpm-package-version together with a alpm-architecture, delimited by a '-'")))
            .layer("buildtool version")
    }
}

impl FromStr for BuildToolVersion {
    type Err = Error;
    /// Creates a [`BuildToolVersion`] from a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - `s` contains no '-' and `s` is not a valid [`MinimalVersion`],
    /// - or `s` contains at least one '-' and after splitting on the right most occurrence, either
    ///   the left-hand side is not a valid [`FullVersion`] or the right hand side is not a valid
    ///   [`Architecture`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser_until_eof.parse(Input::new(s))?)
    }
}

impl Display for BuildToolVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Makepkg(version) => write!(f, "{version}"),
            Self::DevTools {
                version,
                architecture,
            } => write!(f, "{version}-{architecture}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use rstest::rstest;
    use testresult::TestResult;

    use super::*;
    use crate::configure_insta;

    /// Ensure that valid strings are correctly parsed as [`BuildToolVersion`] and invalid ones lead
    /// to an [`Error`].
    #[rstest]
    #[case::devtools_full(
        "1.0.0-1-any",
        BuildToolVersion::DevTools{version: FullVersion::from_str("1.0.0-1")?, architecture: Architecture::from_str("any")?},
    )]
    #[case::devtools_full_with_epoch(
        "1:1.0.0-1-any",
        BuildToolVersion::DevTools{version: FullVersion::from_str("1:1.0.0-1")?, architecture: Architecture::from_str("any")?},
    )]
    #[case::makepkg_minimal(
        "1.0.0",
        BuildToolVersion::Makepkg(MinimalVersion::from_str("1.0.0")?),
    )]
    #[case::makepkg_minimal_with_epoch(
        "1:1.0.0",
        BuildToolVersion::Makepkg(MinimalVersion::from_str("1:1.0.0")?),
    )]
    fn valid_buildtool_version(
        #[case] input: &str,
        #[case] expected: BuildToolVersion,
    ) -> TestResult {
        let version = match BuildToolVersion::from_str(input) {
            Ok(version) => version,
            Err(err) => {
                panic!("Expected BuildToolVersion parsing of string {input} to succeed:\n{err}")
            }
        };

        assert_eq!(
            version, expected,
            "Expected '{expected:#?}' when parsing '{input}' but got '{version:#?}'"
        );

        Ok(())
    }

    #[rstest]
    #[case::full_version_with_architecture("1.0.0-any")]
    #[case::minimal_version_with_epoch_and_architecture("1:1.0.0-any")]
    #[case::bad_package_version("ß-1-any")]
    fn invalid_buildtool_version(#[case] input: &str) -> TestResult {
        let err = match BuildToolVersion::from_str(input) {
            Err(err) => err,
            Ok(_) => {
                panic!("Expected BuildToolVersion parsing of string {input} to fail")
            }
        };

        let (test_name, _guard) = configure_insta();
        assert_snapshot!(test_name, err.to_string());

        Ok(())
    }
}
