use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use alpm_parsers::{iter_char_context, iter_str_context};
use serde::{Deserialize, Serialize};
use strum::VariantNames;
use winnow::{
    ModalResult,
    Parser,
    combinator::{alt, cut_err, eof, fail, opt, peek, repeat},
    error::{
        AddContext,
        ContextError,
        ErrMode,
        ParserError,
        StrContext,
        StrContextValue::{self, *},
    },
    stream::Stream,
    token::{one_of, rest, take_until},
};

use crate::{
    Architecture,
    FullVersion,
    Name,
    PackageFileName,
    PackageRelation,
    VersionComparison,
    VersionRequirement,
    error::Error,
};

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

/// Wraps the [`PackageOption`] and [`BuildEnvironmentOption`] enums.
///
/// This is necessary for metadata files such as [SRCINFO] or [PKGBUILD] package scripts that don't
/// differentiate between the different types and scopes of options.
///
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
/// [PKGBUILD]: https://man.archlinux.org/man/PKGBUILD.5
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MakepkgOption {
    /// A [`BuildEnvironmentOption`]
    BuildEnvironment(BuildEnvironmentOption),
    /// A [`PackageOption`]
    Package(PackageOption),
}

impl MakepkgOption {
    /// Recognizes any [`PackageOption`] and [`BuildEnvironmentOption`] in a
    /// string slice.
    ///
    /// Consumes all of its input.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is neither of the listed options.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        alt((
            BuildEnvironmentOption::parser.map(MakepkgOption::BuildEnvironment),
            PackageOption::parser.map(MakepkgOption::Package),
            fail.context(StrContext::Label("packaging or build environment option"))
                .context_with(iter_str_context!([
                    BuildEnvironmentOption::VARIANTS.to_vec(),
                    PackageOption::VARIANTS.to_vec()
                ])),
        ))
        .parse_next(input)
    }
}

impl FromStr for MakepkgOption {
    type Err = Error;
    /// Creates a [`MakepkgOption`] from string slice.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

impl Display for MakepkgOption {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match self {
            MakepkgOption::BuildEnvironment(option) => write!(fmt, "{option}"),
            MakepkgOption::Package(option) => write!(fmt, "{option}"),
        }
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
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, VariantNames)]
#[serde(rename_all = "lowercase")]
pub enum BuildEnvironmentOption {
    /// Use or unset the values of build flags (e.g. `CPPFLAGS`, `CFLAGS`, `CXXFLAGS`, `LDFLAGS`)
    /// specified in user-specific configs (e.g. [makepkg.conf]).
    ///
    /// [makepkg.conf]: https://man.archlinux.org/man/makepkg.conf.5
    #[strum(serialize = "buildflags")]
    BuildFlags(bool),
    /// Use ccache to cache compilation
    #[strum(serialize = "ccache")]
    Ccache(bool),
    /// Run the check() function if present in the PKGBUILD
    #[strum(serialize = "check")]
    Check(bool),
    /// Colorize output messages
    #[strum(serialize = "color")]
    Color(bool),
    /// Use the Distributed C/C++/ObjC compiler
    #[strum(serialize = "distcc")]
    Distcc(bool),
    /// Generate PGP signature file
    #[strum(serialize = "sign")]
    Sign(bool),
    /// Use or unset the value of the `MAKEFLAGS` environment variable specified in
    /// user-specific configs (e.g. [makepkg.conf]).
    ///
    /// [makepkg.conf]: https://man.archlinux.org/man/makepkg.conf.5
    #[strum(serialize = "makeflags")]
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

        alt((
            "buildflags".value(Self::BuildFlags(on)),
            "ccache".value(Self::Ccache(on)),
            "check".value(Self::Check(on)),
            "color".value(Self::Color(on)),
            "distcc".value(Self::Distcc(on)),
            "makeflags".value(Self::MakeFlags(on)),
            "sign".value(Self::Sign(on)),
            fail.context(StrContext::Label("makepkg build environment option"))
                .context_with(iter_str_context!([BuildEnvironmentOption::VARIANTS])),
        ))
        .parse_next(&mut name)
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
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, VariantNames)]
#[serde(rename_all = "lowercase")]
pub enum PackageOption {
    /// Automatically add dependencies and provisions (see [alpm-sonamev2]).
    ///
    /// [alpm-sonamev2]: https://alpm.archlinux.page/specifications/alpm-sonamev2.7.html
    #[strum(serialize = "autodeps")]
    AutoDeps(bool),

    /// Add debugging flags as specified in DEBUG_* variables
    #[strum(serialize = "debug")]
    Debug(bool),

    /// Save doc directories specified by DOC_DIRS
    #[strum(serialize = "docs")]
    Docs(bool),

    /// Leave empty directories in packages
    #[strum(serialize = "emptydirs")]
    EmptyDirs(bool),

    /// Leave libtool (.la) files in packages
    #[strum(serialize = "libtool")]
    Libtool(bool),

    /// Add compile flags for building with link time optimization
    #[strum(serialize = "lto")]
    Lto(bool),

    /// Remove files specified by PURGE_TARGETS
    #[strum(serialize = "purge")]
    Purge(bool),

    /// Leave static library (.a) files in packages
    #[strum(serialize = "staticlibs")]
    StaticLibs(bool),

    /// Strip symbols from binaries/libraries
    #[strum(serialize = "strip")]
    Strip(bool),

    /// Compress manual (man and info) pages in MAN_DIRS with gzip
    #[strum(serialize = "zipman")]
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
    /// Returns an error if `input` is not the valid string representation of a [`PackageOption`].
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        let on = option_bool_parser.parse_next(input)?;
        let mut name = option_name_parser.parse_next(input)?;

        alt((
            "autodeps".value(Self::AutoDeps(on)),
            "debug".value(Self::Debug(on)),
            "docs".value(Self::Docs(on)),
            "emptydirs".value(Self::EmptyDirs(on)),
            "libtool".value(Self::Libtool(on)),
            "lto".value(Self::Lto(on)),
            "purge".value(Self::Purge(on)),
            "staticlibs".value(Self::StaticLibs(on)),
            "strip".value(Self::Strip(on)),
            "zipman".value(Self::Zipman(on)),
            fail.context(StrContext::Label("makepkg packaging option"))
                .context_with(iter_str_context!([PackageOption::VARIANTS])),
        ))
        .parse_next(&mut name)
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
/// Tracks the [`Name`], [`FullVersion`] and an [`Architecture`] of a package in an environment.
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Architecture, FullVersion, InstalledPackage, Name};
/// # fn main() -> Result<(), alpm_types::Error> {
/// assert_eq!(
///     InstalledPackage::from_str("foo-bar-1:1.0.0-1-any")?,
///     InstalledPackage::new(
///         Name::new("foo-bar")?,
///         FullVersion::from_str("1:1.0.0-1")?,
///         Architecture::Any
///     )
/// );
/// assert_eq!(
///     InstalledPackage::from_str("foo-bar-1.0.0-1-any")?,
///     InstalledPackage::new(
///         Name::new("foo-bar")?,
///         FullVersion::from_str("1.0.0-1")?,
///         Architecture::Any
///     )
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct InstalledPackage {
    name: Name,
    version: FullVersion,
    architecture: Architecture,
}

impl InstalledPackage {
    /// Creates a new [`InstalledPackage`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::InstalledPackage;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// assert_eq!(
    ///     "example-1:1.0.0-1-x86_64",
    ///     InstalledPackage::new("example".parse()?, "1:1.0.0-1".parse()?, "x86_64".parse()?)
    ///         .to_string()
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(name: Name, version: FullVersion, architecture: Architecture) -> Self {
        Self {
            name,
            version,
            architecture,
        }
    }

    /// Returns a reference to the [`Name`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::{InstalledPackage, Name};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let file_name =
    ///     InstalledPackage::new("example".parse()?, "1:1.0.0-1".parse()?, "x86_64".parse()?);
    ///
    /// assert_eq!(file_name.name(), &Name::new("example")?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn name(&self) -> &Name {
        &self.name
    }

    /// Returns a reference to the [`FullVersion`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::{FullVersion, InstalledPackage};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let file_name =
    ///     InstalledPackage::new("example".parse()?, "1:1.0.0-1".parse()?, "x86_64".parse()?);
    ///
    /// assert_eq!(file_name.version(), &FullVersion::from_str("1:1.0.0-1")?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn version(&self) -> &FullVersion {
        &self.version
    }

    /// Returns the [`Architecture`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::{InstalledPackage, SystemArchitecture};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let file_name =
    ///     InstalledPackage::new("example".parse()?, "1:1.0.0-1".parse()?, "x86_64".parse()?);
    ///
    /// assert_eq!(file_name.architecture(), &SystemArchitecture::X86_64.into());
    /// # Ok(())
    /// # }
    /// ```
    pub fn architecture(&self) -> &Architecture {
        &self.architecture
    }

    /// Returns the [`PackageRelation`] encoded in this [`InstalledPackage`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::{InstalledPackage, PackageRelation};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let installed_package =
    ///     InstalledPackage::new("example".parse()?, "1:1.0.0-1".parse()?, "x86_64".parse()?);
    ///
    /// assert_eq!(
    ///     installed_package.to_package_relation(),
    ///     PackageRelation::from_str("example=1:1.0.0-1")?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_package_relation(&self) -> PackageRelation {
        PackageRelation {
            name: self.name.clone(),
            version_requirement: Some(VersionRequirement {
                comparison: VersionComparison::Equal,
                version: self.version.clone().into(),
            }),
        }
    }

    /// Recognizes an [`InstalledPackage`] in a string slice.
    ///
    /// Relies on [`winnow`] to parse `input` and recognize the [`Name`], [`FullVersion`], and
    /// [`Architecture`] components.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - the [`Name`] component can not be recognized,
    /// - the [`FullVersion`] component can not be recognized,
    /// - or the [`Architecture`] component can not be recognized.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_types::InstalledPackage;
    /// use winnow::Parser;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let name = "example-package-1:1.0.0-1-x86_64";
    /// assert_eq!(name, InstalledPackage::parser.parse(name)?.to_string());
    /// # Ok(())
    /// # }
    /// ```
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        // Detect the amount of dashes in input and subsequently in the Name component.
        //
        // This is a necessary step because dashes are used as delimiters between the
        // components of the file name and the Name component (an alpm-package-name) can contain
        // dashes, too.
        // We know that the minimum amount of dashes in a valid alpm-package file name is
        // three (one dash between the Name, Version, PackageRelease, and Architecture
        // component each).
        // We rely on this fact to determine the amount of dashes in the Name component and
        // thereby the cut-off point between the Name and the Version component.
        let dashes: usize = input.chars().filter(|char| char == &'-').count();

        if dashes < 2 {
            let context_error = ContextError::from_input(input)
                .add_context(
                    input,
                    &input.checkpoint(),
                    StrContext::Label("alpm-package file name"),
                )
                .add_context(
                    input,
                    &input.checkpoint(),
                    StrContext::Expected(StrContextValue::Description(
                        concat!(
                        "a package name, followed by an alpm-package-version (full or full with epoch) and an architecture.",
                        "\nAll components must be delimited with a dash ('-')."
                        )
                    ))
                );

            return Err(ErrMode::Cut(context_error));
        }

        // The (zero or more) dashes in the Name component.
        let dashes_in_name = dashes.saturating_sub(3);

        // Advance the parser to the dash just behind the Name component, based on the amount of
        // dashes in the Name, e.g.:
        // "example-package-1:1.0.0-1-x86_64.pkg.tar.zst" -> "-1:1.0.0-1-x86_64.pkg.tar.zst"
        let name = cut_err(
            repeat::<_, _, (), _, _>(
                dashes_in_name + 1,
                // Advances to the next `-`.
                // If multiple `-` are present, the `-` that has been previously advanced to will
                // be consumed in the next itaration via the `opt("-")`. This enables us to go
                // **up to** the last `-`, while still consuming all `-` in between.
                (opt("-"), take_until(0.., "-"), peek("-")),
            )
            .take()
            // example-package
            .and_then(Name::parser),
        )
        .context(StrContext::Label("alpm-package-name"))
        .parse_next(input)?;

        // Consume leading dash in front of Version, e.g.:
        // "-1:1.0.0-1-x86_64.pkg.tar.zst" -> "1:1.0.0-1-x86_64.pkg.tar.zst"
        "-".parse_next(input)?;

        // Advance the parser to beyond the Version component (which contains one dash), e.g.:
        // "1:1.0.0-1-x86_64.pkg.tar.zst" -> "-x86_64.pkg.tar.zst"
        let version: FullVersion = cut_err((take_until(0.., "-"), "-", take_until(0.., "-")))
            .context(StrContext::Label("alpm-package-version"))
            .context(StrContext::Expected(StrContextValue::Description(
                "an alpm-package-version (full or full with epoch) followed by a `-` and an architecture",
            )))
            .take()
            .and_then(cut_err(FullVersion::parser))
            .parse_next(input)?;

        // Consume leading dash, e.g.:
        // "-x86_64.pkg.tar.zst" -> "x86_64.pkg.tar.zst"
        "-".parse_next(input)?;

        // Advance the parser to beyond the Architecture component, e.g.:
        // "x86_64.pkg.tar.zst" -> ".pkg.tar.zst"
        let architecture = rest.and_then(Architecture::parser).parse_next(input)?;

        Ok(Self {
            name,
            version,
            architecture,
        })
    }
}

impl From<PackageFileName> for InstalledPackage {
    /// Creates a [`InstalledPackage`] from a [`PackageFileName`].
    fn from(value: PackageFileName) -> Self {
        Self {
            name: value.name,
            version: value.version,
            architecture: value.architecture,
        }
    }
}

impl FromStr for InstalledPackage {
    type Err = Error;

    /// Creates an [`InstalledPackage`] from a string slice.
    ///
    /// Delegates to [`InstalledPackage::parser`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`InstalledPackage::parser`] fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::InstalledPackage;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let filename = "example-package-1:1.0.0-1-x86_64";
    /// assert_eq!(filename, InstalledPackage::from_str(filename)?.to_string());
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<InstalledPackage, Self::Err> {
        Ok(Self::parser.parse(s)?)
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
    use testresult::TestResult;

    use super::*;
    use crate::SystemArchitecture;

    #[rstest]
    #[case(
        "!makeflags",
        MakepkgOption::BuildEnvironment(BuildEnvironmentOption::MakeFlags(false))
    )]
    #[case("autodeps", MakepkgOption::Package(PackageOption::AutoDeps(true)))]
    #[case(
        "ccache",
        MakepkgOption::BuildEnvironment(BuildEnvironmentOption::Ccache(true))
    )]
    fn makepkg_option(#[case] input: &str, #[case] expected: MakepkgOption) {
        let result = MakepkgOption::from_str(input).expect("Parser should be successful");
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(
        "!somethingelse",
        concat!(
            "expected `buildflags`, `ccache`, `check`, `color`, `distcc`, `sign`, `makeflags`, ",
            "`autodeps`, `debug`, `docs`, `emptydirs`, `libtool`, `lto`, `purge`, ",
            "`staticlibs`, `strip`, `zipman`",
        )
    )]
    #[case(
        "#somethingelse",
        "expected `!`, ASCII alphanumeric character, `-`, `.`, `_`"
    )]
    fn invalid_makepkg_option(#[case] input: &str, #[case] err_snippet: &str) {
        let Err(Error::ParseError(err_msg)) = MakepkgOption::from_str(input) else {
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
        "expected `buildflags`, `ccache`, `check`, `color`, `distcc`, `sign`, `makeflags`"
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
    fn invalid_option(#[case] input: &str, #[case] error_snippet: &str) {
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
        InstalledPackage {
            name: Name::new("foo-bar")?,
            version: FullVersion::from_str("1:1.0.0-1")?,
            architecture: Architecture::Any,
        },
    )]
    #[case(
        "foobar-1.0.0-1-x86_64",
        InstalledPackage {
            name: Name::new("foobar")?,
            version: FullVersion::from_str("1.0.0-1")?,
            architecture: SystemArchitecture::X86_64.into(),
        },
    )]
    fn installed_from_str(#[case] s: &str, #[case] result: InstalledPackage) -> TestResult {
        assert_eq!(InstalledPackage::from_str(s), Ok(result));
        Ok(())
    }

    #[rstest]
    #[case("foo-1:1.0.0-bar-any", "invalid package release")]
    #[case(
        "foo-1:1.0.0_any",
        "expected a package name, followed by an alpm-package-version (full or full with epoch) and an architecture."
    )]
    #[case("packagename-30-0.1oops-any", "expected end of package release value")]
    #[case("package$with$dollars-30-0.1-any", "invalid character in package name")]
    #[case(
        "packagename-30-0.1-any*asdf",
        "invalid character in system architecture"
    )]
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
