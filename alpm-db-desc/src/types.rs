//! Types needed for defining a database desc file.

/// The validation method used during installation of the package ensuring its authenticity.
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
    /// he package integrity and authenticity is not validated.
    None,
    /// The package is validated against an accompanying MD-5 hash digest.
    #[strum(to_string = "md5")]
    Md5,
    /// The package is validated against an accompanying SHA-256 hash digest.
    #[strum(to_string = "sha256")]
    Sha256,
    /// The package is validated using PGP signatures.
    #[strum(to_string = "pgp")]
    Pgp,
}

/// Represents the reason why a package was installed.
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
    /// Failed parsing of local database.
    #[strum(to_string = "2")]
    Unknown = 2,
}
