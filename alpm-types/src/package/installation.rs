//! Package installation handling.

/// Represents the reason why a package was installed.
///
/// # Examples
///
/// Parsing from strings:
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::PackageInstallReason;
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// assert_eq!(
///     PackageInstallReason::from_str("0")?,
///     PackageInstallReason::Explicit
/// );
/// assert_eq!(
///     PackageInstallReason::from_str("1")?,
///     PackageInstallReason::Depend
/// );
///
/// // Invalid values return an error.
/// assert!(PackageInstallReason::from_str("2").is_err());
/// # Ok(())
/// # }
/// ```
///
/// Displaying and serializing:
///
/// ```
/// use alpm_types::PackageInstallReason;
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// assert_eq!(PackageInstallReason::Explicit.to_string(), "0");
/// assert_eq!(
///     serde_json::to_string(&PackageInstallReason::Depend).expect("Serialization failed"),
///     "\"Depend\""
/// );
/// # Ok(())
/// # }
/// ```
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    serde::Deserialize,
    serde::Serialize,
    strum::EnumString,
    strum::Display,
    strum::AsRefStr,
)]
#[repr(u8)]
pub enum PackageInstallReason {
    /// Explicitly requested by the user.
    #[strum(to_string = "0")]
    Explicit = 0,
    /// Installed as a dependency for another package.
    #[strum(to_string = "1")]
    Depend = 1,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;
    use testresult::TestResult;

    use super::*;

    /// Ensures that all valid [`PackageInstallReason`] variants parse successfully.
    #[rstest]
    #[case::explicit("0", PackageInstallReason::Explicit)]
    #[case::depend("1", PackageInstallReason::Depend)]
    fn parses_known_variants(
        #[case] input: &str,
        #[case] expected: PackageInstallReason,
    ) -> TestResult {
        assert_eq!(PackageInstallReason::from_str(input)?, expected);
        Ok(())
    }

    /// Ensures that invalid variants produce an error.
    #[rstest]
    #[case::invalid("2")]
    #[case::negative("-1")]
    #[case::empty("")]
    #[case::text("explicit")]
    fn rejects_unknown_variant(#[case] input: &str) -> TestResult {
        assert!(PackageInstallReason::from_str(input).is_err());
        Ok(())
    }

    /// Ensures that [`Display`] produces the expected numeric output.
    #[rstest]
    #[case::explicit(PackageInstallReason::Explicit, "0")]
    #[case::depend(PackageInstallReason::Depend, "1")]
    fn display_outputs_expected_string(
        #[case] variant: PackageInstallReason,
        #[case] expected: &str,
    ) -> TestResult {
        assert_eq!(variant.to_string(), expected);
        Ok(())
    }

    /// Ensures that [`serde`] serialization and deserialization preserve variant casing.
    #[rstest]
    #[case::explicit(PackageInstallReason::Explicit, "\"Explicit\"")]
    #[case::depend(PackageInstallReason::Depend, "\"Depend\"")]
    fn serde_roundtrip(
        #[case] variant: PackageInstallReason,
        #[case] expected_json: &str,
    ) -> TestResult {
        assert_eq!(serde_json::to_string(&variant)?, expected_json);
        assert_eq!(
            serde_json::from_str::<PackageInstallReason>(expected_json)?,
            variant
        );
        Ok(())
    }
}
