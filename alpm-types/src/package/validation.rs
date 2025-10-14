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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;
    use testresult::TestResult;

    use super::*;

    /// Ensures that all valid [`PackageValidation`] variants parse successfully.
    #[rstest]
    #[case::none("none", PackageValidation::None)]
    #[case::md5("md5", PackageValidation::Md5)]
    #[case::sha256("sha256", PackageValidation::Sha256)]
    #[case::pgp("pgp", PackageValidation::Pgp)]
    fn parses_known_variants(
        #[case] input: &str,
        #[case] expected: PackageValidation,
    ) -> TestResult {
        assert_eq!(PackageValidation::from_str(input)?, expected);
        Ok(())
    }

    /// Ensures that invalid variants produce an error.
    #[rstest]
    #[case::invalid_hash("crc32")]
    #[case::random("random")]
    #[case::empty("")]
    fn rejects_unknown_variant(#[case] input: &str) -> TestResult {
        assert!(PackageValidation::from_str(input).is_err());
        assert!(PackageValidation::try_from(input).is_err());
        Ok(())
    }

    /// Ensures that [`Display`] produces the correct lowercase output.
    #[rstest]
    #[case::none(PackageValidation::None, "none")]
    #[case::md5(PackageValidation::Md5, "md5")]
    #[case::sha256(PackageValidation::Sha256, "sha256")]
    #[case::pgp(PackageValidation::Pgp, "pgp")]
    fn display_outputs_expected_string(
        #[case] variant: PackageValidation,
        #[case] expected: &str,
    ) -> TestResult {
        assert_eq!(variant.to_string(), expected);
        Ok(())
    }

    /// Ensures that [`serde`] serialization and deserialization preserve variant casing.
    #[rstest]
    #[case::none(PackageValidation::None, "\"None\"")]
    #[case::md5(PackageValidation::Md5, "\"Md5\"")]
    #[case::sha256(PackageValidation::Sha256, "\"Sha256\"")]
    #[case::pgp(PackageValidation::Pgp, "\"Pgp\"")]
    fn serde_roundtrip(
        #[case] variant: PackageValidation,
        #[case] expected_json: &str,
    ) -> TestResult {
        assert_eq!(serde_json::to_string(&variant)?, expected_json);
        assert_eq!(
            serde_json::from_str::<PackageValidation>(expected_json)?,
            variant
        );
        Ok(())
    }
}
