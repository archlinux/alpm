use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    string::ToString,
};

use alpm_parsers::{iter_char_context, iter_str_context};
use serde::{Deserialize, Serialize};
use winnow::{
    ModalResult,
    Parser,
    combinator::{alt, cut_err, eof, opt, peek, repeat},
    error::{StrContext, StrContextValue::*},
    token::one_of,
};

use crate::{Architecture, Name, Version, error::Error};

/// Recognizes the `!` boolean operator in option names.
///
/// This parser **does not** fully consume its input.
/// It also expects the package name to be there, if the `!` does not exist.
///
/// # Format
///
/// The parser expects a `!` or either one of ASCII alphanumeric character, hyphen, dot, or
/// underscore.
///
/// # Errors
///
/// If the input string does not match the expected format, an error will be returned.
fn option_bool_parser(input: &mut &str) -> ModalResult<bool> {
    let alphanum = |c: char| c.is_ascii_alphanumeric();
    let special_first_chars = ['-', '.', '_', '!'];
    let valid_chars = one_of((alphanum, special_first_chars));

    // Make sure that we have either a `!` at the start or the first char of a name.
    cut_err(peek(valid_chars))
        .context(StrContext::Expected(CharLiteral('!')))
        .context(StrContext::Expected(Description(
            "ASCII alphanumeric character",
        )))
        .context_with(iter_char_context!(special_first_chars))
        .parse_next(input)?;

    Ok(opt('!').parse_next(input)?.is_none())
}

/// Recognizes option names.
///
/// This parser fully consumes its input.
///
/// # Format
///
/// The parser expects a sequence of ASCII alphanumeric characters, hyphens, dots, or underscores.
///
/// # Errors
///
/// If the input string does not match the expected format, an error will be returned.
fn option_name_parser<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    let alphanum = |c: char| c.is_ascii_alphanumeric();

    let special_chars = ['-', '.', '_'];
    let valid_chars = one_of((alphanum, special_chars));
    let name = repeat::<_, _, (), _, _>(0.., valid_chars)
        .take()
        .parse_next(input)?;

    eof.context(StrContext::Label("character in makepkg option"))
        .context(StrContext::Expected(Description(
            "ASCII alphanumeric character",
        )))
        .context_with(iter_char_context!(special_chars))
        .parse_next(input)?;

    Ok(name)
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
    /// Use or unset the values of build flags (e.g. `CPPFLAGS`, `CFLAGS`, `CXXFLAGS`, `LDFLAGS`)
    /// specified in user-specific configs (e.g. [makepkg.conf]).
    ///
    /// [makepkg.conf]: https://man.archlinux.org/man/makepkg.conf.5
    BuildFlags(bool),
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
    /// Use or unset the value of the `MAKEFLAGS` environment variable specified in
    /// user-specific configs (e.g. [makepkg.conf]).
    ///
    /// [makepkg.conf]: https://man.archlinux.org/man/makepkg.conf.5
    MakeFlags(bool),
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

    /// Get the name of the BuildEnvironmentOption
    pub fn name(&self) -> &str {
        match self {
            Self::BuildFlags(_) => "buildflags",
            Self::Ccache(_) => "ccache",
            Self::Check(_) => "check",
            Self::Color(_) => "color",
            Self::Distcc(_) => "distcc",
            Self::MakeFlags(_) => "makeflags",
            Self::Sign(_) => "sign",
        }
    }

    /// Get whether the BuildEnvironmentOption is on
    pub fn on(&self) -> bool {
        match self {
            Self::BuildFlags(on)
            | Self::Ccache(on)
            | Self::Check(on)
            | Self::Color(on)
            | Self::Distcc(on)
            | Self::MakeFlags(on)
            | Self::Sign(on) => *on,
        }
    }

    const VARIANTS: [&str; 7] = [
        "buildflags",
        "ccache",
        "check",
        "color",
        "distcc",
        "makeflags",
        "sign",
    ];

    /// Recognizes a [`BuildEnvironmentOption`] in a string slice.
    ///
    /// Consumes all of its input.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is not a valid build environment option.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        let on = option_bool_parser.parse_next(input)?;
        let mut name = option_name_parser.parse_next(input)?;

        let name = alt(BuildEnvironmentOption::VARIANTS)
            .context(StrContext::Label("makepkg build environment option"))
            .context_with(iter_str_context!([BuildEnvironmentOption::VARIANTS]))
            .parse_next(&mut name)?;

        match name {
            "buildflags" => Ok(Self::BuildFlags(on)),
            "ccache" => Ok(Self::Ccache(on)),
            "check" => Ok(Self::Check(on)),
            "color" => Ok(Self::Color(on)),
            "distcc" => Ok(Self::Distcc(on)),
            "makeflags" => Ok(Self::MakeFlags(on)),
            "sign" => Ok(Self::Sign(on)),
            // Unreachable because the winnow parser returns an error above.
            _ => unreachable!(),
        }
    }
}

impl FromStr for BuildEnvironmentOption {
    type Err = Error;
    /// Creates a [`BuildEnvironmentOption`] from a string slice.
    ///
    /// Delegates to [`BuildEnvironmentOption::parser`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`BuildEnvironmentOption::parser`] fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
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
            Self::AutoDeps(_) => "autodeps",
            Self::Debug(_) => "debug",
            Self::Docs(_) => "docs",
            Self::EmptyDirs(_) => "emptydirs",
            Self::Libtool(_) => "libtool",
            Self::Lto(_) => "lto",
            Self::Purge(_) => "purge",
            Self::StaticLibs(_) => "staticlibs",
            Self::Strip(_) => "strip",
            Self::Zipman(_) => "zipman",
        }
    }

    /// Returns whether the [`PackageOption`] is on or off.
    pub fn on(&self) -> bool {
        match self {
            Self::AutoDeps(on)
            | Self::Debug(on)
            | Self::Docs(on)
            | Self::EmptyDirs(on)
            | Self::Libtool(on)
            | Self::Lto(on)
            | Self::Purge(on)
            | Self::StaticLibs(on)
            | Self::Strip(on)
            | Self::Zipman(on) => *on,
        }
    }

    const VARIANTS: [&str; 11] = [
        "autodeps",
        "debug",
        "docs",
        "emptydirs",
        "libtool",
        "lto",
        "debug",
        "purge",
        "staticlibs",
        "strip",
        "zipman",
    ];

    /// Recognizes a [`PackageOption`] in a string slice.
    ///
    /// Consumes all of its input.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is not the valid string representation of a [`PackageOption`].
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        let on = option_bool_parser.parse_next(input)?;
        let mut name = option_name_parser.parse_next(input)?;

        let value = alt(PackageOption::VARIANTS)
            .context(StrContext::Label("makepkg option"))
            .context_with(iter_str_context!([PackageOption::VARIANTS]))
            .parse_next(&mut name)?;

        match value {
            "autodeps" => Ok(Self::AutoDeps(on)),
            "debug" => Ok(Self::Debug(on)),
            "docs" => Ok(Self::Docs(on)),
            "emptydirs" => Ok(Self::EmptyDirs(on)),
            "libtool" => Ok(Self::Libtool(on)),
            "lto" => Ok(Self::Lto(on)),
            "purge" => Ok(Self::Purge(on)),
            "staticlibs" => Ok(Self::StaticLibs(on)),
            "strip" => Ok(Self::Strip(on)),
            "zipman" => Ok(Self::Zipman(on)),
            // Unreachable because the winnow parser returns an error above.
            _ => unreachable!(),
        }
    }
}

impl FromStr for PackageOption {
    type Err = Error;
    /// Creates a [`PackageOption`] from a string slice.
    ///
    /// Delegates to [`PackageOption::parser`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`PackageOption::parser`] fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
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
    #[case("autodeps", PackageOption::AutoDeps(true))]
    #[case("debug", PackageOption::Debug(true))]
    #[case("docs", PackageOption::Docs(true))]
    #[case("emptydirs", PackageOption::EmptyDirs(true))]
    #[case("!libtool", PackageOption::Libtool(false))]
    #[case("lto", PackageOption::Lto(true))]
    #[case("purge", PackageOption::Purge(true))]
    #[case("staticlibs", PackageOption::StaticLibs(true))]
    #[case("strip", PackageOption::Strip(true))]
    #[case("zipman", PackageOption::Zipman(true))]
    fn package_option(#[case] s: &str, #[case] expected: PackageOption) {
        let result = PackageOption::from_str(s).expect("Parser should be successful");
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(
        "!somethingelse",
        "expected `autodeps`, `debug`, `docs`, `emptydirs`, `libtool`, `lto`, `debug`, `purge`, `staticlibs`, `strip`, `zipman`"
    )]
    #[case(
        "#somethingelse",
        "expected `!`, ASCII alphanumeric character, `-`, `.`, `_`"
    )]
    fn invalid_package_option(#[case] input: &str, #[case] err_snippet: &str) {
        let Err(Error::ParseError(err_msg)) = PackageOption::from_str(input) else {
            panic!("'{input}' erroneously parsed as VersionRequirement")
        };
        assert!(
            err_msg.contains(err_snippet),
            "Error:\n=====\n{err_msg}\n=====\nshould contain snippet:\n\n{err_snippet}"
        );
    }

    #[rstest]
    #[case("buildflags", BuildEnvironmentOption::BuildFlags(true))]
    #[case("ccache", BuildEnvironmentOption::Ccache(true))]
    #[case("check", BuildEnvironmentOption::Check(true))]
    #[case("color", BuildEnvironmentOption::Color(true))]
    #[case("distcc", BuildEnvironmentOption::Distcc(true))]
    #[case("!makeflags", BuildEnvironmentOption::MakeFlags(false))]
    #[case("sign", BuildEnvironmentOption::Sign(true))]
    #[case("!sign", BuildEnvironmentOption::Sign(false))]
    fn build_environment_option(#[case] input: &str, #[case] expected: BuildEnvironmentOption) {
        let result = BuildEnvironmentOption::from_str(input).expect("Parser should be successful");
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(
        "!somethingelse",
        "expected `buildflags`, `ccache`, `check`, `color`, `distcc`, `makeflags`, `sign`"
    )]
    #[case(
        "#somethingelse",
        "expected `!`, ASCII alphanumeric character, `-`, `.`, `_`"
    )]
    fn invalid_build_environment_option(#[case] input: &str, #[case] err_snippet: &str) {
        let Err(Error::ParseError(err_msg)) = BuildEnvironmentOption::from_str(input) else {
            panic!("'{input}' erroneously parsed as VersionRequirement")
        };
        assert!(
            err_msg.contains(err_snippet),
            "Error:\n=====\n{err_msg}\n=====\nshould contain snippet:\n\n{err_snippet}"
        );
    }

    #[rstest]
    #[case("#test", "invalid character in makepkg option")]
    #[case("test!", "invalid character in makepkg option")]
    fn invalid_makepkg_option(#[case] input: &str, #[case] error_snippet: &str) {
        let result = option_name_parser.parse(input);
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
