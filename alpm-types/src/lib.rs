// SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
// SPDX-License-Identifier: LGPL-3.0-or-later
use std::cmp::Ordering;
use std::fmt::Display;
use std::fmt::Formatter;
use std::num::NonZeroUsize;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::ToString;

use chrono::DateTime;
use chrono::Utc;

use email_address::EmailAddress;

use semver::Version as SemverVersion;

use strum_macros::Display;
use strum_macros::EnumString;

mod error;
pub use error::Error;

mod macros;
use macros::regex_once;

/// A representation of an absolute path
///
/// AbsolutePath wraps a `PathBuf`, that is guaranteed to be absolute.
///
/// ## Examples
/// ```
/// use alpm_types::{AbsolutePath, Error};
/// use std::str::FromStr;
///
/// // create BuildDir from &str
/// assert_eq!(
///     AbsolutePath::from_str("/"),
///     Ok(AbsolutePath::new("/").unwrap())
/// );
/// assert_eq!(
///     AbsolutePath::from_str("./"),
///     Err(Error::InvalidAbsolutePath(String::from("./")))
/// );
///
/// // format as String
/// assert_eq!("/", format!("{}", AbsolutePath::new("/").unwrap()));
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbsolutePath(PathBuf);

impl AbsolutePath {
    pub fn new(input: &str) -> Result<AbsolutePath, Error> {
        match Path::new(input).is_absolute() {
            true => Ok(AbsolutePath(PathBuf::from(input))),
            false => Err(Error::InvalidAbsolutePath(input.to_string())),
        }
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &Path {
        &self.0
    }
}

impl FromStr for AbsolutePath {
    type Err = Error;
    fn from_str(input: &str) -> Result<AbsolutePath, Self::Err> {
        AbsolutePath::new(input)
    }
}

impl Display for AbsolutePath {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner().display())
    }
}

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
pub struct BuildDate(i64);

impl BuildDate {
    /// Create a new BuildDate
    pub fn new(builddate: i64) -> BuildDate {
        BuildDate(builddate)
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &i64 {
        &self.0
    }
}

impl From<DateTime<Utc>> for BuildDate {
    fn from(input: DateTime<Utc>) -> BuildDate {
        let builddate = input.timestamp();
        BuildDate(builddate)
    }
}

impl FromStr for BuildDate {
    type Err = Error;
    /// Create a BuildDate from a string
    fn from_str(input: &str) -> Result<BuildDate, Self::Err> {
        match input.parse::<i64>() {
            Ok(builddate) => Ok(BuildDate(builddate)),
            _ => Err(Error::InvalidBuildDate(input.to_string())),
        }
    }
}

impl Display for BuildDate {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

/// An absolute path used as build directory
///
/// BuildDir wraps an `AbsolutePath`
///
/// ## Examples
/// ```
/// use alpm_types::{BuildDir, Error};
/// use std::str::FromStr;
///
/// // create BuildDir from &str
/// assert_eq!(
///     BuildDir::from_str("/"),
///     Ok(BuildDir::new("/").unwrap())
/// );
/// assert_eq!(
///     BuildDir::from_str("/foo.txt"),
///     Ok(BuildDir::new("/foo.txt").unwrap())
/// );
///
/// // format as String
/// assert_eq!("/", format!("{}", BuildDir::new("/").unwrap()));
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildDir(AbsolutePath);

impl BuildDir {
    /// Create a new BuildDir
    pub fn new(absolute_path: &str) -> Result<BuildDir, Error> {
        match AbsolutePath::new(absolute_path) {
            Ok(abs_path) => Ok(BuildDir(abs_path)),
            _ => Err(Error::InvalidBuildDir(absolute_path.to_string())),
        }
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &AbsolutePath {
        &self.0
    }
}

impl FromStr for BuildDir {
    type Err = Error;
    fn from_str(absolute_path: &str) -> Result<BuildDir, Self::Err> {
        BuildDir::new(absolute_path)
    }
}

impl Display for BuildDir {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
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
            .any(|restriction| restriction.eq(self.inner()))
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &Name {
        &self.0
    }
}

impl FromStr for BuildTool {
    type Err = Error;
    /// Create a BuildTool from a string
    fn from_str(input: &str) -> Result<BuildTool, Self::Err> {
        BuildTool::new(input)
    }
}

impl Display for BuildTool {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
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
pub struct CompressedSize(u64);

impl CompressedSize {
    /// Create a new CompressedSize
    pub fn new(compressedsize: u64) -> CompressedSize {
        CompressedSize(compressedsize)
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &u64 {
        &self.0
    }
}

impl FromStr for CompressedSize {
    type Err = Error;
    /// Create a CompressedSize from a string
    fn from_str(input: &str) -> Result<CompressedSize, Self::Err> {
        match input.parse::<u64>() {
            Ok(compressedsize) => Ok(CompressedSize(compressedsize)),
            _ => Err(Error::InvalidCompressedSize(input.to_string())),
        }
    }
}

impl Display for CompressedSize {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
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
pub struct Md5Sum(String);

impl Md5Sum {
    /// Create a new Md5Sum in a Result
    ///
    /// If the supplied string is valid on the basis of the allowed characters
    /// then an Md5Sum is returned as a Result, otherwise an InvalidMd5Sum Error
    /// is returned.
    pub fn new(md5sum: &str) -> Result<Md5Sum, Error> {
        if regex_once!(r"^[a-f0-9]{32}$").is_match(md5sum) {
            Ok(Md5Sum(md5sum.to_string()))
        } else {
            Err(Error::InvalidMd5Sum(md5sum.to_string()))
        }
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &str {
        &self.0
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
        write!(fmt, "{}", self.inner())
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

    /// Return a reference to the inner type
    pub fn inner(&self) -> &str {
        &self.0
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

impl Display for Name {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
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

/// The schema version of a type
///
/// A `SchemaVersion` wraps a `semver::Version`, which means that the tracked version should follow [semver](https://semver.org).
/// However, for backwards compatibility reasons it is possible to initialize a `SchemaVersion` using a non-semver
/// compatible string, *if* it can be parsed to a single `u64` (e.g. `"1"`).
///
/// ## Examples
/// ```
/// use std::str::FromStr;
/// use alpm_types::SchemaVersion;
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
pub struct SchemaVersion(SemverVersion);

impl SchemaVersion {
    /// Create a new SchemaVersion from a string
    ///
    /// When providing a non-semver string with only a number (i.e. no minor or patch version), the number is treated as
    /// the major version (e.g. `"23"` -> `"23.0.0"`).
    pub fn new(version: &str) -> Result<SchemaVersion, Error> {
        if !version.contains('.') {
            match version.parse() {
                Ok(major) => Ok(SchemaVersion(SemverVersion::new(major, 0, 0))),
                Err(_) => Err(Error::InvalidVersion(version.to_string())),
            }
        } else {
            match SemverVersion::parse(version) {
                Ok(version) => Ok(SchemaVersion(version)),
                Err(_) => Err(Error::InvalidVersion(version.to_string())),
            }
        }
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &SemverVersion {
        &self.0
    }
}

impl FromStr for SchemaVersion {
    type Err = Error;
    /// Create a SchemaVersion from a string
    fn from_str(input: &str) -> Result<SchemaVersion, Self::Err> {
        SchemaVersion::new(input)
    }
}

impl Display for SchemaVersion {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

/// An epoch of a package
///
/// Epoch is used to indicate the downgrade of a package and is prepended to a version, delimited by a `":"` (e.g. `1:`
/// is added to `0.10.0-1` to form `1:0.10.0-1` which then orders newer than `1.0.0-1`).
///
/// An Epoch wraps a usize that is guaranteed to be greater than `0`.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
/// use alpm_types::Epoch;
///
/// assert!(Epoch::new("1".to_string()).is_ok());
/// assert!(Epoch::new("0".to_string()).is_err());
/// ```
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Epoch(NonZeroUsize);

impl Epoch {
    /// Create a new Epoch from a string and return it in a Result
    pub fn new(epoch: String) -> Result<Self, Error> {
        match epoch.parse() {
            Ok(epoch) => Ok(Epoch(epoch)),
            Err(_) => Err(Error::InvalidEpoch(epoch)),
        }
    }

    // Return a reference to the inner type
    pub fn inner(&self) -> NonZeroUsize {
        self.0
    }
}

impl FromStr for Epoch {
    type Err = Error;
    /// Create an Epoch from a string and return it in a Result
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Epoch::new(input.to_string())
    }
}

impl Display for Epoch {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

/// A pkgrel of a package
///
/// Pkgrel is used to indicate the build version of a package and is appended to a version, delimited by a `"-"` (e.g.
/// `-2` is added to `1.0.0` to form `1.0.0-2` which then orders newer than `1.0.0-1`).
///
/// A Pkgrel wraps a String which is guaranteed to not start with a `"0"`, to contain only numeric characters
/// (optionally delimited by a single `"."`, which must be followed by at least one non-`"0"` numeric character).
///
/// ## Examples
/// ```
/// use std::str::FromStr;
/// use alpm_types::Pkgrel;
///
/// assert!(Pkgrel::new("1".to_string()).is_ok());
/// assert!(Pkgrel::new("1.1".to_string()).is_ok());
/// assert!(Pkgrel::new("0".to_string()).is_err());
/// assert!(Pkgrel::new("0.1".to_string()).is_err());
/// assert!(Pkgrel::new("1.0".to_string()).is_err());
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Pkgrel(String);

impl Pkgrel {
    /// Create a new Pkgrel from a string and return it in a Result
    pub fn new(pkgrel: String) -> Result<Self, Error> {
        if regex_once!(r"^[1-9]+[0-9]*(|[.]{1}[1-9]+[0-9]*)$").is_match(pkgrel.as_str()) {
            Ok(Pkgrel(pkgrel))
        } else {
            Err(Error::InvalidPkgrel(pkgrel))
        }
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &str {
        &self.0
    }
}

impl FromStr for Pkgrel {
    type Err = Error;
    /// Create a Pkgrel from a string and return it in a Result
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Pkgrel::new(input.to_string())
    }
}

impl Display for Pkgrel {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

/// A pkgver of a package
///
/// Pkgver is used to denote the upstream version of a package.
///
/// A Pkgver wraps a `String`, which is guaranteed to only contain alphanumeric characters, `"_"`, `"+"` or `"."`, but
/// to not start with a `"_"`, a `"+"` or a `"."` character and to be at least one char long.
///
/// NOTE: This implementation of Pkgver is stricter than that of libalpm/pacman. It does not allow empty strings `""`,
/// or chars that are not in the allowed set, or `"."` as the first character.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
/// use alpm_types::Pkgver;
///
/// assert!(Pkgver::new("1".to_string()).is_ok());
/// assert!(Pkgver::new("1.1".to_string()).is_ok());
/// assert!(Pkgver::new("foo".to_string()).is_ok());
/// assert!(Pkgver::new("0".to_string()).is_ok());
/// assert!(Pkgver::new(".0.1".to_string()).is_err());
/// assert!(Pkgver::new("_1.0".to_string()).is_err());
/// assert!(Pkgver::new("+1.0".to_string()).is_err());
/// ```
#[derive(Clone, Debug, Eq)]
pub struct Pkgver(String);

impl Pkgver {
    /// Create a new Pkgver from a string and return it in a Result
    pub fn new(pkgver: String) -> Result<Self, Error> {
        if regex_once!(r"^([^_+.][[:alnum:]_+.]*)$").is_match(pkgver.as_str()) {
            Ok(Pkgver(pkgver))
        } else {
            Err(Error::InvalidPkgver(pkgver))
        }
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &str {
        &self.0
    }
}

impl FromStr for Pkgver {
    type Err = Error;
    /// Create a Pkgver from a string and return it in a Result
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Pkgver::new(input.to_string())
    }
}

impl Display for Pkgver {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

impl Ord for Pkgver {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_inner = self.inner();
        let other_inner = other.inner();

        // easy comparison to see if versions are identical
        if self_inner == other_inner {
            return Ordering::Equal;
        }

        // Strings for temporarily holding leftovers when comparing
        let mut self_leftover;
        let mut other_leftover;
        // Indices used as left hand pointers for section starts when comparing self and other
        let mut self_left_index = 0;
        let mut other_left_index = 0;
        // Indices used as right hand pointers for section ends when comparing self and other
        let mut self_right_index = 0;
        let mut other_right_index = 0;

        // loop through each version segment of a and b and compare them
        while self_left_index < self_inner.len() && other_left_index < other_inner.len() {
            // set self_left_index to the location of the last alphanumeric char in one
            while self_left_index < self_inner.len()
                && !self_inner
                    .chars()
                    .nth(self_left_index)
                    .unwrap()
                    .is_alphanumeric()
            {
                self_left_index += 1;
            }
            // set other_left_index to the location of the last alphanumeric char in two
            while other_left_index < other_inner.len()
                && !other
                    .inner()
                    .chars()
                    .nth(other_left_index)
                    .unwrap()
                    .is_alphanumeric()
            {
                other_left_index += 1;
            }

            // If we ran to the end of either, we are finished with the loop
            if self_left_index >= self_inner.len() || other_left_index >= other_inner.len() {
                break;
            }

            // If the separator lengths were different, we are finished
            if (self_left_index - self_right_index) != (other_left_index - other_right_index) {
                return if (self_left_index - self_right_index)
                    < (other_left_index - other_right_index)
                {
                    Ordering::Less
                } else {
                    Ordering::Greater
                };
            }

            // adjust left side pointer to current segment start
            self_right_index = self_left_index;
            other_right_index = other_left_index;
            self_leftover = if let Some(leftover) = self_inner.get(self_left_index..) {
                leftover.to_string()
            } else {
                "".to_string()
            };
            other_leftover = if let Some(leftover) = other_inner.get(other_left_index..) {
                leftover.to_string()
            } else {
                "".to_string()
            };

            // grab first completely alpha or completely numeric segment leave one and two pointing to the start of the
            // alpha or numeric segment and walk self_right_index and other_right_index to end of segment
            let isnum = if !self_leftover.is_empty()
                && self_leftover.chars().next().unwrap().is_numeric()
            {
                self_right_index += self_leftover.chars().take_while(|x| x.is_numeric()).count();
                other_right_index += other_leftover
                    .chars()
                    .take_while(|x| x.is_numeric())
                    .count();
                true
            } else {
                self_right_index += self_leftover
                    .chars()
                    .take_while(|x| x.is_alphabetic())
                    .count();
                other_right_index += other_leftover
                    .chars()
                    .take_while(|x| x.is_alphabetic())
                    .count();
                false
            };

            // adjust current segment end with the updated right side pointer
            self_leftover =
                if let Some(leftover) = self_inner.get(self_left_index..self_right_index) {
                    leftover.to_string()
                } else {
                    "".to_string()
                };
            other_leftover =
                if let Some(leftover) = other_inner.get(other_left_index..other_right_index) {
                    leftover.to_string()
                } else {
                    "".to_string()
                };

            // take care of the case where the two version segments are different types: one numeric, the other alpha
            // (i.e. empty) numeric segments are always newer than alpha segments
            if other_leftover.is_empty() {
                return if isnum {
                    Ordering::Greater
                } else {
                    Ordering::Less
                };
            }

            if isnum {
                // throw away any leading zeros - it's a number, right?
                self_leftover = self_leftover.trim_start_matches('0').to_string();
                other_leftover = other_leftover.trim_start_matches('0').to_string();

                // whichever number has more digits wins (discard leading zeros)
                match (self_leftover.len(), other_leftover.len()) {
                    (one_len, two_len) if one_len > two_len => return Ordering::Greater,
                    (one_len, two_len) if one_len < two_len => return Ordering::Less,
                    (_, _) => {}
                }
            }

            // strcmp will return which one is greater - even if the two segments are alpha or if they are numeric.
            // don't return if they are equal because there might be more segments to compare
            if self_leftover.cmp(&other_leftover).is_ne() {
                return self_leftover.cmp(&other_leftover);
            }

            // advance left side pointer to current right side pointer
            self_left_index = self_right_index;
            other_left_index = other_right_index;
        }

        // set leftover using the left side pointer once the segment loop finished
        self_leftover = if let Some(leftover) = self_inner.get(self_left_index..) {
            leftover.to_string()
        } else {
            "".to_string()
        };
        other_leftover = if let Some(leftover) = other_inner.get(other_left_index..) {
            leftover.to_string()
        } else {
            "".to_string()
        };

        // this catches the case where all numeric and alpha segments have compared identically but the segment
        // separating characters were different
        if self_leftover.is_empty() && other_leftover.is_empty() {
            return Ordering::Equal;
        }

        // the final showdown. we never want a remaining alpha string to beat an empty string. the logic is a bit weird,
        // but:
        // - if one is empty and two is not an alpha, two is newer.
        // - if one is an alpha, two is newer.
        // - otherwise one is newer.
        if (self_leftover.is_empty() && !other_leftover.chars().next().unwrap().is_alphabetic())
            || (!self_leftover.is_empty() && self_leftover.chars().next().unwrap().is_alphabetic())
        {
            return Ordering::Less;
        }

        Ordering::Greater
    }
}

impl PartialOrd for Pkgver {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Pkgver {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

/// A version of a package
///
/// A `Version` tracks an optional `Epoch`, a `Pkgver` and an optional `Pkgrel`.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
/// use alpm_types::{Epoch, Pkgrel, Pkgver, Version};
///
/// let version = Version::new("1:1-1").unwrap();
/// assert_eq!(version.epoch(), Some(&Epoch::new("1".to_string()).unwrap()));
/// assert_eq!(version.pkgver(), &Pkgver::new("1".to_string()).unwrap());
/// assert_eq!(version.pkgrel(), Some(&Pkgrel::new("1".to_string()).unwrap()));
/// ```
#[derive(Debug, Eq)]
pub struct Version {
    pkgver: Pkgver,
    epoch: Option<Epoch>,
    pkgrel: Option<Pkgrel>,
}

impl Version {
    /// Create a new Version from a string and return it in a Result
    pub fn new(version: &str) -> Result<Self, Error> {
        let mut epoch_split = vec![];
        let mut pkgrel_split = vec![];
        for (i, char) in version.chars().enumerate() {
            match char {
                ':' => epoch_split.push(i),
                '-' => pkgrel_split.push(i),
                _ => {}
            }
        }

        Ok(Version {
            pkgver: match (epoch_split.len(), pkgrel_split.len()) {
                // pkgrel occurs before epoch
                (1, 1) if epoch_split[0] > pkgrel_split[0] => {
                    return Err(Error::InvalidVersion(version.to_string()))
                }
                // pkgver in between epoch and pkgrel
                (1, 1) => Pkgver::new(version[epoch_split[0] + 1..pkgrel_split[0]].to_string())?,
                // pkgver before pkgrel
                (0, 1) => Pkgver::new(version[..pkgrel_split[0]].to_string())?,
                // only pkgver
                (0, 0) => Pkgver::new(version.to_string())?,
                // pkgver after epoch
                (1, 0) => Pkgver::new(version[epoch_split[0] + 1..].to_string())?,
                // more than one epoch or pkgrel
                (_, _) => return Err(Error::InvalidVersion(version.to_string())),
            },
            epoch: if epoch_split.len() == 1 {
                Some(Epoch::new(version[..epoch_split[0]].to_string())?)
            } else {
                None
            },
            pkgrel: if pkgrel_split.len() == 1 {
                Some(Pkgrel::new(version[pkgrel_split[0] + 1..].to_string())?)
            } else {
                None
            },
        })
    }

    /// Return the optional reference to the Epoch of the Version
    pub fn epoch(&self) -> Option<&Epoch> {
        self.epoch.as_ref()
    }

    /// Return a reference to Pkgver of the Version
    pub fn pkgver(&self) -> &Pkgver {
        &self.pkgver
    }

    /// Return the optional reference to the Pkgrel of the Version
    pub fn pkgrel(&self) -> Option<&Pkgrel> {
        self.pkgrel.as_ref()
    }

    /// Compare two Versions and return a number
    ///
    /// The comparison algorithm is based on libalpm/ pacman's vercmp behavior.
    ///
    /// * `1` if `a` is newer than `b`
    /// * `0` if `a` and `b` are considered to be the same version
    /// * `-1` if `a` is older than `b`
    ///
    /// ## Examples
    /// ```
    /// use alpm_types::Version;
    ///
    /// assert_eq!(Version::vercmp(&Version::new("1.0.0").unwrap(), &Version::new("0.1.0").unwrap()), 1);
    /// assert_eq!(Version::vercmp(&Version::new("1.0.0").unwrap(), &Version::new("1.0.0").unwrap()), 0);
    /// assert_eq!(Version::vercmp(&Version::new("0.1.0").unwrap(), &Version::new("1.0.0").unwrap()), -1);
    /// ```
    pub fn vercmp(a: &Version, b: &Version) -> i8 {
        match a.cmp(b) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }
}

impl FromStr for Version {
    type Err = Error;
    /// Create a SchemaVersion from a string
    fn from_str(input: &str) -> Result<Version, Self::Err> {
        Version::new(input)
    }
}

impl Display for Version {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{}{}{}",
            if let Some(epoch) = self.epoch() {
                format!("{}:", epoch)
            } else {
                "".to_string()
            },
            self.pkgver(),
            if let Some(pkgrel) = self.pkgrel() {
                format!("-{}", pkgrel)
            } else {
                "".to_string()
            }
        )
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.epoch, other.epoch) {
            (Some(self_epoch), Some(other_epoch)) if self_epoch.cmp(&other_epoch).is_ne() => {
                return self_epoch.cmp(&other_epoch)
            }
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
            (_, _) => {}
        }

        let pkgver_cmp = self.pkgver.cmp(&other.pkgver);
        if pkgver_cmp.is_ne() {
            return pkgver_cmp;
        }

        self.pkgrel.cmp(&other.pkgrel)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.epoch == other.epoch
            && self.pkgver.cmp(&other.pkgver).is_eq()
            && self.pkgrel == other.pkgrel
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
    #[case("1", Ok(BuildDate(1)))]
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
        let builddate = BuildDate(1);
        let datetime: BuildDate =
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(1, 0).unwrap(), Utc).into();
        assert_eq!(builddate, datetime);
    }

    #[rstest]
    #[case("/home", BuildDir::new("/home"))]
    #[case("./", Err(Error::InvalidBuildDir(String::from("./"))))]
    #[case("~/", Err(Error::InvalidBuildDir(String::from("~/"))))]
    #[case("foo.txt", Err(Error::InvalidBuildDir(String::from("foo.txt"))))]
    fn build_dir_from_string(#[case] from_str: &str, #[case] result: Result<BuildDir, Error>) {
        assert_eq!(BuildDir::from_str(from_str), result);
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
    #[case("1.0.0", Ok(SchemaVersion(SemverVersion::new(1, 0, 0))))]
    #[case("1", Ok(SchemaVersion(SemverVersion::new(1, 0, 0))))]
    #[case("-1.0.0", Err(Error::InvalidVersion("-1.0.0".to_string())))]
    fn schema_version(#[case] version: &str, #[case] result: Result<SchemaVersion, Error>) {
        assert_eq!(result, SchemaVersion::new(version))
    }

    #[rstest]
    #[case(
        SchemaVersion(SemverVersion::new(1, 0, 0)),
        SchemaVersion(SemverVersion::new(0, 1, 0))
    )]
    fn compare_schema_version(#[case] version_a: SchemaVersion, #[case] version_b: SchemaVersion) {
        assert!(version_a > version_b);
    }

    #[rstest]
    #[case("foo", Ok(Version{epoch: None, pkgver: Pkgver::new("foo".to_string()).unwrap(), pkgrel: None}))]
    #[case(
        "1:foo-1",
        Ok(Version{
            pkgver: Pkgver::new("foo".to_string()).unwrap(),
            epoch: Some(Epoch::new("1".to_string()).unwrap()),
            pkgrel: Some(Pkgrel::new("1".to_string()).unwrap()),
        }),
    )]
    #[case(
        "1:foo",
        Ok(Version{
            pkgver: Pkgver::new("foo".to_string()).unwrap(),
            epoch: Some(Epoch::new("1".to_string()).unwrap()),
            pkgrel: None,
        }),
    )]
    #[case(
        "foo-1",
        Ok(Version{
            pkgver: Pkgver::new("foo".to_string()).unwrap(),
            epoch: None,
            pkgrel: Some(Pkgrel::new("1".to_string()).unwrap())
        })
    )]
    #[case("-1foo:1", Err(Error::InvalidVersion("-1foo:1".to_string())))]
    #[case("1-foo:1", Err(Error::InvalidVersion("1-foo:1".to_string())))]
    #[case("1:1:foo-1", Err(Error::InvalidVersion("1:1:foo-1".to_string())))]
    #[case("1:foo-1-1", Err(Error::InvalidVersion("1:foo-1-1".to_string())))]
    #[case("", Err(Error::InvalidPkgver("".to_string())))]
    #[case(":", Err(Error::InvalidPkgver("".to_string())))]
    #[case(".", Err(Error::InvalidPkgver(".".to_string())))]
    fn version_from_string(#[case] version: &str, #[case] result: Result<Version, Error>) {
        if result.is_ok() {
            assert_eq!(result.as_ref().unwrap(), &Version::new(version).unwrap())
        } else {
            assert_eq!(
                result.as_ref().expect_err("Should be an Err"),
                &Version::new(version).expect_err("Should be an Err")
            )
        }
    }

    #[rstest]
    #[case("1".to_string(), Ok(Epoch(NonZeroUsize::new(1).unwrap())))]
    #[case("0".to_string(), Err(Error::InvalidEpoch("0".to_string())))]
    #[case("-0".to_string(), Err(Error::InvalidEpoch("-0".to_string())))]
    #[case("z".to_string(), Err(Error::InvalidEpoch("z".to_string())))]
    fn epoch(#[case] version: String, #[case] result: Result<Epoch, Error>) {
        assert_eq!(result, Epoch::new(version));
    }

    #[rstest]
    #[case("foo".to_string(), Ok(Pkgver::new("foo".to_string()).unwrap()))]
    #[case("1.0.0".to_string(), Ok(Pkgver::new("1.0.0".to_string()).unwrap()))]
    #[case("1:foo".to_string(), Err(Error::InvalidPkgver("1:foo".to_string())))]
    #[case("foo-1".to_string(), Err(Error::InvalidPkgver("foo-1".to_string())))]
    #[case("foo,1".to_string(), Err(Error::InvalidPkgver("foo,1".to_string())))]
    #[case(".foo".to_string(), Err(Error::InvalidPkgver(".foo".to_string())))]
    #[case("_foo".to_string(), Err(Error::InvalidPkgver("_foo".to_string())))]
    fn pkgver(#[case] version: String, #[case] result: Result<Pkgver, Error>) {
        assert_eq!(result, Pkgver::new(version));
    }

    #[rstest]
    #[case("1".to_string(), Ok(Pkgrel::new("1".to_string()).unwrap()))]
    #[case("1.1".to_string(), Ok(Pkgrel::new("1.1".to_string()).unwrap()))]
    #[case("0.1".to_string(), Err(Error::InvalidPkgrel("0.1".to_string())))]
    #[case("0".to_string(), Err(Error::InvalidPkgrel("0".to_string())))]
    fn pkgrel(#[case] version: String, #[case] result: Result<Pkgrel, Error>) {
        assert_eq!(result, Pkgrel::new(version));
    }

    #[rstest]
    #[case(Pkgrel::new("1".to_string()).unwrap(), Pkgrel::new("2".to_string()).unwrap())]
    #[case(Pkgrel::new("1".to_string()).unwrap(), Pkgrel::new("1.1".to_string()).unwrap())]
    #[case(Pkgrel::new("1".to_string()).unwrap(), Pkgrel::new("11".to_string()).unwrap())]
    fn pkgrel_cmp(#[case] pkgrel_a: Pkgrel, #[case] pkgrel_b: Pkgrel) {
        assert!(pkgrel_a.lt(&pkgrel_b));
    }

    #[rstest]
    #[case(Version::new("1:1-1").unwrap(), "1:1-1")]
    #[case(Version::new("1-1").unwrap(), "1-1")]
    #[case(Version::new("1").unwrap(), "1")]
    #[case(Version::new("1:1").unwrap(), "1:1")]
    fn version_to_string(#[case] version: Version, #[case] to_str: &str) {
        assert_eq!(format!("{}", version), to_str);
    }

    #[rstest]
    #[case(Version::new("1").unwrap(), Version::new("1").unwrap(), Ordering::Equal, 0)]
    #[case(Version::new("2").unwrap(), Version::new("1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1").unwrap(), Version::new("2").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1").unwrap(), Version::new("1.1").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1.1").unwrap(), Ordering::Equal, 0)]
    #[case(Version::new("1.2").unwrap(), Version::new("1.1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1.2").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1+2").unwrap(), Version::new("1+1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1+1").unwrap(), Version::new("1+2").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1.1a").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.1a").unwrap(), Version::new("1.1").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1.1a1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.1a1").unwrap(), Version::new("1.1").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1.11a").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.11a").unwrap(), Version::new("1.1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.1_a").unwrap(), Version::new("1.1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1.1_a").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1.1.a").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1.a").unwrap(), Version::new("1.1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.a").unwrap(), Version::new("1.1").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1.a").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.a1").unwrap(), Version::new("1.1").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1.a1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.a11").unwrap(), Version::new("1.1").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1").unwrap(), Version::new("1.a11").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("a.1").unwrap(), Version::new("1.1").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1").unwrap(), Version::new("a.1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("foo").unwrap(), Version::new("1.1").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1.1").unwrap(), Version::new("foo").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("a1a").unwrap(), Version::new("a1b").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("a1b").unwrap(), Version::new("a1a").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("20220102").unwrap(), Version::new("20220202").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("20220202").unwrap(), Version::new("20220102").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.0..").unwrap(), Version::new("1.0.").unwrap(), Ordering::Equal, 0)]
    #[case(Version::new("1.0.").unwrap(), Version::new("1.0").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1..0").unwrap(), Version::new("1.0").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1..0").unwrap(), Version::new("1..0").unwrap(), Ordering::Equal, 0)]
    #[case(Version::new("1..1").unwrap(), Version::new("1..0").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1..0").unwrap(), Version::new("1..1").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("1+0").unwrap(), Version::new("1.0").unwrap(), Ordering::Equal, 0)]
    #[case(Version::new("1.111").unwrap(), Version::new("1.1a1").unwrap(), Ordering::Greater, 1)]
    #[case(Version::new("1.1a1").unwrap(), Version::new("1.111").unwrap(), Ordering::Less, -1)]
    #[case(Version::new("01").unwrap(), Version::new("1").unwrap(), Ordering::Equal, 0)]
    #[case(Version::new("001a").unwrap(), Version::new("1a").unwrap(), Ordering::Equal, 0)]
    #[case(Version::new("1.a001a.1").unwrap(), Version::new("1.a1a.1").unwrap(), Ordering::Equal, 0)]
    fn version_cmp(
        #[case] version_a: Version,
        #[case] version_b: Version,
        #[case] ordering: Ordering,
        #[case] vercmp_result: i8,
    ) {
        assert_eq!(version_a.cmp(&version_b), ordering);
        assert_eq!(Version::vercmp(&version_a, &version_b), vercmp_result);
    }
}
