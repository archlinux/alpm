use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    string::ToString,
};

use serde::{Deserialize, Serialize};

use crate::{Architecture, Name, Version, error::Error};

/// A makepkg option
///
/// This type is used in the context of `makepkg` options, build environment options
/// ([`BuildEnvironmentOption`]), and package options ([`PackageOption`]).
///
/// See [the makepkg.conf manpage](https://man.archlinux.org/man/makepkg.conf.5.en) for more information.
///
/// ## Examples
/// ```
/// # fn main() -> Result<(), alpm_types::Error> {
/// use alpm_types::MakepkgOption;
///
/// let option = MakepkgOption::new("strip")?;
/// assert_eq!(option.on(), true);
/// assert_eq!(option.name(), "strip");
///
/// let not_option = MakepkgOption::new("!zipman")?;
/// assert_eq!(not_option.on(), false);
/// assert_eq!(not_option.name(), "zipman");
///
/// let invalid_option = MakepkgOption::new("!foo");
/// assert!(invalid_option.is_err());
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MakepkgOption {
    // BUILD ENVIRONMENT
    /// Use the Distributed C/C++/ObjC compiler
    Distcc(bool),
    /// Colorize output messages
    Color(bool),
    /// Use ccache to cache compilation
    Ccache(bool),
    /// Run the check() function if present in the PKGBUILD
    Check(bool),
    /// Generate PGP signature file
    Sign(bool),
    // OPTIONS
    /// Strip symbols from binaries/libraries
    Strip(bool),
    /// Save doc directories specified by DOC_DIRS
    Docs(bool),
    /// Leave libtool (.la) files in packages
    Libtool(bool),
    /// Leave static library (.a) files in packages
    StaticLibs(bool),
    /// Leave empty directories in packages
    EmptyDirs(bool),
    /// Compress manual (man and info) pages in MAN_DIRS with gzip
    Zipman(bool),
    /// Remove files specified by PURGE_TARGETS
    Purge(bool),
    /// Add debugging flags as specified in DEBUG_* variables
    Debug(bool),
    /// Add compile flags for building with link time optimization
    Lto(bool),
    /// Automatically add depends/provides
    AutoDeps(bool),
}

impl MakepkgOption {
    /// Create a new MakepkgOption in a Result
    pub fn new(option: &str) -> Result<Self, Error> {
        Self::from_str(option)
    }

    /// Get the name of the MakepkgOption
    pub fn name(&self) -> &str {
        match self {
            MakepkgOption::Distcc(_) => "distcc",
            MakepkgOption::Color(_) => "color",
            MakepkgOption::Ccache(_) => "ccache",
            MakepkgOption::Check(_) => "check",
            MakepkgOption::Sign(_) => "sign",
            MakepkgOption::Strip(_) => "strip",
            MakepkgOption::Docs(_) => "docs",
            MakepkgOption::Libtool(_) => "libtool",
            MakepkgOption::StaticLibs(_) => "staticlibs",
            MakepkgOption::EmptyDirs(_) => "emptydirs",
            MakepkgOption::Zipman(_) => "zipman",
            MakepkgOption::Purge(_) => "purge",
            MakepkgOption::Debug(_) => "debug",
            MakepkgOption::Lto(_) => "lto",
            MakepkgOption::AutoDeps(_) => "autodeps",
        }
    }

    /// Get whether the MakepkgOption is on
    pub fn on(&self) -> bool {
        match self {
            MakepkgOption::Distcc(on)
            | MakepkgOption::Color(on)
            | MakepkgOption::Ccache(on)
            | MakepkgOption::Check(on)
            | MakepkgOption::Sign(on)
            | MakepkgOption::Strip(on)
            | MakepkgOption::Docs(on)
            | MakepkgOption::Libtool(on)
            | MakepkgOption::StaticLibs(on)
            | MakepkgOption::EmptyDirs(on)
            | MakepkgOption::Zipman(on)
            | MakepkgOption::Purge(on)
            | MakepkgOption::Debug(on)
            | MakepkgOption::Lto(on)
            | MakepkgOption::AutoDeps(on) => *on,
        }
    }
}

impl FromStr for MakepkgOption {
    type Err = Error;
    /// Create an Option from a string
    fn from_str(s: &str) -> Result<MakepkgOption, Self::Err> {
        let (name, on) = if let Some(name) = s.strip_prefix('!') {
            (name.to_owned(), false)
        } else {
            (s.to_owned(), true)
        };
        if let Some(c) = name
            .chars()
            .find(|c| !(c.is_alphanumeric() || ['-', '.', '_'].contains(c)))
        {
            return Err(Error::ValueContainsInvalidChars { invalid_char: c });
        }
        match name.as_str() {
            "distcc" => Ok(MakepkgOption::Distcc(on)),
            "color" => Ok(MakepkgOption::Color(on)),
            "ccache" => Ok(MakepkgOption::Ccache(on)),
            "check" => Ok(MakepkgOption::Check(on)),
            "sign" => Ok(MakepkgOption::Sign(on)),
            "strip" => Ok(MakepkgOption::Strip(on)),
            "docs" => Ok(MakepkgOption::Docs(on)),
            "libtool" => Ok(MakepkgOption::Libtool(on)),
            "staticlibs" => Ok(MakepkgOption::StaticLibs(on)),
            "emptydirs" => Ok(MakepkgOption::EmptyDirs(on)),
            "zipman" => Ok(MakepkgOption::Zipman(on)),
            "purge" => Ok(MakepkgOption::Purge(on)),
            "debug" => Ok(MakepkgOption::Debug(on)),
            "lto" => Ok(MakepkgOption::Lto(on)),
            "autodeps" => Ok(MakepkgOption::AutoDeps(on)),
            _ => Err(Error::InvalidMakepkgOption(name)),
        }
    }
}

impl Display for MakepkgOption {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}{}", if self.on() { "" } else { "!" }, self.name())
    }
}

/// An option string used in a build environment
///
/// The option string is identified by its name and whether it is on (not prefixed with "!") or off
/// (prefixed with "!"). This type is an alias for [`MakepkgOption`].
///
/// ## Examples
/// ```
/// # fn main() -> Result<(), alpm_types::Error> {
/// use alpm_types::BuildEnvironmentOption;
///
/// let option = BuildEnvironmentOption::new("distcc")?;
/// assert_eq!(option.on(), true);
/// assert_eq!(option.name(), "distcc");
///
/// let not_option = BuildEnvironmentOption::new("!ccache")?;
/// assert_eq!(not_option.on(), false);
/// assert_eq!(not_option.name(), "ccache");
/// # Ok(())
/// # }
/// ```
pub type BuildEnvironmentOption = MakepkgOption;

/// An option string used in packaging
///
/// The option string is identified by its name and whether it is on (not prefixed with "!") or off
/// (prefixed with "!"). This type is an alias for [`MakepkgOption`].
///
/// ## Examples
/// ```
/// # fn main() -> Result<(), alpm_types::Error> {
/// use alpm_types::PackageOption;
///
/// let option = PackageOption::new("debug")?;
/// assert_eq!(option.on(), true);
/// assert_eq!(option.name(), "debug");
///
/// let not_option = PackageOption::new("!lto")?;
/// assert_eq!(not_option.on(), false);
/// assert_eq!(not_option.name(), "lto");
/// # Ok(())
/// # }
/// ```
pub type PackageOption = MakepkgOption;

/// Information on an installed package in an environment
///
/// Tracks a `Name`, `Version` (which is guaranteed to have a `PackageRelease`) and `Architecture`
/// of a package in an environment.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::InstalledPackage;
///
/// assert!(InstalledPackage::from_str("foo-bar-1:1.0.0-1-any").is_ok());
/// assert!(InstalledPackage::from_str("foo-bar-1:1.0.0-1").is_err());
/// assert!(InstalledPackage::from_str("foo-bar-1:1.0.0-any").is_err());
/// assert!(InstalledPackage::from_str("1:1.0.0-1-any").is_err());
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct InstalledPackage {
    name: Name,
    version: Version,
    architecture: Architecture,
}

impl InstalledPackage {
    /// Create a new InstalledPackage
    pub fn new(name: Name, version: Version, architecture: Architecture) -> Result<Self, Error> {
        Ok(InstalledPackage {
            name,
            version,
            architecture,
        })
    }
}

impl FromStr for InstalledPackage {
    type Err = Error;
    /// Create an Installed from a string
    fn from_str(s: &str) -> Result<InstalledPackage, Self::Err> {
        const DELIMITER: char = '-';
        let mut parts = s.rsplitn(4, DELIMITER);

        let architecture = parts.next().ok_or(Error::MissingComponent {
            component: "architecture",
        })?;
        let architecture = architecture.parse()?;
        let version = {
            let Some(pkgrel) = parts.next() else {
                return Err(Error::MissingComponent {
                    component: "pkgrel",
                })?;
            };
            let Some(epoch_pkgver) = parts.next() else {
                return Err(Error::MissingComponent {
                    component: "epoch_pkgver",
                })?;
            };
            epoch_pkgver.to_string() + "-" + pkgrel
        };
        let name = parts
            .next()
            .ok_or(Error::MissingComponent { component: "name" })?
            .to_string();

        Ok(InstalledPackage {
            name: Name::new(&name)?,
            version: Version::with_pkgrel(version.as_str())?,
            architecture,
        })
    }
}

impl Display for InstalledPackage {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}-{}-{}", self.name, self.version, self.architecture)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("strip", Ok(MakepkgOption::Strip(true)))]
    #[case("docs", Ok(MakepkgOption::Docs(true)))]
    #[case("!libtool", Ok(MakepkgOption::Libtool(false)))]
    #[case("staticlibs", Ok(MakepkgOption::StaticLibs(true)))]
    #[case("distcc", Ok(MakepkgOption::Distcc(true)))]
    #[case("!invalid", Err(Error::InvalidMakepkgOption("invalid".to_string())))]
    fn makepkgoption(#[case] s: &str, #[case] result: Result<MakepkgOption, Error>) {
        assert_eq!(MakepkgOption::from_str(s), result);
    }

    #[rstest]
    #[case(
        "foo-bar-1:1.0.0-1-any",
        Ok(InstalledPackage{
            name: Name::new("foo-bar").unwrap(),
            version: Version::from_str("1:1.0.0-1").unwrap(),
            architecture: Architecture::Any,
        }),
    )]
    #[case("foo-bar-1:1.0.0-1", Err(strum::ParseError::VariantNotFound.into()))]
    #[case("1:1.0.0-1-any", Err(Error::MissingComponent { component: "name" }))]
    fn installed_new(#[case] s: &str, #[case] result: Result<InstalledPackage, Error>) {
        assert_eq!(InstalledPackage::from_str(s), result);
    }

    #[rstest]
    #[case("foo-1:1.0.0-bar-any", "invalid package release")]
    #[case("packagename-30-0.1oops-any", "expected end of package release value")]
    #[case("package$with$dollars-30-0.1-any", "invalid character in package name")]
    fn installed_new_parse_error(#[case] input: &str, #[case] error_snippet: &str) {
        let result = InstalledPackage::from_str(input);
        assert!(result.is_err(), "Expected InstalledPackage parsing to fail");
        let err = result.unwrap_err();
        let pretty_error = err.to_string();
        assert!(
            pretty_error.contains(error_snippet),
            "Error:\n=====\n{pretty_error}\n=====\nshould contain snippet:\n\n{error_snippet}"
        );
    }
}
