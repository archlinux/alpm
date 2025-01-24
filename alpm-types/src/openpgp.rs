use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    string::ToString,
};

use email_address::EmailAddress;
use lazy_regex::{lazy_regex, Lazy};
use regex::Regex;
use serde::Serialize;

use crate::Error;

/// An OpenPGP key identifier.
///
/// The `OpenPGPIdentifier` enum represents a valid OpenPGP identifier, which can be either an
/// OpenPGP Key ID or an OpenPGP v4 fingerprint.
///
/// This type wraps an [`OpenPGPKeyId`] and an [`OpenPGPv4Fingerprint`] and provides a unified
/// interface for both.
///
/// ## Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Error, OpenPGPIdentifier, OpenPGPKeyId, OpenPGPv4Fingerprint};
/// # fn main() -> Result<(), alpm_types::Error> {
/// // Create a OpenPGPIdentifier from a valid OpenPGP v4 fingerprint
/// let key = OpenPGPIdentifier::from_str("4A0C4DFFC02E1A7ED969ED231C2358A25A10D94E")?;
/// assert_eq!(
///     key,
///     OpenPGPIdentifier::OpenPGPv4Fingerprint(OpenPGPv4Fingerprint::from_str(
///         "4A0C4DFFC02E1A7ED969ED231C2358A25A10D94E"
///     )?)
/// );
/// assert_eq!(key.to_string(), "4A0C4DFFC02E1A7ED969ED231C2358A25A10D94E");
/// assert_eq!(
///     key,
///     OpenPGPv4Fingerprint::from_str("4A0C4DFFC02E1A7ED969ED231C2358A25A10D94E")?.into()
/// );
///
/// // Create a OpenPGPIdentifier from a valid OpenPGP Key ID
/// let key = OpenPGPIdentifier::from_str("2F2670AC164DB36F")?;
/// assert_eq!(
///     key,
///     OpenPGPIdentifier::OpenPGPKeyId(OpenPGPKeyId::from_str("2F2670AC164DB36F")?)
/// );
/// assert_eq!(key.to_string(), "2F2670AC164DB36F");
/// assert_eq!(key, OpenPGPKeyId::from_str("2F2670AC164DB36F")?.into());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum OpenPGPIdentifier {
    /// An OpenPGP Key ID.
    OpenPGPKeyId(OpenPGPKeyId),
    /// An OpenPGP v4 fingerprint.
    OpenPGPv4Fingerprint(OpenPGPv4Fingerprint),
}

impl FromStr for OpenPGPIdentifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<OpenPGPv4Fingerprint>() {
            Ok(fingerprint) => Ok(OpenPGPIdentifier::OpenPGPv4Fingerprint(fingerprint)),
            Err(_) => match s.parse::<OpenPGPKeyId>() {
                Ok(key_id) => Ok(OpenPGPIdentifier::OpenPGPKeyId(key_id)),
                Err(e) => Err(e),
            },
        }
    }
}

impl From<OpenPGPKeyId> for OpenPGPIdentifier {
    fn from(key_id: OpenPGPKeyId) -> Self {
        OpenPGPIdentifier::OpenPGPKeyId(key_id)
    }
}

impl From<OpenPGPv4Fingerprint> for OpenPGPIdentifier {
    fn from(fingerprint: OpenPGPv4Fingerprint) -> Self {
        OpenPGPIdentifier::OpenPGPv4Fingerprint(fingerprint)
    }
}

impl Display for OpenPGPIdentifier {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            OpenPGPIdentifier::OpenPGPKeyId(key_id) => write!(f, "{key_id}"),
            OpenPGPIdentifier::OpenPGPv4Fingerprint(fingerprint) => write!(f, "{fingerprint}"),
        }
    }
}

/// An OpenPGP Key ID.
///
/// The `OpenPGPKeyId` type wraps a `String` representing an [OpenPGP Key ID],
/// ensuring that it consists of exactly 16 uppercase hexadecimal characters.
///
/// [OpenPGP Key ID]: https://openpgp.dev/book/glossary.html#term-Key-ID
///
/// ## Note
///
/// - This type supports constructing from both uppercase and lowercase hexadecimal characters but
///   guarantees to return the key ID in uppercase.
///
/// - The usage of this type is highly discouraged as the keys may not be unique. This will lead to
///   a linting error in the future.
///
/// ## Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Error, OpenPGPKeyId};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // Create OpenPGPKeyId from a valid key ID
/// let key = OpenPGPKeyId::from_str("2F2670AC164DB36F")?;
/// assert_eq!(key.as_str(), "2F2670AC164DB36F");
///
/// // Attempting to create an OpenPGPKeyId from an invalid key ID will fail
/// assert!(OpenPGPKeyId::from_str("INVALIDKEYID").is_err());
///
/// // Format as String
/// assert_eq!(format!("{key}"), "2F2670AC164DB36F");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OpenPGPKeyId(String);

impl OpenPGPKeyId {
    /// Creates a new `OpenPGPKeyId` instance.
    ///
    /// See [`OpenPGPKeyId::from_str`] for more information on how the OpenPGP Key ID is validated.
    pub fn new(key_id: String) -> Result<Self, Error> {
        if key_id.len() == 16 && key_id.chars().all(|c| c.is_ascii_hexdigit()) {
            Ok(Self(key_id.to_ascii_uppercase()))
        } else {
            Err(Error::InvalidOpenPGPKeyId(key_id))
        }
    }

    /// Returns a reference to the inner OpenPGP Key ID as a `&str`.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the `OpenPGPKeyId` and returns the inner `String`.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl FromStr for OpenPGPKeyId {
    type Err = Error;

    /// Creates a new `OpenPGPKeyId` instance after validating that it follows the correct format.
    ///
    /// A valid OpenPGP Key ID should be exactly 16 characters long and consist only
    /// of digits (`0-9`) and hexadecimal letters (`A-F`).
    ///
    /// # Errors
    ///
    /// Returns an error if the OpenPGP Key ID is not valid.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl Display for OpenPGPKeyId {
    /// Converts the `OpenPGPKeyId` to an uppercase `String`.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
    /// # Errors
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
/// # fn main() -> Result<(), alpm_types::Error> {
/// // create Packager from &str
/// let packager = Packager::from_str("Foobar McFooface <foobar@mcfooface.org>")?;
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
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
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
    #[case("2F2670AC164DB36F")]
    #[case("584A3EBFE705CDCD")]
    fn test_parse_openpgp_key_id(#[case] input: &str) -> Result<(), Error> {
        input.parse::<OpenPGPKeyId>()?;
        Ok(())
    }

    #[rstest]
    // Contains non-hex characters 'G' and 'H'
    #[case("1234567890ABCGH", Err(Error::InvalidOpenPGPKeyId("1234567890ABCGH".to_string())))]
    // Less than 16 characters
    #[case("1234567890ABCDE", Err(Error::InvalidOpenPGPKeyId("1234567890ABCDE".to_string())))]
    // More than 16 characters
    #[case("1234567890ABCDEF0", Err(Error::InvalidOpenPGPKeyId("1234567890ABCDEF0".to_string())))]
    // Just invalid
    #[case("invalid", Err(Error::InvalidOpenPGPKeyId("invalid".to_string())))]
    fn test_parse_invalid_openpgp_key_id(
        #[case] input: &str,
        #[case] expected: Result<OpenPGPKeyId, Error>,
    ) {
        let result = input.parse::<OpenPGPKeyId>();
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
}
