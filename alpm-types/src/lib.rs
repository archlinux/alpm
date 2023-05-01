// SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
// SPDX-License-Identifier: LGPL-3.0-or-later
use chrono::DateTime;
use chrono::Utc;

use std::fmt::Display;
use std::fmt::Formatter;
use std::ops::Deref;
use std::str::FromStr;
use std::string::ToString;

use email_address::EmailAddress;

use semver::Version;

use strum_macros::Display;
use strum_macros::EnumString;

mod error;
pub use error::Error;

mod macros;
use macros::regex_once;

/// CPU architecture
///
/// Members of the Architecture enum can be created from `&str`.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
/// use alpm_types::Architecture;
///
/// // create Architecture from str
/// assert_eq!(Architecture::from_str("aarch64"), Ok(Architecture::Aarch64));
///
/// // format as String
/// assert_eq!("aarch64", format!("{}", Architecture::Aarch64));
/// assert_eq!("any", format!("{}", Architecture::Any));
/// assert_eq!("arm", format!("{}", Architecture::Arm));
/// assert_eq!("armv6h", format!("{}", Architecture::Armv6h));
/// assert_eq!("armv7h", format!("{}", Architecture::Armv7h));
/// assert_eq!("i486", format!("{}", Architecture::I486));
/// assert_eq!("i686", format!("{}", Architecture::I686));
/// assert_eq!("pentium4", format!("{}", Architecture::Pentium4));
/// assert_eq!("riscv32", format!("{}", Architecture::Riscv32));
/// assert_eq!("riscv64", format!("{}", Architecture::Riscv64));
/// assert_eq!("x86_64", format!("{}", Architecture::X86_64));
/// assert_eq!("x86_64_v2", format!("{}", Architecture::X86_64V2));
/// assert_eq!("x86_64_v3", format!("{}", Architecture::X86_64V3));
/// assert_eq!("x86_64_v4", format!("{}", Architecture::X86_64V4));
/// ```
#[derive(Debug, Display, EnumString, Eq, PartialEq)]
#[non_exhaustive]
pub enum Architecture {
    #[strum(to_string = "aarch64")]
    Aarch64,
    #[strum(to_string = "any")]
    Any,
    #[strum(to_string = "arm")]
    Arm,
    #[strum(to_string = "armv6h")]
    Armv6h,
    #[strum(to_string = "armv7h")]
    Armv7h,
    #[strum(to_string = "i486")]
    I486,
    #[strum(to_string = "i686")]
    I686,
    #[strum(to_string = "pentium4")]
    Pentium4,
    #[strum(to_string = "riscv32")]
    Riscv32,
    #[strum(to_string = "riscv64")]
    Riscv64,
    #[strum(to_string = "x86_64")]
    X86_64,
    #[strum(to_string = "x86_64_v2")]
    X86_64V2,
    #[strum(to_string = "x86_64_v3")]
    X86_64V3,
    #[strum(to_string = "x86_64_v4")]
    X86_64V4,
}

/// A build date in seconds since the epoch
///
/// # Examples
/// ```
/// use alpm_types::{BuildDate, Error};
/// use chrono::{DateTime, NaiveDateTime, Utc};
/// use std::str::FromStr;
///
/// // create BuildDate from DateTime<Utc>
/// let datetime: BuildDate =
/// DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(1, 0).unwrap(), Utc).into();
/// assert_eq!(BuildDate::new(1), datetime);
///
/// // create BuildDate from &str
/// assert_eq!(BuildDate::from_str("1"), Ok(BuildDate::new(1)));
/// assert_eq!(
///     BuildDate::from_str("foo"),
///     Err(Error::InvalidBuildDate(String::from("foo")))
/// );
///
/// // format as String
/// assert_eq!("1", format!("{}", BuildDate::new(1)));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildDate {
    date: i64,
}

impl BuildDate {
    /// Create a new BuildDate
    pub fn new(date: i64) -> BuildDate {
        BuildDate { date }
    }
}

impl From<DateTime<Utc>> for BuildDate {
    fn from(input: DateTime<Utc>) -> BuildDate {
        let date = input.timestamp();
        BuildDate { date }
    }
}

impl FromStr for BuildDate {
    type Err = Error;
    /// Create a BuildDate from a string
    fn from_str(input: &str) -> Result<BuildDate, Self::Err> {
        match input.parse::<i64>() {
            Ok(date) => Ok(BuildDate { date }),
            _ => Err(Error::InvalidBuildDate(input.to_string())),
        }
    }
}

impl Display for BuildDate {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.date)
    }
}

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

    /// Get the name of the BuildEnv
    pub fn name(&self) -> &str {
        self.deref().name()
    }

    /// Get whether the BuildEnv is on
    pub fn on(&self) -> bool {
        self.deref().on()
    }
}

impl FromStr for BuildEnv {
    type Err = Error;
    /// Create a BuildEnv from a string
    fn from_str(input: &str) -> Result<BuildEnv, Self::Err> {
        BuildEnv::new(input)
    }
}

impl Deref for BuildEnv {
    type Target = BuildOption;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for BuildEnv {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.deref())
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

/// A build tool name
///
/// The same character restrictions as with `Name` apply.
/// Further name restrictions may be enforced on an existing instances using `matches_restriction()`.
///
/// ## Examples
/// ```
/// use alpm_types::{BuildTool, Name, Error};
/// use std::str::FromStr;
///
/// // create BuildTool from &str
/// assert_eq!(
///     BuildTool::from_str("test-123@.foo_+"),
///     Ok(BuildTool::new("test-123@.foo_+").unwrap()),
/// );
/// assert_eq!(
///     BuildTool::from_str(".test"),
///     Err(Error::InvalidBuildTool(".test".to_string()))
/// );
///
/// // format as String
/// assert_eq!("foo", format!("{}", BuildTool::new("foo").unwrap()));
///
/// // validate that BuildTool follows naming restrictions
/// let buildtool = BuildTool::new("foo").unwrap();
/// let restrictions = vec![Name::new("foo").unwrap(), Name::new("bar").unwrap()];
/// assert!(buildtool.matches_restriction(&restrictions));
/// ```
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BuildTool(Name);

impl BuildTool {
    /// Create a new BuildTool in a Result
    pub fn new(buildtool: &str) -> Result<Self, Error> {
        match Name::new(buildtool) {
            Ok(name) => Ok(BuildTool(name)),
            Err(_) => Err(Error::InvalidBuildTool(buildtool.to_string())),
        }
    }

    /// Create a new BuildTool in a Result, which matches one Name in a list of restrictions
    ///
    /// ## Examples
    /// ```
    /// use alpm_types::{BuildTool, Name, Error};
    ///
    /// assert!(BuildTool::new_with_restriction("foo", &[Name::new("foo").unwrap()]).is_ok());
    /// assert!(BuildTool::new_with_restriction("foo", &[Name::new("bar").unwrap()]).is_err());
    /// ```
    pub fn new_with_restriction(name: &str, restrictions: &[Name]) -> Result<Self, Error> {
        match BuildTool::new(name) {
            Ok(buildtool) => {
                if buildtool.matches_restriction(restrictions) {
                    Ok(buildtool)
                } else {
                    Err(Error::InvalidBuildTool(name.to_string()))
                }
            }
            Err(_) => Err(Error::InvalidBuildTool(name.to_string())),
        }
    }

    /// Validate that the BuildTool has a name matching one Name in a list of restrictions
    pub fn matches_restriction(&self, restrictions: &[Name]) -> bool {
        restrictions
            .iter()
            .any(|restriction| restriction.eq(self.deref()))
    }
}

impl FromStr for BuildTool {
    type Err = Error;
    /// Create a BuildTool from a string
    fn from_str(input: &str) -> Result<BuildTool, Self::Err> {
        BuildTool::new(input)
    }
}

impl Deref for BuildTool {
    type Target = Name;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for BuildTool {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.deref())
    }
}

/// Compressed size of a file (in bytes)
///
/// ## Examples
/// ```
/// use alpm_types::{CompressedSize, Error};
/// use std::str::FromStr;
///
/// // create CompressedSize from &str
/// assert_eq!(
///     CompressedSize::from_str("1"),
///     Ok(CompressedSize::new(1))
/// );
/// assert_eq!(
///     CompressedSize::from_str("-1"),
///     Err(Error::InvalidCompressedSize(String::from("-1")))
/// );
///
/// // format as String
/// assert_eq!("1", format!("{}", CompressedSize::new(1)));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompressedSize {
    size: u64,
}

impl CompressedSize {
    /// Create a new CompressedSize
    pub fn new(size: u64) -> CompressedSize {
        CompressedSize { size }
    }
}

impl FromStr for CompressedSize {
    type Err = Error;
    /// Create a CompressedSize from a string
    fn from_str(input: &str) -> Result<CompressedSize, Self::Err> {
        match input.parse::<u64>() {
            Ok(size) => Ok(CompressedSize { size }),
            _ => Err(Error::InvalidCompressedSize(input.to_string())),
        }
    }
}

impl Display for CompressedSize {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.size)
    }
}

/// Installed size of a package (in bytes)
///
/// ## Examples
/// ```
/// use alpm_types::{InstalledSize, Error};
/// use std::str::FromStr;
///
/// // create InstalledSize from &str
/// assert_eq!(InstalledSize::from_str("1"), Ok(InstalledSize::new(1)));
/// assert_eq!(
///     InstalledSize::from_str("-1"),
///     Err(Error::InvalidInstalledSize(String::from("-1")))
/// );
///
/// // format as String
/// assert_eq!("1", format!("{}", InstalledSize::new(1)));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InstalledSize {
    size: u64,
}

impl InstalledSize {
    /// Create a new InstalledSize
    pub fn new(size: u64) -> InstalledSize {
        InstalledSize { size }
    }
}

impl FromStr for InstalledSize {
    type Err = Error;
    /// Create a InstalledSize from a string
    fn from_str(input: &str) -> Result<InstalledSize, Self::Err> {
        match input.parse::<u64>() {
            Ok(size) => Ok(InstalledSize { size }),
            _ => Err(Error::InvalidInstalledSize(input.to_string())),
        }
    }
}

impl Display for InstalledSize {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.size)
    }
}

/// A single 'md5sum' attribute
///
/// Md5Sum consists of 32 characters `[a-f0-9]`.
///
/// ## Examples
/// ```
/// use alpm_types::{Md5Sum, Error};
/// use std::str::FromStr;
///
/// // create Md5Sum from &str
/// assert_eq!(
///     Md5Sum::from_str("5eb63bbbe01eeed093cb22bb8f5acdc3"),
///     Ok(Md5Sum::new("5eb63bbbe01eeed093cb22bb8f5acdc3").unwrap())
/// );
/// assert_eq!(
///     Md5Sum::from_str("foobar"),
///     Err(Error::InvalidMd5Sum("foobar".to_string()))
/// );
///
/// // format as String
/// assert_eq!("5eb63bbbe01eeed093cb22bb8f5acdc3", format!("{}", Md5Sum::new("5eb63bbbe01eeed093cb22bb8f5acdc3").unwrap()));
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Md5Sum {
    md5sum: String,
}

impl Md5Sum {
    /// Create a new Md5Sum in a Result
    ///
    /// If the supplied string is valid on the basis of the allowed characters
    /// then an Md5Sum is returned as a Result, otherwise an InvalidMd5Sum Error
    /// is returned.
    pub fn new(md5sum: &str) -> Result<Md5Sum, Error> {
        if regex_once!(r"^[a-f0-9]{32}$").is_match(md5sum) {
            Ok(Md5Sum {
                md5sum: md5sum.to_string(),
            })
        } else {
            Err(Error::InvalidMd5Sum(md5sum.to_string()))
        }
    }
}

impl FromStr for Md5Sum {
    type Err = Error;
    /// Create a Md5Sum from a string
    fn from_str(input: &str) -> Result<Md5Sum, Self::Err> {
        Md5Sum::new(input)
    }
}

impl Display for Md5Sum {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.md5sum)
    }
}

/// A package name
///
/// Package names may contain the characters `[a-z\d\-._@+]`, but must not
/// start with `[-.]`.
///
/// ## Examples
/// ```
/// use alpm_types::{Name, Error};
/// use std::str::FromStr;
///
/// // create Name from &str
/// assert_eq!(
///     Name::from_str("test-123@.foo_+"),
///     Ok(Name::new("test-123@.foo_+").unwrap())
/// );
/// assert_eq!(
///     Name::from_str(".test"),
///     Err(Error::InvalidName(".test".to_string()))
/// );
///
/// // format as String
/// assert_eq!("foo", format!("{}", Name::new("foo").unwrap()));
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Name(String);

impl Name {
    /// Create a new Name in a Result
    pub fn new(name: &str) -> Result<Self, Error> {
        Name::validate(name)
    }

    /// Validate a string and return a Name in a Result
    ///
    /// The validation happens on the basis of the allowed characters as
    /// defined by the Name type.
    pub fn validate(name: &str) -> Result<Name, Error> {
        if regex_once!(r"^[a-z\d_@+]+[a-z\d\-._@+]*$").is_match(name) {
            Ok(Name(name.to_string()))
        } else {
            Err(Error::InvalidName(name.to_string()))
        }
    }
}

impl FromStr for Name {
    type Err = Error;
    /// Create a Name from a string
    fn from_str(input: &str) -> Result<Name, Self::Err> {
        Name::validate(input)
    }
}

impl Deref for Name {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Name {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.deref())
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

    /// Get the name of the PackageOption
    pub fn name(&self) -> &str {
        self.deref().name()
    }

    /// Get whether the PackageOption is on
    pub fn on(&self) -> bool {
        self.deref().on()
    }
}

impl FromStr for PackageOption {
    type Err = Error;
    /// Create a PackageOption from a string
    fn from_str(input: &str) -> Result<PackageOption, Self::Err> {
        PackageOption::new(input)
    }
}

impl Deref for PackageOption {
    type Target = BuildOption;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for PackageOption {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.deref())
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

/// The schema version of a type
///
/// A `SchemaVersion` wraps a `semver::Version`, which means that the tracked version should follow [semver](https://semver.org).
/// However, for backwards compatibility reasons it is possible to initialize a `SchemaVersion` using a non-semver
/// compatible string, *if* it can be parsed to a single `u64` (e.g. `"1"`).
///
/// Examples
/// ```
/// use std::str::FromStr;
/// use alpm_types::SchemaVersion;
/// use semver::Version;
///
/// // create SchemaVersion from str
/// let version_one = SchemaVersion::from_str("1.0.0").unwrap();
/// let version_also_one = SchemaVersion::new("1").unwrap();
/// assert_eq!(version_one, version_also_one);
///
/// // format as String
/// assert_eq!("1.0.0", format!("{}", version_one));
/// assert_eq!("1.0.0", format!("{}", version_also_one));
/// ```
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SchemaVersion(Version);

impl SchemaVersion {
    /// Create a new SchemaVersion from a string
    ///
    /// When providing a non-semver string with only a number (i.e. no minor or patch version), the number is treated as
    /// the major version (e.g. `"23"` -> `"23.0.0"`).
    pub fn new(version: &str) -> Result<SchemaVersion, Error> {
        if !version.contains('.') {
            match version.parse() {
                Ok(major) => Ok(SchemaVersion(Version::new(major, 0, 0))),
                Err(_) => Err(Error::InvalidVersion(version.to_string())),
            }
        } else {
            match Version::parse(version) {
                Ok(version) => Ok(SchemaVersion(version)),
                Err(_) => Err(Error::InvalidVersion(version.to_string())),
            }
        }
    }
}

impl FromStr for SchemaVersion {
    type Err = Error;
    /// Create a SchemaVersion from a string
    fn from_str(input: &str) -> Result<SchemaVersion, Self::Err> {
        SchemaVersion::new(input)
    }
}

impl Deref for SchemaVersion {
    type Target = Version;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for SchemaVersion {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use proptest::prelude::*;
    use rstest::rstest;
    use strum::ParseError;

    #[rstest]
    #[case("aarch64", Ok(Architecture::Aarch64))]
    #[case("any", Ok(Architecture::Any))]
    #[case("arm", Ok(Architecture::Arm))]
    #[case("armv6h", Ok(Architecture::Armv6h))]
    #[case("armv7h", Ok(Architecture::Armv7h))]
    #[case("i486", Ok(Architecture::I486))]
    #[case("i686", Ok(Architecture::I686))]
    #[case("pentium4", Ok(Architecture::Pentium4))]
    #[case("riscv32", Ok(Architecture::Riscv32))]
    #[case("riscv64", Ok(Architecture::Riscv64))]
    #[case("x86_64", Ok(Architecture::X86_64))]
    #[case("x86_64_v2", Ok(Architecture::X86_64V2))]
    #[case("x86_64_v3", Ok(Architecture::X86_64V3))]
    #[case("x86_64_v4", Ok(Architecture::X86_64V4))]
    #[case("foo", Err(ParseError::VariantNotFound))]
    fn architecture_from_string(
        #[case] from_str: &str,
        #[case] arch: Result<Architecture, ParseError>,
    ) {
        assert_eq!(Architecture::from_str(from_str), arch);
    }

    #[rstest]
    #[case(Architecture::Aarch64, "aarch64")]
    #[case(Architecture::Any, "any")]
    #[case(Architecture::Arm, "arm")]
    #[case(Architecture::Armv6h, "armv6h")]
    #[case(Architecture::Armv7h, "armv7h")]
    #[case(Architecture::I486, "i486")]
    #[case(Architecture::I686, "i686")]
    #[case(Architecture::Pentium4, "pentium4")]
    #[case(Architecture::Riscv32, "riscv32")]
    #[case(Architecture::Riscv64, "riscv64")]
    #[case(Architecture::X86_64, "x86_64")]
    #[case(Architecture::X86_64V2, "x86_64_v2")]
    #[case(Architecture::X86_64V3, "x86_64_v3")]
    #[case(Architecture::X86_64V4, "x86_64_v4")]
    fn architecture_format_string(#[case] arch: Architecture, #[case] arch_str: &str) {
        assert_eq!(arch_str, format!("{}", arch));
    }

    #[rstest]
    #[case("1", Ok(BuildDate { date: 1 }))]
    #[case("foo", Err(Error::InvalidBuildDate(String::from("foo"))))]
    fn builddate_from_string(#[case] from_str: &str, #[case] result: Result<BuildDate, Error>) {
        assert_eq!(BuildDate::from_str(from_str), result);
    }

    #[rstest]
    fn builddate_format_string() {
        assert_eq!("1", format!("{}", BuildDate::new(1)));
    }

    #[rstest]
    fn datetime_into_builddate() {
        let builddate = BuildDate { date: 1 };
        let datetime: BuildDate =
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(1, 0).unwrap(), Utc).into();
        assert_eq!(builddate, datetime);
    }

    #[rstest]
    #[case("bar", vec![Name::new("foo").unwrap(), Name::new("bar").unwrap()], Ok(BuildTool::new("bar").unwrap()))]
    #[case("bar", vec![Name::new("foo").unwrap(), Name::new("foo").unwrap()], Err(Error::InvalidBuildTool("bar".to_string())))]
    fn buildtool_new_with_restriction(
        #[case] buildtool: &str,
        #[case] restrictions: Vec<Name>,
        #[case] result: Result<BuildTool, Error>,
    ) {
        assert_eq!(
            BuildTool::new_with_restriction(buildtool, &restrictions),
            result
        );
    }

    #[rstest]
    #[case("bar", vec![Name::new("foo").unwrap(), Name::new("bar").unwrap()], true)]
    #[case("bar", vec![Name::new("foo").unwrap(), Name::new("foo").unwrap()], false)]
    fn buildtool_matches_restriction(
        #[case] buildtool: &str,
        #[case] restrictions: Vec<Name>,
        #[case] result: bool,
    ) {
        let buildtool = BuildTool::new(buildtool).unwrap();
        assert_eq!(buildtool.matches_restriction(&restrictions), result);
    }

    #[rstest]
    #[case("1", Ok(CompressedSize::new(1)))]
    #[case("-1", Err(Error::InvalidCompressedSize(String::from("-1"))))]
    fn compressedsize_from_string(
        #[case] from_str: &str,
        #[case] result: Result<CompressedSize, Error>,
    ) {
        assert_eq!(CompressedSize::from_str(from_str), result);
    }

    #[rstest]
    fn compressedsize_format_string() {
        assert_eq!("1", format!("{}", CompressedSize::new(1)));
    }

    #[rstest]
    #[case("1", Ok(InstalledSize::new(1)))]
    #[case("-1", Err(Error::InvalidInstalledSize(String::from("-1"))))]
    fn installedsize_from_string(
        #[case] from_str: &str,
        #[case] result: Result<InstalledSize, Error>,
    ) {
        assert_eq!(InstalledSize::from_str(from_str), result);
    }

    #[rstest]
    fn installedsize_format_string() {
        assert_eq!("1", format!("{}", InstalledSize::new(1)));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn valid_md5sum_from_string(md5sum_str in r"[a-f0-9]{32}") {
            let md5sum = Md5Sum::from_str(&md5sum_str).unwrap();
            prop_assert_eq!(md5sum_str, format!("{}", md5sum));
        }

        #[test]
        fn invalid_md5sum_from_string_bigger_size(md5sum_str in r"[a-f0-9]{64}") {
            let error = Md5Sum::from_str(&md5sum_str).unwrap_err();
            assert!(format!("{}", error).ends_with(&md5sum_str));
        }

        #[test]
        fn invalid_md5sum_from_string_smaller_size(md5sum_str in r"[a-f0-9]{16}") {
            let error = Md5Sum::from_str(&md5sum_str).unwrap_err();
            assert!(format!("{}", error).ends_with(&md5sum_str));
        }

        #[test]
        fn invalid_md5sum_from_string_wrong_chars(md5sum_str in r"[e-z0-9]{32}") {
            let error = Md5Sum::from_str(&md5sum_str).unwrap_err();
            assert!(format!("{}", error).ends_with(&md5sum_str));
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn valid_name_from_string(name_str in r"[a-z\d_@+]+[a-z\d\-._@+]*") {
            let name = Name::from_str(&name_str).unwrap();
            prop_assert_eq!(name_str, format!("{}", name));
        }

        #[test]
        fn invalid_name_from_string_start(name_str in r"[\-.]+[a-z\d\-._@+]*") {
            let error = Name::from_str(&name_str).unwrap_err();
            assert!(format!("{}", error).ends_with(&name_str));
        }
    }

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

    #[rstest]
    #[case("1.0.0", Ok(SchemaVersion(Version::new(1, 0, 0))))]
    #[case("1", Ok(SchemaVersion(Version::new(1, 0, 0))))]
    #[case("-1.0.0", Err(Error::InvalidVersion("-1.0.0".to_string())))]
    fn schema_version(#[case] version: &str, #[case] result: Result<SchemaVersion, Error>) {
        assert_eq!(result, SchemaVersion::new(version))
    }

    #[rstest]
    #[case(
        SchemaVersion(Version::new(1, 0, 0)),
        SchemaVersion(Version::new(0, 1, 0))
    )]
    fn compare_schema_version(#[case] version_a: SchemaVersion, #[case] version_b: SchemaVersion) {
        assert!(version_a > version_b);
    }
}
