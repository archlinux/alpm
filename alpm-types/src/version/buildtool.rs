//! Build tool related version handling.

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::Serialize;

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
        match s.rsplit_once('-') {
            Some((version, architecture)) => Ok(BuildToolVersion::DevTools {
                version: FullVersion::from_str(version)?,
                architecture: Architecture::from_str(architecture)?,
            }),
            None => Ok(BuildToolVersion::Makepkg(MinimalVersion::from_str(s)?)),
        }
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
    use rstest::rstest;
    use testresult::TestResult;

    use super::*;

    /// Ensure that valid strings are correctly parsed as [`BuildToolVersion`] and invalid ones lead
    /// to an [`Error`].
    #[rstest]
    #[case::devtools_full(
        "1.0.0-1-any",
        Ok(BuildToolVersion::DevTools{version: FullVersion::from_str("1.0.0-1")?, architecture: Architecture::from_str("any")?}),
    )]
    #[case::devtools_full_with_epoch(
        "1:1.0.0-1-any",
        Ok(BuildToolVersion::DevTools{version: FullVersion::from_str("1:1.0.0-1")?, architecture: Architecture::from_str("any")?}),
    )]
    #[case::makepkg_minimal(
        "1.0.0",
        Ok(BuildToolVersion::Makepkg(MinimalVersion::from_str("1.0.0")?)),
    )]
    #[case::makepkg_minimal_with_epoch(
        "1:1.0.0",
        Ok(BuildToolVersion::Makepkg(MinimalVersion::from_str("1:1.0.0")?)),
    )]
    #[case::minimal_version_with_architecture("1.0.0-any",
        Err(Error::ParseError(
            "1.0.0\n^\nexpected alpm-pkgver string, followed by a '-' and an alpm-pkgrel string".to_string()
        ))
     )]
    #[case::minimal_version_with_epoch_and_architecture(
        "1:1.0.0-any",
        Err(Error::ParseError(
            "1:1.0.0\n  ^\nexpected alpm-pkgver string, followed by a '-' and an alpm-pkgrel string".to_string()
        ))
    )]
    fn valid_buildtoolver_new(
        #[case] input: &str,
        #[case] expected: Result<BuildToolVersion, Error>,
    ) -> TestResult {
        let parse_result = BuildToolVersion::from_str(input);
        assert_eq!(
            parse_result, expected,
            "Expected '{expected:?}' when parsing '{input}' but got '{parse_result:?}'"
        );

        Ok(())
    }

    /// Ensures that [`BuildToolVersion::from_str`] fails on invalid version strings with specific
    /// errors.
    #[rstest]
    #[case::minimal_version_with_architecture(
        "1.0.0-any",
        Error::ParseError(
            "1.0.0\n^\nexpected alpm-pkgver string, followed by a '-' and an alpm-pkgrel string".to_string()
        )
    )]
    #[case::minimal_version_with_unknown_architecture(
        "1.0.0-foo",
        Error::ParseError(
            "1.0.0\n^\nexpected alpm-pkgver string, followed by a '-' and an alpm-pkgrel string".to_string()
        )
    )]
    fn invalid_buildtoolver_new(#[case] buildtoolver: &str, #[case] expected: Error) {
        assert_eq!(
            BuildToolVersion::from_str(buildtoolver),
            Err(expected),
            "Expected error during parse of buildtoolver '{buildtoolver}'"
        );
    }

    #[rstest]
    #[case("ß-1-any", "invalid pkgver character")]
    fn invalid_buildtoolver_badpkgver(#[case] buildtoolver: &str, #[case] err_snippet: &str) {
        let Err(Error::ParseError(err_msg)) = BuildToolVersion::from_str(buildtoolver) else {
            panic!("'{buildtoolver}' erroneously parsed as BuildToolVersion")
        };
        assert!(
            err_msg.contains(err_snippet),
            "Error:\n=====\n{err_msg}\n=====\nshould contain snippet:\n\n{err_snippet}"
        );
    }
}
