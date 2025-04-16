use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    string::ToString,
};

use serde::{Deserialize, Serialize};
use winnow::{
    ModalResult,
    Parser,
    combinator::{alt, cut_err, eof, fail, opt, peek, repeat},
    error::{StrContext, StrContextValue::*},
    token::one_of,
};

use crate::{Architecture, Name, Version, error::Error};

/// Parser function for the `!` boolean operator in option names.
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
    let valid_chars = one_of((alphanum, '-', '.', '_', '!'));

    // Make sure that we have either a `!` at the start or the first char of a name.
    cut_err(peek(valid_chars))
        .context(StrContext::Expected(CharLiteral('!')))
        .context(StrContext::Expected(Description(
            "ASCII alphanumeric character",
        )))
        .context(StrContext::Expected(CharLiteral('-')))
        .context(StrContext::Expected(CharLiteral('.')))
        .context(StrContext::Expected(CharLiteral('_')))
        .parse_next(input)?;

    Ok(opt('!').parse_next(input)?.is_none())
}

/// Parser function for option names.
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
    let valid_chars = one_of((alphanum, '-', '.', '_'));
    let name = repeat::<_, _, (), _, _>(0.., valid_chars)
        .take()
        .parse_next(input)?;

    eof.context(StrContext::Label("character in makepkg option"))
        .context(StrContext::Expected(Description(
            "ASCII alphanumeric character",
        )))
        .context(StrContext::Expected(CharLiteral('-')))
        .context(StrContext::Expected(CharLiteral('.')))
        .context(StrContext::Expected(CharLiteral('_')))
        .parse_next(input)?;

    Ok(name)
}

/// This type wraps the [`PackageBuildOption`], [`PackageOption`] and [`BuildEnvironmentOption`]
/// enums. This is necessary for metadata files such as SRCINFO or PKGBUILD that don't
/// differentiate between the different types and scopes of options.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnyOption {
    /// A [`BuildEnvironmentOption`]
    BuildEnvironment(BuildEnvironmentOption),
    /// A [`PackageBuildOption`]
    PackageBuild(PackageBuildOption),
    /// A [`PackageOption`]
    Package(PackageOption),
}

impl AnyOption {
    /// Recognizes any [`PackageBuildOption`], [`PackageOption`] and [`BuildEnvironmentOption`] in a
    /// string slice.
    ///
    /// Consumes all of its input.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is neither of the listed options.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        alt((
            BuildEnvironmentOption::parser.map(AnyOption::BuildEnvironment),
            PackageBuildOption::parser.map(AnyOption::PackageBuild),
            PackageOption::parser.map(AnyOption::Package),
            fail.context(StrContext::Label("makepkg or package build option")),
        ))
        .parse_next(input)
    }
}

impl FromStr for AnyOption {
    type Err = Error;
    /// Creates a [`PackageBuildOption`] from string slice.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

impl Display for AnyOption {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match self {
            AnyOption::BuildEnvironment(option) => write!(fmt, "{option}"),
            AnyOption::PackageBuild(option) => write!(fmt, "{option}"),
            AnyOption::Package(option) => write!(fmt, "{option}"),
        }
    }
}

/// An option string used in packaging.
///
/// See [the PKGBUILD manpage](https://man.archlinux.org/man/PKGBUILD.5.en) for more info on these options.
///
/// ## Examples
/// ```
/// # fn main() -> Result<(), alpm_types::Error> {
/// use alpm_types::PackageBuildOption;
///
/// let option = PackageBuildOption::new("!makeflags")?;
/// assert_eq!(option.on(), false);
/// assert_eq!(option.name(), "makeflags");
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageBuildOption {
    /// Allow or forbid the use some buildflags (CPPFLAGS, CFLAGS, CXXFLAGS, LDFLAGS) from
    /// user-specific configs as specified in `makepkg.conf`.
    BuildFlags(bool),

    /// Completely allow or forbid the use of user-specific make configs as specified in
    /// `makepkg.conf`.
    MakeFlags(bool),
}

impl PackageBuildOption {
    /// Creates a new [`PackageBuildOption`] from a string slice.
    ///
    /// # Errors
    ///
    /// An error is returned if the string slice does not match a valid package option.
    pub fn new(option: &str) -> Result<Self, Error> {
        Self::from_str(option)
    }

    /// Returns the name of the [`PackageBuildOption`] as string slice.
    pub fn name(&self) -> &str {
        match self {
            Self::BuildFlags(_) => "buildflags",
            Self::MakeFlags(_) => "makeflags",
        }
    }

    /// Returns whether the [`PackageBuildOption`] is on or off.
    pub fn on(&self) -> bool {
        match self {
            Self::BuildFlags(on) | Self::MakeFlags(on) => *on,
        }
    }

    /// Recognizes a [`PackageBuildOption`] in a string slice.
    ///
    /// Consumes all of its input.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is not a valid package build option.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        let on = option_bool_parser.parse_next(input)?;
        let mut name = option_name_parser.parse_next(input)?;
        let variants = ("buildflags", "makeflags");
        let value = alt(variants)
            .context(StrContext::Label("package build option"))
            .context(StrContext::Expected(StringLiteral(variants.0)))
            .context(StrContext::Expected(StringLiteral(variants.1)))
            .parse_next(&mut name)?;

        match value {
            "buildflags" => Ok(Self::BuildFlags(on)),
            "makeflags" => Ok(Self::MakeFlags(on)),
            _ => unreachable!(),
        }
    }
}

impl FromStr for PackageBuildOption {
    type Err = Error;
    /// Creates a [`PackageBuildOption`] from string slice.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

impl Display for PackageBuildOption {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}{}", if self.on() { "" } else { "!" }, self.name())
    }
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
            Self::Ccache(_) => "ccache",
            Self::Check(_) => "check",
            Self::Color(_) => "color",
            Self::Distcc(_) => "distcc",
            Self::Sign(_) => "sign",
        }
    }

    /// Get whether the BuildEnvironmentOption is on
    pub fn on(&self) -> bool {
        match self {
            Self::Ccache(on)
            | Self::Check(on)
            | Self::Color(on)
            | Self::Distcc(on)
            | Self::Sign(on) => *on,
        }
    }

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
        let variants = ("ccache", "check", "color", "distcc", "sign");
        let name = alt(variants)
            .context(StrContext::Label("makepkg build environment option"))
            .context(StrContext::Expected(StringLiteral(variants.0)))
            .context(StrContext::Expected(StringLiteral(variants.1)))
            .context(StrContext::Expected(StringLiteral(variants.2)))
            .context(StrContext::Expected(StringLiteral(variants.3)))
            .context(StrContext::Expected(StringLiteral(variants.4)))
            .parse_next(&mut name)?;

        match name {
            "ccache" => Ok(Self::Ccache(on)),
            "check" => Ok(Self::Check(on)),
            "color" => Ok(Self::Color(on)),
            "distcc" => Ok(Self::Distcc(on)),
            "sign" => Ok(Self::Sign(on)),
            _ => unreachable!(),
        }
    }
}

impl FromStr for BuildEnvironmentOption {
    type Err = Error;
    /// Create an Option from a string
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

    /// Recognizes a [`PackageOption`] in a string slice.
    ///
    /// Consumes all of its input.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is not a valid package option.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        let on = option_bool_parser.parse_next(input)?;
        let mut name = option_name_parser.parse_next(input)?;
        let variants = (
            "autodeps",
            "debug",
            "docs",
            "emptydirs",
            "libtool",
            "lto",
            "purge",
            "staticlibs",
            "strip",
            "zipman",
        );
        let value = alt(variants)
            .context(StrContext::Label("makepkg option"))
            .context(StrContext::Expected(StringLiteral(variants.0)))
            .context(StrContext::Expected(StringLiteral(variants.1)))
            .context(StrContext::Expected(StringLiteral(variants.2)))
            .context(StrContext::Expected(StringLiteral(variants.3)))
            .context(StrContext::Expected(StringLiteral(variants.4)))
            .context(StrContext::Expected(StringLiteral(variants.5)))
            .context(StrContext::Expected(StringLiteral(variants.6)))
            .context(StrContext::Expected(StringLiteral(variants.7)))
            .context(StrContext::Expected(StringLiteral(variants.8)))
            .context(StrContext::Expected(StringLiteral(variants.9)))
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
            _ => unreachable!(),
        }
    }
}

impl FromStr for PackageOption {
    type Err = Error;
    /// Creates a [`PackageOption`] from string slice.
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
    #[case(
        "!makeflags",
        AnyOption::PackageBuild(PackageBuildOption::MakeFlags(false))
    )]
    #[case("autodeps", AnyOption::Package(PackageOption::AutoDeps(true)))]
    #[case(
        "ccache",
        AnyOption::BuildEnvironment(BuildEnvironmentOption::Ccache(true))
    )]
    fn any_option(#[case] input: &str, #[case] expected: AnyOption) {
        let result = AnyOption::from_str(input).expect("Parser should be successful");
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("!somethingelse", "invalid makepkg or package build option")]
    #[case(
        "#somethingelse",
        "expected `!`, ASCII alphanumeric character, `-`, `.`, `_`"
    )]
    fn invalid_any_option(#[case] input: &str, #[case] err_snippet: &str) {
        let Err(Error::ParseError(err_msg)) = AnyOption::from_str(input) else {
            panic!("'{input}' erroneously parsed as VersionRequirement")
        };
        assert!(
            err_msg.contains(err_snippet),
            "Error:\n=====\n{err_msg}\n=====\nshould contain snippet:\n\n{err_snippet}"
        );
    }

    #[rstest]
    #[case("buildflags", PackageBuildOption::BuildFlags(true))]
    #[case("!makeflags", PackageBuildOption::MakeFlags(false))]
    fn package_build_option(#[case] input: &str, #[case] expected: PackageBuildOption) {
        let result = PackageBuildOption::from_str(input).expect("Parser should be successful");
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("!somethingelse", "expected `buildflags`, `makeflags`")]
    #[case(
        "#somethingelse",
        "expected `!`, ASCII alphanumeric character, `-`, `.`, `_`"
    )]
    fn invalid_package_build_option(#[case] input: &str, #[case] err_snippet: &str) {
        let Err(Error::ParseError(err_msg)) = PackageBuildOption::from_str(input) else {
            panic!("'{input}' erroneously parsed as VersionRequirement")
        };
        assert!(
            err_msg.contains(err_snippet),
            "Error:\n=====\n{err_msg}\n=====\nshould contain snippet:\n\n{err_snippet}"
        );
    }

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
        "expected `autodeps`, `debug`, `docs`, `emptydirs`, `libtool`, `lto`, `purge`, `staticlibs`, `strip`, `zipman`"
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
    #[case("ccache", BuildEnvironmentOption::Ccache(true))]
    #[case("check", BuildEnvironmentOption::Check(true))]
    #[case("color", BuildEnvironmentOption::Color(true))]
    #[case("distcc", BuildEnvironmentOption::Distcc(true))]
    #[case("sign", BuildEnvironmentOption::Sign(true))]
    #[case("!sign", BuildEnvironmentOption::Sign(false))]
    fn build_environment_option(#[case] input: &str, #[case] expected: BuildEnvironmentOption) {
        let result = BuildEnvironmentOption::from_str(input).expect("Parser should be successful");
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(
        "!somethingelse",
        "expected `ccache`, `check`, `color`, `distcc`, `sign`"
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
