// SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
// SPDX-License-Identifier: LGPL-3.0-or-later
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;
use std::string::ToString;

use email_address::EmailAddress;

use strum_macros::Display;
use strum_macros::EnumString;

mod checksum;
pub use checksum::Md5Sum;

mod date;
pub use date::BuildDate;

mod error;
pub use error::Error;

mod macros;
use macros::regex_once;

mod name;
pub use name::BuildTool;
pub use name::Name;

mod path;
pub use path::AbsolutePath;
pub use path::BuildDir;

mod size;
pub use size::CompressedSize;
pub use size::InstalledSize;

mod system;
pub use system::Architecture;

mod version;
pub use version::Epoch;
pub use version::Pkgrel;
pub use version::Pkgver;
pub use version::SchemaVersion;
pub use version::Version;

/// An option string used in a build environment
///
/// The option string is identified by its name and whether it is on (not prefixed with "!") or off (prefixed with "!").
/// This type dereferences to `BuildOption`.
///
/// ## Examples
/// ```
/// use alpm_types::BuildEnv;
///
/// let option = BuildEnv::new("foo").unwrap();
/// assert_eq!(option.on(), true);
/// assert_eq!(option.name(), "foo");
///
/// let not_option = BuildEnv::new("!foo").unwrap();
/// assert_eq!(not_option.on(), false);
/// assert_eq!(not_option.name(), "foo");
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildEnv(BuildOption);

impl BuildEnv {
    /// Create a new BuildEnv from a string
    pub fn new(option: &str) -> Result<Self, Error> {
        match BuildOption::new(option) {
            Ok(build_option) => Ok(BuildEnv(build_option)),
            Err(_) => Err(Error::InvalidBuildEnv(option.to_string())),
        }
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &BuildOption {
        &self.0
    }

    /// Get the name of the BuildEnv
    pub fn name(&self) -> &str {
        self.inner().name()
    }

    /// Get whether the BuildEnv is on
    pub fn on(&self) -> bool {
        self.inner().on()
    }
}

impl FromStr for BuildEnv {
    type Err = Error;
    /// Create a BuildEnv from a string
    fn from_str(input: &str) -> Result<BuildEnv, Self::Err> {
        BuildEnv::new(input)
    }
}

impl Display for BuildEnv {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

/// An option string
///
/// The option string is identified by its name and whether it is on (not prefixed with "!") or off (prefixed with "!").
///
/// ## Examples
/// ```
/// use alpm_types::BuildOption;
///
/// let option = BuildOption::new("foo").unwrap();
/// assert_eq!(option.on(), true);
/// assert_eq!(option.name(), "foo");
///
/// let not_option = BuildOption::new("!foo").unwrap();
/// assert_eq!(not_option.on(), false);
/// assert_eq!(not_option.name(), "foo");
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildOption {
    name: String,
    on: bool,
}

impl BuildOption {
    /// Create a new BuildOption in a Result
    pub fn new(option: &str) -> Result<Self, Error> {
        let option_regex = regex_once!(r"^(?P<on>!?)(?P<name>[\w\-.]+)$");
        if let Some(captures) = option_regex.captures(option) {
            if captures.name("on").is_some() && captures.name("name").is_some() {
                Ok(BuildOption {
                    name: captures.name("name").unwrap().as_str().into(),
                    on: !captures.name("on").unwrap().as_str().contains('!'),
                })
            } else {
                Err(Error::InvalidBuildOption(option.into()))
            }
        } else {
            Err(Error::InvalidBuildOption(option.into()))
        }
    }

    /// Get the name of the BuildOption
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get whether the BuildOption is on
    pub fn on(&self) -> bool {
        self.on
    }
}

impl FromStr for BuildOption {
    type Err = Error;
    /// Create an Option from a string
    fn from_str(input: &str) -> Result<BuildOption, Self::Err> {
        BuildOption::new(input)
    }
}

impl Display for BuildOption {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}{}", if self.on { "" } else { "!" }, self.name)
    }
}

/// A packager of a package
///
/// A `Packager` is represented by a User ID (e.g. `"Foobar McFooFace <foobar@mcfooface.org>"`).
/// Internally this struct wraps a `String` for the name and an `EmailAddress` for a valid email address.
///
/// ## Examples
/// ```
/// use alpm_types::{Packager, Error};
/// use std::str::FromStr;
///
/// // create Packager from &str
/// let packager = Packager::new("Foobar McFooface <foobar@mcfooface.org>").unwrap();
///
/// // get name
/// assert_eq!("Foobar McFooface", packager.name());
///
/// // get email
/// assert_eq!("foobar@mcfooface.org", packager.email().to_string());
///
/// // get email domain
/// assert_eq!("mcfooface.org", packager.email().domain());
///
/// // format as String
/// assert_eq!("Foobar McFooface <foobar@mcfooface.org>", format!("{}", packager));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Packager {
    name: String,
    email: EmailAddress,
}

impl Packager {
    /// Create a new Packager from a string
    pub fn new(packager: &str) -> Result<Packager, Error> {
        let packager_regex = regex_once!(r"^(?P<name>[\w\s\-().]+) <(?P<email>.*)>$");
        if let Some(captures) = packager_regex.captures(packager) {
            if captures.name("name").is_some() && captures.name("email").is_some() {
                if let Ok(email) = EmailAddress::from_str(captures.name("email").unwrap().as_str())
                {
                    Ok(Packager {
                        name: captures.name("name").unwrap().as_str().to_string(),
                        email,
                    })
                } else {
                    Err(Error::InvalidPackagerEmail(packager.to_string()))
                }
            } else {
                Err(Error::InvalidPackager(packager.to_string()))
            }
        } else {
            Err(Error::InvalidPackager(packager.to_string()))
        }
    }

    /// Return the name of the Packager
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the email of the Packager
    pub fn email(&self) -> &EmailAddress {
        &self.email
    }
}

impl FromStr for Packager {
    type Err = Error;
    /// Create a Packager from a string
    fn from_str(input: &str) -> Result<Packager, Self::Err> {
        Packager::new(input)
    }
}

impl Display for Packager {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{} <{}>", self.name, self.email)
    }
}

/// An option string used in packaging
///
/// The option string is identified by its name and whether it is on (not prefixed with "!") or off (prefixed with "!").
/// This type dereferences to `BuildOption`.
///
/// ## Examples
/// ```
/// use alpm_types::PackageOption;
///
/// let option = PackageOption::new("foo").unwrap();
/// assert_eq!(option.on(), true);
/// assert_eq!(option.name(), "foo");
///
/// let not_option = PackageOption::new("!foo").unwrap();
/// assert_eq!(not_option.on(), false);
/// assert_eq!(not_option.name(), "foo");
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageOption(BuildOption);

impl PackageOption {
    /// Create a new PackageOption in a Result
    pub fn new(option: &str) -> Result<Self, Error> {
        match BuildOption::new(option) {
            Ok(build_option) => Ok(PackageOption(build_option)),
            Err(_) => Err(Error::InvalidPackageOption(option.to_string())),
        }
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &BuildOption {
        &self.0
    }

    /// Get the name of the PackageOption
    pub fn name(&self) -> &str {
        self.inner().name()
    }

    /// Get whether the PackageOption is on
    pub fn on(&self) -> bool {
        self.inner().on()
    }
}

impl FromStr for PackageOption {
    type Err = Error;
    /// Create a PackageOption from a string
    fn from_str(input: &str) -> Result<PackageOption, Self::Err> {
        PackageOption::new(input)
    }
}

impl Display for PackageOption {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

/// The type of a package
///
/// ## Examples
/// ```
/// use std::str::FromStr;
/// use alpm_types::PkgType;
///
/// // create PkgType from str
/// assert_eq!(PkgType::from_str("pkg"), Ok(PkgType::Package));
///
/// // format as String
/// assert_eq!("debug", format!("{}", PkgType::Debug));
/// assert_eq!("pkg", format!("{}", PkgType::Package));
/// assert_eq!("src", format!("{}", PkgType::Source));
/// assert_eq!("split", format!("{}", PkgType::Split));
/// ```
#[derive(Debug, Display, EnumString, PartialEq)]
#[non_exhaustive]
pub enum PkgType {
    /// a debug package
    #[strum(to_string = "debug")]
    Debug,
    /// a single (non-split) package
    #[strum(to_string = "pkg")]
    Package,
    /// a source-only package
    #[strum(to_string = "src")]
    Source,
    /// one split package out of a set of several
    #[strum(to_string = "split")]
    Split,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use strum::ParseError;

    #[rstest]
    #[case("something", Ok(BuildEnv(BuildOption{name: "something".to_string(), on: true})))]
    #[case("!something", Ok(BuildEnv(BuildOption{name: "something".to_string(), on: false})))]
    #[case("foo\\", Err(Error::InvalidBuildEnv("foo\\".to_string())))]
    fn buildenv(#[case] from_str: &str, #[case] result: Result<BuildEnv, Error>) {
        assert_eq!(BuildEnv::from_str(from_str), result);
    }

    #[rstest]
    #[case("something", Ok(BuildOption{name: "something".to_string(), on: true}))]
    #[case("!something", Ok(BuildOption{name: "something".to_string(), on: false}))]
    #[case("foo\\", Err(Error::InvalidBuildOption("foo\\".to_string())))]
    fn buildoption(#[case] from_str: &str, #[case] result: Result<BuildOption, Error>) {
        assert_eq!(BuildOption::from_str(from_str), result);
    }

    #[rstest]
    #[case(
        "Foobar McFooface (The Third) <foobar@mcfooface.org>",
        Ok(Packager{
            name: "Foobar McFooface (The Third)".to_string(),
            email: EmailAddress::from_str("foobar@mcfooface.org").unwrap()
        })
    )]
    #[case(
        "Foobar McFooface <foobar@mcfooface.org>",
        Ok(Packager{
            name: "Foobar McFooface".to_string(),
            email: EmailAddress::from_str("foobar@mcfooface.org").unwrap()
        })
    )]
    #[case(
        "Foobar McFooface <@mcfooface.org>",
        Err(Error::InvalidPackagerEmail("Foobar McFooface <@mcfooface.org>".to_string())),
    )]
    #[case(
        "Foobar McFooface <foobar@mcfooface.org> <foobar@mcfoofacemcfooface.org>",
        Err(Error::InvalidPackagerEmail("Foobar McFooface <foobar@mcfooface.org> <foobar@mcfoofacemcfooface.org>".to_string())),
    )]
    #[case(
        "<foobar@mcfooface.org>",
        Err(Error::InvalidPackager("<foobar@mcfooface.org>".to_string())),
    )]
    #[case(
        "[foo] <foobar@mcfooface.org>",
        Err(Error::InvalidPackager("[foo] <foobar@mcfooface.org>".to_string())),
    )]
    #[case(
        "foobar@mcfooface.org",
        Err(Error::InvalidPackager("foobar@mcfooface.org".to_string())),
    )]
    #[case(
        "Foobar McFooface",
        Err(Error::InvalidPackager("Foobar McFooface".to_string())),
    )]
    fn packager(#[case] from_str: &str, #[case] result: Result<Packager, Error>) {
        assert_eq!(Packager::from_str(from_str), result);
    }

    #[rstest]
    #[case(
        Packager::new("Foobar McFooface <foobar@mcfooface.org>").unwrap(),
        "Foobar McFooface <foobar@mcfooface.org>"
    )]
    fn packager_format_string(#[case] packager: Packager, #[case] packager_str: &str) {
        assert_eq!(packager_str, format!("{}", packager));
    }

    #[rstest]
    #[case(Packager::new("Foobar McFooface <foobar@mcfooface.org>").unwrap(), "Foobar McFooface")]
    fn packager_name(#[case] packager: Packager, #[case] name: &str) {
        assert_eq!(name, packager.name());
    }

    #[rstest]
    #[case(
        Packager::new("Foobar McFooface <foobar@mcfooface.org>").unwrap(),
        &EmailAddress::from_str("foobar@mcfooface.org").unwrap(),
    )]
    fn packager_email(#[case] packager: Packager, #[case] email: &EmailAddress) {
        assert_eq!(email, packager.email());
    }

    #[rstest]
    #[case("something", Ok(PackageOption(BuildOption{name: "something".to_string(), on: true})))]
    #[case("!something", Ok(PackageOption(BuildOption{name: "something".to_string(), on: false})))]
    #[case("foo\\", Err(Error::InvalidPackageOption("foo\\".to_string())))]
    fn packageoption(#[case] from_str: &str, #[case] result: Result<PackageOption, Error>) {
        assert_eq!(PackageOption::from_str(from_str), result);
    }

    #[rstest]
    #[case("debug", Ok(PkgType::Debug))]
    #[case("pkg", Ok(PkgType::Package))]
    #[case("src", Ok(PkgType::Source))]
    #[case("split", Ok(PkgType::Split))]
    #[case("foo", Err(ParseError::VariantNotFound))]
    fn pkgtype_from_string(#[case] from_str: &str, #[case] result: Result<PkgType, ParseError>) {
        assert_eq!(PkgType::from_str(from_str), result);
    }

    #[rstest]
    #[case(PkgType::Debug, "debug")]
    #[case(PkgType::Package, "pkg")]
    #[case(PkgType::Source, "src")]
    #[case(PkgType::Split, "split")]
    fn pkgtype_format_string(#[case] pkgtype: PkgType, #[case] pkgtype_str: &str) {
        assert_eq!(pkgtype_str, format!("{}", pkgtype));
    }
}
