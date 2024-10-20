use std::fmt::{Display, Formatter};
use std::str::FromStr;

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
}
