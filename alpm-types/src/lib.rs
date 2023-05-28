// SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
// SPDX-License-Identifier: LGPL-3.0-or-later
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;
use std::string::ToString;

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

mod pkg;
pub use pkg::Packager;
pub use pkg::PkgType;

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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

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
    #[case("something", Ok(PackageOption(BuildOption{name: "something".to_string(), on: true})))]
    #[case("!something", Ok(PackageOption(BuildOption{name: "something".to_string(), on: false})))]
    #[case("foo\\", Err(Error::InvalidPackageOption("foo\\".to_string())))]
    fn packageoption(#[case] from_str: &str, #[case] result: Result<PackageOption, Error>) {
        assert_eq!(PackageOption::from_str(from_str), result);
    }
}
