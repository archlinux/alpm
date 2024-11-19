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
/// let packager = Packager::from_str("Foobar McFooface <foobar@mcfooface.org>").unwrap();
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Packager {
    name: String,
    email: EmailAddress,
}

impl Packager {
    /// Create a new Packager
    pub fn new(name: String, email: EmailAddress) -> Packager {
        Packager { name, email }
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
    fn from_str(s: &str) -> Result<Packager, Self::Err> {
        if let Some(captures) = PACKAGER_REGEX.captures(s) {
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
                value: s.to_string(),
                regex_type: "packager".to_string(),
                regex: PACKAGER_REGEX.to_string(),
            })
        }
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
        Packager{
            name: "Foobar McFooface (The Third)".to_string(),
            email: EmailAddress::from_str("foobar@mcfooface.org").unwrap()
        }
    )]
    #[case(
        "Foobar McFooface <foobar@mcfooface.org>",
        Packager{
            name: "Foobar McFooface".to_string(),
            email: EmailAddress::from_str("foobar@mcfooface.org").unwrap()
        }
    )]
    fn valid_packager(#[case] from_str: &str, #[case] packager: Packager) {
        assert_eq!(Packager::from_str(from_str), Ok(packager));
    }

    /// Test that invalid packager email expressions throw the expected email errors.
    #[rstest]
    #[case(
        "Foobar McFooface <@mcfooface.org>",
        email_address::Error::LocalPartEmpty
    )]
    #[case(
        "Foobar McFooface <foobar@mcfooface.org> <foobar@mcfoofacemcfooface.org>",
        email_address::Error::MissingEndBracket
    )]
    fn invalid_packager_email(#[case] packager: &str, #[case] error: email_address::Error) {
        assert_eq!(Packager::from_str(packager), Err(error.into()));
    }

    /// Test that invalid packager expressionare detected as such throw the expected Regex error.
    #[rstest]
    #[case("<foobar@mcfooface.org>")]
    #[case("[foo] <foobar@mcfooface.org>")]
    #[case("foobar@mcfooface.org")]
    #[case("Foobar McFooface")]
    fn invalid_packager_regex(#[case] packager: &str) {
        assert_eq!(
            Packager::from_str(packager),
            Err(Error::RegexDoesNotMatch {
                value: packager.to_string(),
                regex_type: "packager".to_string(),
                regex: PACKAGER_REGEX.to_string(),
            })
        );
    }

    #[rstest]
    #[case(
        Packager::from_str("Foobar McFooface <foobar@mcfooface.org>").unwrap(),
        "Foobar McFooface <foobar@mcfooface.org>"
    )]
    fn packager_format_string(#[case] packager: Packager, #[case] packager_str: &str) {
        assert_eq!(packager_str, format!("{}", packager));
    }

    #[rstest]
    #[case(Packager::from_str("Foobar McFooface <foobar@mcfooface.org>").unwrap(), "Foobar McFooface")]
    fn packager_name(#[case] packager: Packager, #[case] name: &str) {
        assert_eq!(name, packager.name());
    }

    #[rstest]
    #[case(
        Packager::from_str("Foobar McFooface <foobar@mcfooface.org>").unwrap(),
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
