use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    string::ToString,
};

use alpm_parsers::prelude::*;
use base64::{Engine, prelude::BASE64_STANDARD};
use email_address::EmailAddress;
use fluent_i18n::t;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use serde_with::DeserializeFromStr;
use winnow::{
    ascii::space0,
    combinator::{alt, not, peek, repeat_till},
    error::ErrMode,
    token::any,
};

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
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum OpenPGPIdentifier {
    /// An OpenPGP Key ID.
    #[cfg_attr(feature = "serde", serde(rename = "openpgp_key_id"))]
    OpenPGPKeyId(OpenPGPKeyId),
    /// An OpenPGP v4 fingerprint.
    #[cfg_attr(feature = "serde", serde(rename = "openpgp_v4_fingerprint"))]
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
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(DeserializeFromStr, Serialize))]
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
/// ensuring that it consists of 40 uppercase hexadecimal characters with optional whitespace
/// separators.
///
/// [OpenPGP v4 fingerprint]: https://openpgp.dev/book/certificates.html#fingerprint
///
/// ## Note
///
/// - This type supports constructing from both uppercase and lowercase hexadecimal characters, with
///   and without whitespace separators, but guarantees to return the fingerprint in uppercase and
///   with no whitespaces.
///
/// - Whitespaces are only allowed between hexadecimal characters, not at the start or end of the
///   fingerprint.
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
/// // Space separated fingerprint is also valid
/// let key = OpenPGPv4Fingerprint::from_str("4A0C 4DFF C02E 1A7E D969 ED23 1C23 58A2 5A10 D94E")?;
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
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(DeserializeFromStr, Serialize))]
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
    /// A valid OpenPGP v4 fingerprint should be a 40 characters long string of digits (`0-9`)
    /// and hexadecimal letters (`A-F`) optionally separated by whitespaces.
    ///
    /// # Errors
    ///
    /// Returns an error if the OpenPGP v4 fingerprint is not valid.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.to_ascii_uppercase().replace(" ", "");

        if !s.starts_with(' ')
            && !s.ends_with(' ')
            && normalized.len() == 40
            && normalized.chars().all(|c| c.is_ascii_hexdigit())
        {
            Ok(Self(normalized))
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

/// A base64 encoded OpenPGP detached signature.
///
/// Wraps a [`String`] representing a [base64] encoded [OpenPGP detached signature]
/// ensuring it consists of valid [base64] characters.
///
/// ## Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Error, Base64OpenPGPSignature};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // Create Base64OpenPGPSignature from a valid base64 String
/// let sig = Base64OpenPGPSignature::from_str("iHUEABYKAB0WIQRizHP4hOUpV7L92IObeih9mi7GCAUCaBZuVAAKCRCbeih9mi7GCIlMAP9ws/jU4f580ZRQlTQKvUiLbAZOdcB7mQQj83hD1Nc/GwD/WIHhO1/OQkpMERejUrLo3AgVmY3b4/uGhx9XufWEbgE=")?;
///
/// // Attempting to create a Base64OpenPGPSignature from an invalid base64 String will fail
/// assert!(Base64OpenPGPSignature::from_str("!@#$^&*").is_err());
///
/// // Format as String
/// assert_eq!(
///     format!("{}", sig),
///     "iHUEABYKAB0WIQRizHP4hOUpV7L92IObeih9mi7GCAUCaBZuVAAKCRCbeih9mi7GCIlMAP9ws/jU4f580ZRQlTQKvUiLbAZOdcB7mQQj83hD1Nc/GwD/WIHhO1/OQkpMERejUrLo3AgVmY3b4/uGhx9XufWEbgE="
/// );
/// # Ok(())
/// # }
/// ```
///
/// [base64]: https://en.wikipedia.org/wiki/Base64
/// [OpenPGP detached signature]: https://openpgp.dev/book/signing_data.html#detached-signatures
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(DeserializeFromStr, Serialize))]
pub struct Base64OpenPGPSignature(String);

impl Base64OpenPGPSignature {
    /// Creates a new [`Base64OpenPGPSignature`] instance.
    ///
    /// See [`Base64OpenPGPSignature::from_str`] for more information on how the OpenPGP signature
    /// is validated.
    pub fn new(signature: String) -> Result<Self, Error> {
        Self::from_str(&signature)
    }

    /// Returns a reference to the inner OpenPGP signature as a `&str`.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the [`Base64OpenPGPSignature`] and returns the inner [`String`].
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for Base64OpenPGPSignature {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for Base64OpenPGPSignature {
    type Err = Error;

    /// Creates a new [`Base64OpenPGPSignature`] instance after validating that it follows the
    /// correct format.
    ///
    /// A valid [OpenPGP signature] should consist only of [base64] characters (A-Z, a-z, 0-9, +, /)
    /// and may include padding characters (=) at the end.
    ///
    /// # Errors
    ///
    /// Returns an error if the OpenPGP signature is not valid.
    ///
    /// [base64]: https://en.wikipedia.org/wiki/Base64
    /// [OpenPGP signature]: https://openpgp.dev/book/signing_data.html#detached-signatures
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BASE64_STANDARD
            .decode(s)
            .map_err(|_| Error::InvalidBase64Encoding {
                expected_item: t!("error-invalid-base64-encoding-pgp-signature"),
            })?
            .to_vec();
        Ok(Self(s.to_string()))
    }
}

impl Display for Base64OpenPGPSignature {
    /// Converts the [`Base64OpenPGPSignature`] to a [`String`].
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
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

impl ParserUntil for Packager {
    /// Parses a [`Packager`] from a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` does not represent a valid [`Packager`].
    fn parser_until<'a, P>(delimiter: P) -> impl Parser<Input<'a>, Self, ErrMode<ParseStack<'a>>>
    where
        P: Parser<Input<'a>, &'a str, ErrMode<ParseStack<'a>>>,
    {
        // Define the actual parser closure.
        // The delimiter is moved into the closure and borrowed via `by_ref()` on each call.
        let mut delimiter_parser = delimiter;
        let parser = move |input: &mut Input<'a>| -> PResult<'a, Self> {
            // This format is a bit opaque and may include leading whitespaces.
            // Consume any of those so we don't have to guess about them later on.
            space0.parse_next(input)?;

            // Make sure the first character isn't a `<`, which may happen if the packager name is
            // missing.
            not("<")
                .expected_text("a packager name")
                .parse_next(input)?;

            // The name that precedes the email address
            let name = repeat_till::<_, _, (), _, _, _, _>(
                1..,
                any,
                peek(alt(("<", delimiter_parser.by_ref()))),
            )
            .take()
            .map(|name: &str| name.trim().to_string())
            .layer("packager name")
            .parse_next(input)?;

            // The '<' delimiter that marks the start of the email string
            '<'.expected_text("opening delimiter '<'")
                .layer("Email address")
                .parse_next(input)?;

            // The email address, which is validated by the EmailAddress struct.
            let email = repeat_till::<_, _, (), _, _, _, _>(
                1..,
                any,
                peek(alt((">", delimiter_parser.by_ref()))),
            )
            .take()
            .try_map(EmailAddress::from_str)
            .layer("email address")
            .parse_next(input)?;

            // The '>' delimiter that marks the end of the email string
            '>'.expected_text("closing delimiter '>' of packager email address")
                .layer("Email address")
                .parse_next(input)?;

            peek(delimiter_parser.by_ref())
                .label("packager: unexpected trailing content")
                .expected_text("end of input.")
                .parse_next(input)?;

            Ok(Self { name, email })
        };

        parser.layer("alpm packager")
    }
}

impl FromStr for Packager {
    type Err = Error;
    /// Creates a [`Packager`] from a string slice.
    ///
    /// Delegates to [`Packager::parser_until`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`Packager::parser_until`] fails.
    fn from_str(s: &str) -> Result<Packager, Self::Err> {
        Ok(Self::parser_until_eof.parse(Input::new(s))?)
    }
}

impl Display for Packager {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{} <{}>", self.name, self.email)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use rstest::rstest;
    use testresult::TestResult;

    use super::*;
    use crate::configure_insta;

    #[rstest]
    #[case("4A0C4DFFC02E1A7ED969ED231C2358A25A10D94E")]
    #[case("4A0C 4DFF C02E 1A7E D969 ED23 1C23 58A2 5A10 D94E")]
    #[case("1234567890abcdef1234567890abcdef12345678")]
    #[case("1234 5678 90ab cdef 1234 5678 90ab cdef 1234 5678")]
    fn test_parse_openpgp_fingerprint(#[case] input: &str) -> Result<(), Error> {
        input.parse::<OpenPGPv4Fingerprint>()?;
        Ok(())
    }

    #[rstest]
    // Contains non-hex characters 'G' and 'H'
    #[case("A1B2C3D4E5F6A7B8C9D0E1F2A3B4C5D6E7F8G9H0")]
    // Less than 40 characters
    #[case("1234567890ABCDEF1234567890ABCDEF1234567")]
    // More than 40 characters
    #[case("1234567890ABCDEF1234567890ABCDEF1234567890")]
    // Starts with whitespace
    #[case(" 4A0C 4DFF C02E 1A7E D969 ED23 1C23 58A2 5A10 D94E")]
    // Ends with whitespace
    #[case("4A0C 4DFF C02E 1A7E D969 ED23 1C23 58A2 5A10 D94E ")]
    // Just invalid
    #[case("invalid")]
    fn openpgpv4_fingerprint_from_str_fails_on_invalid_input(#[case] input: &str) -> TestResult {
        let Err(Error::InvalidOpenPGPv4Fingerprint) = OpenPGPv4Fingerprint::from_str(input) else {
            panic!("Expected to fail with Error::InvalidOpenPGPv4Fingerprint");
        };

        Ok(())
    }

    /// Make sure that invalid OpenPGP v4 fingerprints don't deserialize.
    #[cfg(feature = "serde")]
    #[rstest]
    #[case("A1B2C3D4E5F6A7B8C9D0E1F2A3B4C5D6E7F8G9H0")]
    #[case("invalid")]
    fn openpgp_fingerprint_deserialize_error(#[case] input: &str) {
        let Err(serde_json::Error { .. }) =
            serde_json::from_str::<OpenPGPv4Fingerprint>(&format!("\"{input}\""))
        else {
            panic!("'{input}' erroneously deserialized as an OpenPGPv4Fingerprint")
        };
    }

    #[rstest]
    #[case("2F2670AC164DB36F")]
    #[case("584A3EBFE705CDCD")]
    fn test_parse_openpgp_key_id(#[case] input: &str) -> Result<(), Error> {
        input.parse::<OpenPGPKeyId>()?;
        Ok(())
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serialize_openpgp_key_id() -> TestResult {
        let id = "584A3EBFE705CDCD".parse::<OpenPGPKeyId>()?;
        let json = serde_json::to_string(&OpenPGPIdentifier::OpenPGPKeyId(id))?;
        assert_eq!(r#"{"openpgp_key_id":"584A3EBFE705CDCD"}"#, json);

        Ok(())
    }

    #[cfg(feature = "serde")]
    #[rstest]
    #[case(
        "1234567890abcdef1234567890abcdef12345678",
        "1234567890ABCDEF1234567890ABCDEF12345678"
    )]
    #[case(
        "1234 5678 90ab cdef 1234 5678 90ab cdef 1234 5678",
        "1234567890ABCDEF1234567890ABCDEF12345678"
    )]
    fn test_serialize_openpgp_v4_fingerprint(
        #[case] input: &str,
        #[case] output: &str,
    ) -> TestResult {
        let print = input.parse::<OpenPGPv4Fingerprint>()?;
        let json = serde_json::to_string(&OpenPGPIdentifier::OpenPGPv4Fingerprint(print))?;
        assert_eq!(format!("{{\"openpgp_v4_fingerprint\":\"{output}\"}}"), json);

        Ok(())
    }

    /// Ensures, that [`OpenPGPKeyId::from_str`] fails on invalid input.
    #[rstest]
    // Contains non-hex characters 'G' and 'H'
    #[case("1234567890ABCGH")]
    // Less than 16 characters
    #[case("1234567890ABCDE")]
    // More than 16 characters
    #[case("1234567890ABCDEF0")]
    // Just invalid
    #[case("invalid")]
    fn test_parse_invalid_openpgp_key_id(#[case] input: &str) {
        let Err(Error::InvalidOpenPGPKeyId(invalid_input)) = OpenPGPKeyId::from_str(input) else {
            panic!("Expected to fail with Error::InvalidOpenPGPKeyId");
        };

        assert_eq!(invalid_input, input);
    }

    /// Make sure that invalid OpenPGP key IDs don't deserialize.
    #[cfg(feature = "serde")]
    #[rstest]
    #[case("1234567890ABCGH")]
    #[case("invalid")]
    fn openpgp_key_id_deserialize_error(#[case] input: &str) {
        let Err(serde_json::Error { .. }) =
            serde_json::from_str::<OpenPGPKeyId>(&format!("\"{input}\""))
        else {
            panic!("'{input}' erroneously deserialized as an OpenPGPKeyId")
        };
    }

    #[rstest]
    #[case("d2hhdCBhcmUgeW91IGxvb2tpbmcgZm9yPyA7LTsK")]
    fn test_parse_openpgp_signature(#[case] input: &str) -> Result<(), Error> {
        input.parse::<Base64OpenPGPSignature>()?;
        Ok(())
    }

    /// Ensures, that [`Base64OpenPGPSignature::from_str`] fails on invalid input.
    #[rstest]
    // "=" in the middle
    #[case("d2hhdCBhcmUge=W91IGxvb2tpbmcgZm9yPyA7LTsK")]
    // invalid characters
    #[case("!@#$%^&*")]
    // just invalid
    #[case("iHUEABYKh9mi7GCIlMAP9ws/jU4WEbgE=")]
    fn base64_openpgp_signature_from_str_fails_on_invalid_input(#[case] input: &str) {
        let Err(Error::InvalidBase64Encoding { .. }) = Base64OpenPGPSignature::from_str(input)
        else {
            panic!("Expected to fail with Error::InvalidBase64Encoding");
        };
    }

    /// Make sure that invalid base64 encoded OpenPGP signatures don't deserialize.
    #[cfg(feature = "serde")]
    #[rstest]
    #[case("d2hhdCBhcmUge=W91IGxvb2tpbmcgZm9yPyA7LTsK")]
    #[case("!@#$%^&*")]
    fn openpgp_signature_deserialize_error(#[case] input: &str) {
        let Err(serde_json::Error { .. }) =
            serde_json::from_str::<Base64OpenPGPSignature>(&format!("\"{input}\""))
        else {
            panic!("'{input}' erroneously deserialized as a Base64OpenPGPSignature")
        };
    }

    #[rstest]
    #[case(
        "Foobar McFooface (The Third) <foobar@mcfooface.org>",
        Packager{
            name: "Foobar McFooface (The Third)".to_string(),
            email: EmailAddress::from_str("foobar@mcfooface.org")?
        }
    )]
    #[case(
        "Foobar McFooface <foobar@mcfooface.org>",
        Packager{
            name: "Foobar McFooface".to_string(),
            email: EmailAddress::from_str("foobar@mcfooface.org")?
        }
    )]
    fn valid_packager(#[case] from_str: &str, #[case] packager: Packager) -> TestResult {
        assert_eq!(Packager::from_str(from_str)?, packager);

        Ok(())
    }

    /// Test that invalid packager expressions are detected as such and throw the expected error.
    #[rstest]
    #[case::no_name("<foobar@mcfooface.org>")]
    #[case::no_name_and_address_not_wrapped("foobar@mcfooface.org")]
    #[case::no_wrapped_address("Foobar McFooface")]
    #[case::two_wrapped_addresses(
        "Foobar McFooface <foobar@mcfooface.org> <foobar@mcfoofacemcfooface.org>"
    )]
    #[case::address_without_local_part("Foobar McFooface <@mcfooface.org>")]
    fn invalid_packager(#[case] input: &str) -> TestResult {
        let (test_name, _guard) = configure_insta();

        let Err(err_msg) = Packager::from_str(input) else {
            panic!("'{input}' erroneously parsed as a Package")
        };

        assert_snapshot!(test_name, err_msg.to_string());

        Ok(())
    }

    #[rstest]
    #[case(
        Packager::from_str("Foobar McFooface <foobar@mcfooface.org>")?,
        "Foobar McFooface <foobar@mcfooface.org>"
    )]
    fn packager_format_string(
        #[case] packager: Packager,
        #[case] packager_str: &str,
    ) -> TestResult {
        assert_eq!(packager_str, format!("{packager}"));

        Ok(())
    }

    #[rstest]
    #[case(Packager::from_str("Foobar McFooface <foobar@mcfooface.org>")?, "Foobar McFooface")]
    fn packager_name(#[case] packager: Packager, #[case] name: &str) -> TestResult {
        assert_eq!(name, packager.name());

        Ok(())
    }

    #[rstest]
    #[case(
        Packager::from_str("Foobar McFooface <foobar@mcfooface.org>")?,
        &EmailAddress::from_str("foobar@mcfooface.org")?,
    )]
    fn packager_email(#[case] packager: Packager, #[case] email: &EmailAddress) -> TestResult {
        assert_eq!(email, packager.email());

        Ok(())
    }
}
