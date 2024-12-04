use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    string::ToString,
};

use email_address::EmailAddress;
use winnow::{
    ascii::space0,
    combinator::{delimited, seq},
    error::{StrContext, StrContextValue},
    prelude::*,
    token::{take_till, take_while},
};

use crate::Error;

/// An OpenPGP v4 fingerprint.
///
/// The `OpenPGPv4Fingerprint` type wraps a `String` representing an [OpenPGP v4 fingerprint],
/// ensuring that it consists of exactly 40 uppercase hexadecimal characters.
///
/// [OpenPGP v4 fingerprint]: https://openpgp.dev/book/certificates.html#fingerprint
///
/// ## Note
///
/// This type supports constructing from both uppercase and lowercase hexadecimal characters but
/// guarantees to return the fingerprint in uppercase.
///
/// ## Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Error, OpenPGPv4Fingerprint};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // Create OpenPGPv4Fingerprint from a valid OpenPGP v4 fingerprint
/// let key = OpenPGPv4Fingerprint::from_str("4A0C4DFFC02E1A7ED969ED231C2358A25A10D94E")?;
/// assert_eq!(key.as_str(), "4A0C4DFFC02E1A7ED969ED231C2358A25A10D94E");
///
/// // Attempting to create a OpenPGPv4Fingerprint from an invalid fingerprint will fail
/// assert!(OpenPGPv4Fingerprint::from_str("INVALIDKEY").is_err());
///
/// // Format as String
/// assert_eq!(
///     format!("{}", key),
///     "4A0C4DFFC02E1A7ED969ED231C2358A25A10D94E"
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenPGPv4Fingerprint(String);

impl OpenPGPv4Fingerprint {
    /// Creates a new `OpenPGPv4Fingerprint` instance
    ///
    /// See [`OpenPGPv4Fingerprint::from_str`] for more information on how the OpenPGP v4
    /// fingerprint is validated.
    pub fn new(fingerprint: String) -> Result<Self, Error> {
        Self::from_str(&fingerprint)
    }

    /// Returns a reference to the inner OpenPGP v4 fingerprint as a `&str`.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the `OpenPGPv4Fingerprint` and returns the inner `String`.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl FromStr for OpenPGPv4Fingerprint {
    type Err = Error;

    /// Creates a new `OpenPGPv4Fingerprint` instance after validating that it follows the correct
    /// format.
    ///
    /// A valid OpenPGP v4 fingerprint should be exactly 40 characters long and consist only
    /// of digits (`0-9`) and hexadecimal letters (`A-F`).
    ///
    /// ## Errors
    ///
    /// Returns an error if the OpenPGP v4 fingerprint is not valid.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit()) {
            Ok(Self(s.to_ascii_uppercase()))
        } else {
            Err(Error::InvalidOpenPGPv4Fingerprint)
        }
    }
}

impl Display for OpenPGPv4Fingerprint {
    /// Converts the `OpenPGPv4Fingerprint` to a uppercase `String`.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str().to_ascii_uppercase())
    }
}

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

    /// Parse the name of a Packager
    fn parse_name(input: &mut &str) -> PResult<String> {
        take_till(0.., '<')
            .verify(|s: &str| !s.is_empty())
            .verify(|s: &str| {
                s.chars()
                    .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '(' | ')'))
            })
            .context(StrContext::Label("name"))
            .context(StrContext::Expected(StrContextValue::Description(
                "an alphanumeric packager name before '<'",
            )))
            .parse_next(input)
            .map(|v| v.trim().to_string())
    }

    /// Parse the email of a Packager
    fn parse_email(input: &mut &str) -> PResult<EmailAddress> {
        delimited('<', take_while(0.., |c| c != '>'), '>')
            .parse_to()
            .context(StrContext::Label("email"))
            .context(StrContext::Expected(StrContextValue::Description(
                "a valid email address between '<' and '>'",
            )))
            .parse_next(input)
    }

    /// Parse a Packager from a string
    fn parse(input: &mut &str) -> PResult<Packager> {
        seq!(Packager {
            name: Self::parse_name,
            _: space0,
            email: Self::parse_email,
        })
        .parse_next(input)
    }
}

impl FromStr for Packager {
    type Err = Error;
    /// Create a Packager from a string
    fn from_str(s: &str) -> Result<Packager, Self::Err> {
        Ok(Self::parse.parse(s)?)
    }
}

impl Display for Packager {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{} <{}>", self.name, self.email)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("4A0C4DFFC02E1A7ED969ED231C2358A25A10D94E")]
    #[case("1234567890abcdef1234567890abcdef12345678")]
    fn test_parse_openpgp_fingerprint(#[case] input: &str) -> Result<(), Error> {
        input.parse::<OpenPGPv4Fingerprint>()?;
        Ok(())
    }

    #[rstest]
    // Contains non-hex characters 'G' and 'H'
    #[case(
        "A1B2C3D4E5F6A7B8C9D0E1F2A3B4C5D6E7F8G9H0",
        Err(Error::InvalidOpenPGPv4Fingerprint)
    )]
    // Less than 40 characters
    #[case(
        "1234567890ABCDEF1234567890ABCDEF1234567",
        Err(Error::InvalidOpenPGPv4Fingerprint)
    )]
    // More than 40 characters
    #[case(
        "1234567890ABCDEF1234567890ABCDEF1234567890",
        Err(Error::InvalidOpenPGPv4Fingerprint)
    )]
    // Just invalid
    #[case("invalid", Err(Error::InvalidOpenPGPv4Fingerprint))]
    fn test_parse_invalid_openpgp_fingerprint(
        #[case] input: &str,
        #[case] expected: Result<OpenPGPv4Fingerprint, Error>,
    ) {
        let result = input.parse::<OpenPGPv4Fingerprint>();
        assert_eq!(result, expected);
    }

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

    #[rstest]
    #[case(
        "Foobar McFooface <@mcfooface.org>",
        "                 ^
invalid email
expected a valid email address between '<' and '>'"
    )]
    #[case(
        "Foobar McFooface <foobar@mcfooface.org> <foobar@mcfoofacemcfooface.org>",
        "                                       ^"
    )]
    #[case(
        "<foobar@mcfooface.org>",
        "^
invalid name
expected an alphanumeric packager name before '<'"
    )]
    #[case(
        "[foo] <foobar@mcfooface.org>",
        "^
invalid name
expected an alphanumeric packager name before '<'"
    )]
    #[case(
        "foobar@mcfooface.org",
        "^
invalid name
expected an alphanumeric packager name before '<'"
    )]
    #[case(
        "Foobar McFooface",
        "                ^
invalid email
expected a valid email address between '<' and '>'"
    )]
    fn invalid_packager_email(#[case] packager: &str, #[case] error: &str) {
        assert_eq!(
            Packager::from_str(packager).map_err(|e| e
                .to_string()
                .lines()
                .skip(1)
                .collect::<Vec<_>>()
                .join("\n")),
            Err(error.into())
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
}
