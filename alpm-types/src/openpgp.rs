use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    string::ToString,
};

use base64::{Engine, prelude::BASE64_STANDARD};
use email_address::EmailAddress;
use fluent_i18n::t;
use serde::{Deserialize, Serialize};
use winnow::{
    ModalResult,
    Parser,
    combinator::{cut_err, eof, seq},
    error::{StrContext, StrContextValue},
    token::take_till,
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
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum OpenPGPIdentifier {
    /// An OpenPGP Key ID.
    #[serde(rename = "openpgp_key_id")]
    OpenPGPKeyId(OpenPGPKeyId),
    /// An OpenPGP v4 fingerprint.
    #[serde(rename = "openpgp_v4_fingerprint")]
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
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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

    /// Parses a [`Packager`] from a string slice.
    ///
    /// Consumes all of its input.
    ///
    /// # Examples
    ///
    /// See [`Self::from_str`] for code examples.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` does not represent a valid [`Packager`].
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        seq!(Self {
            // The name that precedes the email address
            name: cut_err(take_till(1.., '<'))
                .map(|s: &str| s.trim().to_string())
                .context(StrContext::Label("packager name")),
            // The '<' delimiter that marks the start of the email string
            _: cut_err('<').context(StrContext::Label("or missing opening delimiter '<' for email address")),
            // The email address, which is validated by the EmailAddress struct.
            email: cut_err(
                take_till(1.., '>')
                    .try_map(EmailAddress::from_str))
                    .context(StrContext::Label("Email address")
                ),
            // The '>' delimiter that marks the end of the email string
            _: cut_err('>').context(StrContext::Label("or missing closing delimiter '>' for email address")),
            _: eof.context(StrContext::Expected(StrContextValue::Description("end of packager string"))),
        })
        .parse_next(input)
    }
}

impl FromStr for Packager {
    type Err = Error;
    /// Create a Packager from a string
    fn from_str(s: &str) -> Result<Packager, Self::Err> {
        Ok(Self::parser.parse(s)?)
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
    use testresult::TestResult;

    use super::*;

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
    // Starts with whitespace
    #[case(
        " 4A0C 4DFF C02E 1A7E D969 ED23 1C23 58A2 5A10 D94E",
        Err(Error::InvalidOpenPGPv4Fingerprint)
    )]
    // Ends with whitespace
    #[case(
        "4A0C 4DFF C02E 1A7E D969 ED23 1C23 58A2 5A10 D94E ",
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

    #[test]
    fn test_serialize_openpgp_key_id() -> TestResult {
        let id = "584A3EBFE705CDCD".parse::<OpenPGPKeyId>()?;
        let json = serde_json::to_string(&OpenPGPIdentifier::OpenPGPKeyId(id))?;
        assert_eq!(r#"{"openpgp_key_id":"584A3EBFE705CDCD"}"#, json);

        Ok(())
    }

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
    #[case("d2hhdCBhcmUgeW91IGxvb2tpbmcgZm9yPyA7LTsK")]
    fn test_parse_openpgp_signature(#[case] input: &str) -> Result<(), Error> {
        input.parse::<Base64OpenPGPSignature>()?;
        Ok(())
    }

    #[rstest]
    // "=" in the middle
    #[case(
        "d2hhdCBhcmUge=W91IGxvb2tpbmcgZm9yPyA7LTsK",
        Err(Error::InvalidBase64Encoding { expected_item: t!("error-invalid-base64-encoding-pgp-signature") })
    )]
    // invalid characters
    #[case("!@#$%^&*", Err(Error::InvalidBase64Encoding { expected_item: t!("error-invalid-base64-encoding-pgp-signature") }))]
    // just invalid
    #[case(
        "iHUEABYKh9mi7GCIlMAP9ws/jU4WEbgE=",
        Err(Error::InvalidBase64Encoding { expected_item: t!("error-invalid-base64-encoding-pgp-signature") })
    )]
    fn test_parse_invalid_openpgp_signature(
        #[case] input: &str,
        #[case] expected: Result<Base64OpenPGPSignature, Error>,
    ) {
        let result = input.parse::<Base64OpenPGPSignature>();
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

    /// Test that invalid packager expressions are detected as such and throw the expected error.
    #[rstest]
    #[case::no_name("<foobar@mcfooface.org>", "invalid packager name")]
    #[case::no_name_and_address_not_wrapped(
        "foobar@mcfooface.org",
        "invalid or missing opening delimiter '<' for email address"
    )]
    #[case::no_wrapped_address(
        "Foobar McFooface",
        "invalid or missing opening delimiter '<' for email address"
    )]
    #[case::two_wrapped_addresses(
        "Foobar McFooface <foobar@mcfooface.org> <foobar@mcfoofacemcfooface.org>",
        "expected end of packager string"
    )]
    #[case::address_without_local_part("Foobar McFooface <@mcfooface.org>", "Local part is empty")]
    fn invalid_packager(#[case] packager: &str, #[case] expected_error: &str) -> TestResult {
        let Err(err) = Packager::from_str(packager) else {
            panic!("Expected packager string to be invalid: {packager}");
        };

        let error = err.to_string();
        assert!(
            error.contains(expected_error),
            "Expected error:\n{error}\n\nto contain string:\n{expected_error}"
        );

        Ok(())
    }

    #[rstest]
    #[case(
        Packager::from_str("Foobar McFooface <foobar@mcfooface.org>").unwrap(),
        "Foobar McFooface <foobar@mcfooface.org>"
    )]
    fn packager_format_string(#[case] packager: Packager, #[case] packager_str: &str) {
        assert_eq!(packager_str, format!("{packager}"));
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
