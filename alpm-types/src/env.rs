use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    string::ToString,
};

use serde::Serialize;

use crate::error::Error;
use crate::Architecture;
use crate::Name;
use crate::Version;

/// An option string
///
/// The option string is identified by its name and whether it is on (not prefixed with "!") or off
/// (prefixed with "!").
///
/// This type is used in the context of `makepkg` options, build environment options ([`BuildEnv`]),
/// and package options ([`PackageOption`]).
///
/// See [the makepkg.conf manpage](https://man.archlinux.org/man/makepkg.conf.5.en) for more information.
///
/// ## Examples
/// ```
/// # fn main() -> Result<(), alpm_types::Error> {
/// use alpm_types::MakepkgOption;
///
/// let option = MakepkgOption::new("foo")?;
/// assert_eq!(option.on(), true);
/// assert_eq!(option.name(), "foo");
///
/// let not_option = MakepkgOption::new("!foo")?;
/// assert_eq!(not_option.on(), false);
/// assert_eq!(not_option.name(), "foo");
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MakepkgOption {
    name: String,
    on: bool,
}

impl MakepkgOption {
    /// Create a new MakepkgOption in a Result
    pub fn new(option: &str) -> Result<Self, Error> {
        Self::from_str(option)
    }

    /// Get the name of the MakepkgOption
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get whether the MakepkgOption is on
    pub fn on(&self) -> bool {
        self.on
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
        Ok(MakepkgOption { name, on })
    }
}

impl Display for MakepkgOption {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}{}", if self.on { "" } else { "!" }, self.name)
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
/// use alpm_types::BuildEnv;
///
/// let option = BuildEnv::new("foo")?;
/// assert_eq!(option.on(), true);
/// assert_eq!(option.name(), "foo");
///
/// let not_option = BuildEnv::new("!foo")?;
/// assert_eq!(not_option.on(), false);
/// assert_eq!(not_option.name(), "foo");
/// # Ok(())
/// # }
/// ```
pub type BuildEnv = MakepkgOption;

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
/// let option = PackageOption::new("foo")?;
/// assert_eq!(option.on(), true);
/// assert_eq!(option.name(), "foo");
///
/// let not_option = PackageOption::new("!foo")?;
/// assert_eq!(not_option.on(), false);
/// assert_eq!(not_option.name(), "foo");
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
    #[case("something", Ok(MakepkgOption{name: "something".to_string(), on: true}))]
    #[case("1cool.build-option", Ok(MakepkgOption{name: "1cool.build-option".to_string(), on: true}))]
    #[case("üñıçøĐë", Ok(MakepkgOption{name: "üñıçøĐë".to_string(), on: true}))]
    #[case("!üñıçøĐë", Ok(MakepkgOption{name: "üñıçøĐë".to_string(), on: false}))]
    #[case("!something", Ok(MakepkgOption{name: "something".to_string(), on: false}))]
    #[case("!!something", Err(Error::ValueContainsInvalidChars { invalid_char: '!'}))]
    #[case("foo\\", Err(Error::ValueContainsInvalidChars { invalid_char: '\\'}))]
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
    #[case("foo-bar-1:1.0.0-any", Err(Error::InvalidInteger{ kind: std::num::IntErrorKind::InvalidDigit}))]
    #[case("1:1.0.0-1-any", Err(Error::MissingComponent { component: "name" }))]
    fn installed_new(#[case] s: &str, #[case] result: Result<InstalledPackage, Error>) {
        assert_eq!(InstalledPackage::from_str(s), result);
    }
}
