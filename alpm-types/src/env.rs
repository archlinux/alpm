use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    string::ToString,
};

use alpm_parsers::iter_char_context;
use serde::{Deserialize, Serialize};
use winnow::{
    ModalResult,
    Parser,
    combinator::{Repeat, eof, opt, repeat},
    error::{StrContext, StrContextValue},
    token::one_of,
};

use crate::{Architecture, Name, Version, error::Error};

/// Parser function for makepkg options.
///
/// The parser will return a tuple containing the option name and a boolean indicating whether the
/// option is "on" (`true`) or "off" (`false`).
///
/// # Format
///
/// The parser expects a string that starts with an optional "!" character, followed by a sequence
/// of ASCII alphanumeric characters, hyphens, dots, or underscores.
///
/// # Errors
///
/// If the input string does not match the expected format, an error will be returned.
fn makepkg_option_parser(input: &mut &str) -> ModalResult<(String, bool)> {
    let on = opt('!').parse_next(input)?.is_none();
    let alphanum = |c: char| c.is_ascii_alphanumeric();
    let special_chars = ['-', '.', '_'];
    let valid_chars = one_of((alphanum, special_chars));
    let option_name: Repeat<_, _, _, (), _> = repeat(0.., valid_chars);
    let full_parser = (
        option_name,
        eof.context(StrContext::Label("character in makepkg option"))
            .context(StrContext::Expected(StrContextValue::Description(
                "ASCII alphanumeric character",
            )))
            .context_with(iter_char_context!(special_chars)),
    );
    full_parser
        .take()
        .map(|n: &str| (n.to_owned(), on))
        .parse_next(input)
}

/// An option string used in a build environment
///
/// The option string is identified by its name and whether it is on (not prefixed with "!") or off
/// (prefixed with "!").
///
/// See [the makepkg.conf manpage](https://man.archlinux.org/man/makepkg.conf.5.en) for more information.
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
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildEnvironmentOption {
    /// Use ccache to cache compilation
    Ccache(bool),
    /// Run the check() function if present in the PKGBUILD
    Check(bool),
    /// Colorize output messages
    Color(bool),
    /// Use the Distributed C/C++/ObjC compiler
    Distcc(bool),
    /// Generate PGP signature file
    Sign(bool),
}

impl BuildEnvironmentOption {
    /// Create a new [`BuildEnvironmentOption`] in a Result
    ///
    /// # Errors
    ///
    /// An error is returned if the string slice does not match a valid build environment option.
    pub fn new(option: &str) -> Result<Self, Error> {
        Self::from_str(option)
    }

    /// Get the name of the MakepkgOption
    pub fn name(&self) -> &str {
        match self {
            Self::Distcc(_) => "distcc",
            Self::Color(_) => "color",
            Self::Ccache(_) => "ccache",
            Self::Check(_) => "check",
            Self::Sign(_) => "sign",
        }
    }

    /// Get whether the BuildEnvironmentOption is on
    pub fn on(&self) -> bool {
        match self {
            Self::Distcc(on)
            | Self::Color(on)
            | Self::Ccache(on)
            | Self::Check(on)
            | Self::Sign(on) => *on,
        }
    }
}

impl FromStr for BuildEnvironmentOption {
    type Err = Error;
    /// Create an Option from a string
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, on) = makepkg_option_parser.parse(s)?;
        match name.as_str() {
            "distcc" => Ok(Self::Distcc(on)),
            "color" => Ok(Self::Color(on)),
            "ccache" => Ok(Self::Ccache(on)),
            "check" => Ok(Self::Check(on)),
            "sign" => Ok(Self::Sign(on)),
            _ => Err(Error::InvalidBuildEnvironmentOption(name)),
        }
    }
}

impl Display for BuildEnvironmentOption {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}{}", if self.on() { "" } else { "!" }, self.name())
    }
}

/// An option string used in packaging
///
/// The option string is identified by its name and whether it is on (not prefixed with "!") or off
/// (prefixed with "!").
///
/// See [the makepkg.conf manpage](https://man.archlinux.org/man/makepkg.conf.5.en) for more information.
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
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageOption {
    /// Automatically add dependencies and provisions (see [alpm-sonamev2]).
    ///
    /// [alpm-sonamev2]: https://alpm.archlinux.page/specifications/alpm-sonamev2.7.html
    AutoDeps(bool),

    /// Add debugging flags as specified in DEBUG_* variables
    Debug(bool),

    /// Save doc directories specified by DOC_DIRS
    Docs(bool),

    /// Leave empty directories in packages
    EmptyDirs(bool),

    /// Leave libtool (.la) files in packages
    Libtool(bool),

    /// Add compile flags for building with link time optimization
    Lto(bool),

    /// Remove files specified by PURGE_TARGETS
    Purge(bool),

    /// Leave static library (.a) files in packages
    StaticLibs(bool),

    /// Strip symbols from binaries/libraries
    Strip(bool),

    /// Compress manual (man and info) pages in MAN_DIRS with gzip
    Zipman(bool),
}

impl PackageOption {
    /// Creates a new [`PackageOption`] from a string slice.
    ///
    /// # Errors
    ///
    /// An error is returned if the string slice does not match a valid package option.
    pub fn new(option: &str) -> Result<Self, Error> {
        Self::from_str(option)
    }

    /// Returns the name of the [`PackageOption`] as string slice.
    pub fn name(&self) -> &str {
        match self {
            Self::Strip(_) => "strip",
            Self::Docs(_) => "docs",
            Self::Libtool(_) => "libtool",
            Self::StaticLibs(_) => "staticlibs",
            Self::EmptyDirs(_) => "emptydirs",
            Self::Zipman(_) => "zipman",
            Self::Purge(_) => "purge",
            Self::Debug(_) => "debug",
            Self::Lto(_) => "lto",
            Self::AutoDeps(_) => "autodeps",
        }
    }

    /// Returns whether the [`PackageOption`] is on or off.
    pub fn on(&self) -> bool {
        match self {
            Self::Strip(on)
            | Self::Docs(on)
            | Self::Libtool(on)
            | Self::StaticLibs(on)
            | Self::EmptyDirs(on)
            | Self::Zipman(on)
            | Self::Purge(on)
            | Self::Debug(on)
            | Self::Lto(on)
            | Self::AutoDeps(on) => *on,
        }
    }
}

impl FromStr for PackageOption {
    type Err = Error;
    /// Creates a [`PackageOption`] from string slice.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, on) = makepkg_option_parser.parse(s)?;
        match name.as_str() {
            "strip" => Ok(Self::Strip(on)),
            "docs" => Ok(Self::Docs(on)),
            "libtool" => Ok(Self::Libtool(on)),
            "staticlibs" => Ok(Self::StaticLibs(on)),
            "emptydirs" => Ok(Self::EmptyDirs(on)),
            "zipman" => Ok(Self::Zipman(on)),
            "purge" => Ok(Self::Purge(on)),
            "debug" => Ok(Self::Debug(on)),
            "lto" => Ok(Self::Lto(on)),
            "autodeps" => Ok(Self::AutoDeps(on)),
            _ => Err(Error::InvalidPackageOption(name)),
        }
    }
}

impl Display for PackageOption {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}{}", if self.on() { "" } else { "!" }, self.name())
    }
}

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
    #[case("autodeps", Ok(PackageOption::AutoDeps(true)))]
    #[case("debug", Ok(PackageOption::Debug(true)))]
    #[case("docs", Ok(PackageOption::Docs(true)))]
    #[case("emptydirs", Ok(PackageOption::EmptyDirs(true)))]
    #[case("!libtool", Ok(PackageOption::Libtool(false)))]
    #[case("lto", Ok(PackageOption::Lto(true)))]
    #[case("purge", Ok(PackageOption::Purge(true)))]
    #[case("staticlibs", Ok(PackageOption::StaticLibs(true)))]
    #[case("strip", Ok(PackageOption::Strip(true)))]
    #[case("zipman", Ok(PackageOption::Zipman(true)))]
    #[case("!invalid", Err(Error::InvalidPackageOption("invalid".to_string())))]
    fn package_option(#[case] s: &str, #[case] result: Result<PackageOption, Error>) {
        assert_eq!(PackageOption::from_str(s), result);
    }

    #[rstest]
    #[case("ccache", Ok(BuildEnvironmentOption::Ccache(true)))]
    #[case("check", Ok(BuildEnvironmentOption::Check(true)))]
    #[case("color", Ok(BuildEnvironmentOption::Color(true)))]
    #[case("distcc", Ok(BuildEnvironmentOption::Distcc(true)))]
    #[case("sign", Ok(BuildEnvironmentOption::Sign(true)))]
    #[case("!sign", Ok(BuildEnvironmentOption::Sign(false)))]
    #[case("!invalid", Err(Error::InvalidBuildEnvironmentOption("invalid".to_string())))]
    fn build_environment_option(
        #[case] s: &str,
        #[case] result: Result<BuildEnvironmentOption, Error>,
    ) {
        assert_eq!(BuildEnvironmentOption::from_str(s), result);
    }

    #[rstest]
    #[case("#test", "invalid character in makepkg option")]
    #[case("test!", "invalid character in makepkg option")]
    fn invalid_makepkg_option(#[case] input: &str, #[case] error_snippet: &str) {
        let result = makepkg_option_parser.parse(input);
        assert!(result.is_err(), "Expected makepkg option parsing to fail");
        let err = result.unwrap_err();
        let pretty_error = err.to_string();
        assert!(
            pretty_error.contains(error_snippet),
            "Error:\n=====\n{pretty_error}\n=====\nshould contain snippet:\n\n{error_snippet}"
        );
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
