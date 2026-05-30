//! Package validation handling.

use std::str::FromStr;

use alpm_parsers::{iter_str_context, prelude::*};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString, VariantNames};
use winnow::{
    Parser,
    ascii::alphanumeric1,
    error::{ErrMode, StrContext, StrContextValue},
};

/// The validation method used during installation of a package.
///
/// A validation method can ensure the integrity of a package.
/// Certain methods (i.e. [`PackageValidation::Pgp`]) can also be used to ensure a package's
/// authenticity.
///
/// # Examples
///
/// Parsing from strings:
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::PackageValidation;
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// assert_eq!(
///     PackageValidation::from_str("none")?,
///     PackageValidation::None
/// );
/// assert_eq!(PackageValidation::from_str("md5")?, PackageValidation::Md5);
/// assert_eq!(
///     PackageValidation::from_str("sha256")?,
///     PackageValidation::Sha256
/// );
/// assert_eq!(PackageValidation::from_str("pgp")?, PackageValidation::Pgp);
///
/// // Invalid values return an error.
/// assert!(PackageValidation::from_str("crc32").is_err());
/// # Ok(())
/// # }
/// ```
///
/// Displaying and serializing:
///
/// ```
/// use alpm_types::PackageValidation;
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// assert_eq!(PackageValidation::Md5.to_string(), "md5");
/// assert_eq!(
///     serde_json::to_string(&PackageValidation::Sha256).expect("Serialization failed"),
///     "\"Sha256\""
/// );
/// # Ok(())
/// # }
/// ```
#[derive(
    Clone, Debug, PartialEq, Deserialize, Serialize, EnumString, Display, AsRefStr, VariantNames,
)]
#[strum(serialize_all = "lowercase")]
pub enum PackageValidation {
    /// The package integrity and authenticity is **not validated**.
    None,
    /// The package is validated against an accompanying **MD5 hash digest**.
    Md5,
    /// The package is validated against an accompanying **SHA-256 hash digest**.
    Sha256,
    /// The package is validated using **PGP signatures**.
    Pgp,
}

impl AlpmParser for PackageValidation {
    /// Recognizes a [`PackageValidation`] in a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if the immediate alphanumeric `input` is not a valid variant
    /// a `PackageValidation`.
    fn parser<'a>(input: &mut Input<'a>) -> PResult<'a, Self> {
        alphanumeric1
            .try_map(PackageValidation::from_str)
            .context_with(iter_str_context!([PackageValidation::VARIANTS]))
            .layer("package validation method")
            .parse_next(input)
    }

    fn delimiter_error_context<'a, O, P>(
        parser: P,
    ) -> impl Parser<Input<'a>, O, ErrMode<ParseStack<'a>>>
    where
        P: Parser<Input<'a>, O, ErrMode<ParseStack<'a>>>,
    {
        parser
            .context(StrContext::Expected(StrContextValue::Description(
                "an alphanumeric string",
            )))
            .context_with(iter_str_context!([PackageValidation::VARIANTS]))
            .layer("package validation method")
    }
}
