//! Build tool related version handling.

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::Serialize;

use crate::{Architecture, Error, Version};

/// The version and architecture of a build tool
///
/// `BuildToolVersion` is used in conjunction with `BuildTool` to denote the specific build tool a
/// package is built with. A `BuildToolVersion` wraps a `Version` (that is guaranteed to have a
/// `PackageRelease`) and an `Architecture`.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::BuildToolVersion;
///
/// assert!(BuildToolVersion::from_str("1-1-any").is_ok());
/// assert!(BuildToolVersion::from_str("1").is_ok());
/// assert!(BuildToolVersion::from_str("1-1").is_err());
/// assert!(BuildToolVersion::from_str("1-1-foo").is_err());
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct BuildToolVersion {
    version: Version,
    architecture: Option<Architecture>,
}

impl BuildToolVersion {
    /// Create a new BuildToolVersion
    pub fn new(version: Version, architecture: Option<Architecture>) -> Self {
        BuildToolVersion {
            version,
            architecture,
        }
    }

    /// Return a reference to the Architecture
    pub fn architecture(&self) -> &Option<Architecture> {
        &self.architecture
    }

    /// Return a reference to the Version
    pub fn version(&self) -> &Version {
        &self.version
    }
}

impl FromStr for BuildToolVersion {
    type Err = Error;
    /// Create an BuildToolVersion from a string and return it in a Result
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const VERSION_DELIMITER: char = '-';
        match s.rsplit_once(VERSION_DELIMITER) {
            Some((version, architecture)) => match Architecture::from_str(architecture) {
                Ok(architecture) => Ok(BuildToolVersion {
                    version: Version::with_pkgrel(version)?,
                    architecture: Some(architecture),
                }),
                Err(err) => Err(err.into()),
            },
            None => Ok(BuildToolVersion {
                version: Version::from_str(s)?,
                architecture: None,
            }),
        }
    }
}

impl Display for BuildToolVersion {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        if let Some(architecture) = &self.architecture {
            write!(fmt, "{}-{}", self.version, architecture)
        } else {
            write!(fmt, "{}", self.version)
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    /// Ensure that valid buildtool version strings are parsed as expected.
    #[rstest]
    #[case(
        "1.0.0-1-any",
        BuildToolVersion::new(Version::from_str("1.0.0-1").unwrap(), Some(Architecture::from_str("any").unwrap())),
    )]
    #[case(
        "1:1.0.0-1-any",
        BuildToolVersion::new(Version::from_str("1:1.0.0-1").unwrap(), Some(Architecture::from_str("any").unwrap())),
    )]
    #[case(
        "1.0.0",
        BuildToolVersion::new(Version::from_str("1.0.0").unwrap(), None),
    )]
    fn valid_buildtoolver_new(#[case] buildtoolver: &str, #[case] expected: BuildToolVersion) {
        assert_eq!(
            BuildToolVersion::from_str(buildtoolver),
            Ok(expected),
            "Expected valid parse of buildtoolver '{buildtoolver}'"
        );
    }

    /// Ensure that invalid buildtool version strings produce the respective errors.
    #[rstest]
    #[case("1.0.0-any", Error::MissingComponent { component: "pkgrel" })]
    #[case("1.0.0-1-foo", strum::ParseError::VariantNotFound.into())]
    fn invalid_buildtoolver_new(#[case] buildtoolver: &str, #[case] expected: Error) {
        assert_eq!(
            BuildToolVersion::from_str(buildtoolver),
            Err(expected),
            "Expected error during parse of buildtoolver '{buildtoolver}'"
        );
    }

    #[rstest]
    #[case(".1.0.0-1-any", "invalid first pkgver character")]
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
