use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    string::ToString,
};

use email_address::EmailAddress;
use lazy_regex::{lazy_regex, Lazy};
use regex::Regex;
use strum::{Display, EnumString};

use crate::error::Error;

pub(crate) static PACKAGER_REGEX: Lazy<Regex> =
    lazy_regex!(r"^(?P<name>[\w\s\-().]+) <(?P<email>.*)>$");

/// A packager of a package
///
/// A `Packager` is represented by a User ID (e.g. `"Foobar McFooFace <foobar@mcfooface.org>"`).
/// Internally this struct wraps a `String` for the name and an `EmailAddress` for a valid email
/// address.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Error, Packager};
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
/// assert_eq!(
///     "Foobar McFooface <foobar@mcfooface.org>",
///     format!("{}", packager)
/// );
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Packager {
    name: String,
    email: EmailAddress,
}

impl Packager {
    /// Create a new Packager from a string
    pub fn new(packager: &str) -> Result<Packager, Error> {
        if let Some(captures) = PACKAGER_REGEX.captures(packager) {
            if captures.name("name").is_some() && captures.name("email").is_some() {
                let email = EmailAddress::from_str(captures.name("email").unwrap().as_str())?;
                Ok(Packager {
                    name: captures.name("name").unwrap().as_str().to_string(),
                    email,
                })
            } else {
                Err(Error::MissingComponent {
                    component: if captures.name("name").is_none() {
                        "name"
                    } else {
                        "email"
                    },
                })
            }
        } else {
            Err(Error::RegexDoesNotMatch {
                regex: PACKAGER_REGEX.to_string(),
            })
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

/// The type of a package
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
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
#[derive(Clone, Debug, Display, EnumString, PartialEq)]
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
    use rstest::rstest;

    use super::*;

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
        Err(email_address::Error::LocalPartEmpty.into()),
    )]
    #[case(
        "Foobar McFooface <foobar@mcfooface.org> <foobar@mcfoofacemcfooface.org>",
        Err(email_address::Error::MissingEndBracket.into()),
    )]
    #[case(
        "<foobar@mcfooface.org>",
        Err(Error::RegexDoesNotMatch {
            regex: PACKAGER_REGEX.to_string(),
        })
    )]
    #[case(
        "[foo] <foobar@mcfooface.org>",
        Err(Error::RegexDoesNotMatch {
            regex: PACKAGER_REGEX.to_string(),
        })
    )]
    #[case(
        "foobar@mcfooface.org",
        Err(Error::RegexDoesNotMatch {
            regex: PACKAGER_REGEX.to_string(),
        })
    )]
    #[case(
        "Foobar McFooface",
        Err(Error::RegexDoesNotMatch {
            regex: PACKAGER_REGEX.to_string(),
        })
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
    #[case("debug", Ok(PkgType::Debug))]
    #[case("pkg", Ok(PkgType::Package))]
    #[case("src", Ok(PkgType::Source))]
    #[case("split", Ok(PkgType::Split))]
    #[case("foo", Err(strum::ParseError::VariantNotFound))]
    fn pkgtype_from_string(
        #[case] from_str: &str,
        #[case] result: Result<PkgType, strum::ParseError>,
    ) {
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
