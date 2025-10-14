//! Package validation handling.

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
    Clone,
    Debug,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    strum::EnumString,
    strum::Display,
    strum::AsRefStr,
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
