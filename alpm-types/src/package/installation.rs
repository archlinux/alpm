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
    Default,
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
    #[default]
    #[strum(to_string = "0")]
    Explicit = 0,
    /// Installed as a dependency for another package.
    #[strum(to_string = "1")]
    Depend = 1,
}
