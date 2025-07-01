use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

use strum::IntoStaticStr;

use crate::{error::Error, identifiers};

/// Combines a [`Role`] and a [`Mode`] to describe in what context a verifier is used.
///
/// Describes in what context signature verifiers in a directory structure are used.
///
/// The combination of [`Role`] and [`Mode`] reflects one directory layer in the VOA directory
/// hierarchy. Purpose paths have values such as: `packages`, `trust-anchor-packages`,
/// `repository-metadata`.
///
/// See <https://uapi-group.org/specifications/specs/file_hierarchy_for_the_verification_of_os_artifacts/#purpose>
#[derive(Clone, Debug, PartialEq)]
pub struct Purpose {
    role: Role,
    mode: Mode,
}

impl Purpose {
    /// Create a new [`Purpose`].
    ///
    /// # Examples
    ///
    /// ```
    /// use voa_core::types::{Mode, Purpose, Role};
    ///
    /// # fn main() -> Result<(), voa_core::Error> {
    /// Purpose::new(Role::Packages, Mode::ArtifactVerifier);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(role: Role, mode: Mode) -> Self {
        Self { role, mode }
    }

    pub(crate) fn path_segment(&self) -> PathBuf {
        let role = self.role.to_string();
        let mode: &str = self.mode.into();

        match self.mode {
            Mode::TrustAnchor => format!("{mode}-{role}").into(),
            Mode::ArtifactVerifier => role.into(),
        }
    }
}

/// Acts as a trust domain that is associated with a set of verifiers.
///
/// A [`Role`] is always combined with a [`Mode`] and in combination forms a [`Purpose`].
/// E.g. [`Role::Packages`] combined with [`Mode::TrustAnchor`] specify the purpose path
/// `trust-anchor-packages`.
///
/// See <https://uapi-group.org/specifications/specs/file_hierarchy_for_the_verification_of_os_artifacts/#purpose>
#[derive(Clone, Debug, strum::Display, PartialEq)]
pub enum Role {
    /// Identifies verifiers used for verifying package signatures.
    #[strum(to_string = "packages")]
    Packages,

    /// Identifies verifiers used for verifying package repository metadata signatures.
    #[strum(to_string = "repository-metadata")]
    RepositoryMetadata,

    /// Identifies verifiers used for verifying OS image signatures.
    #[strum(to_string = "image")]
    Image,

    /// Identifies verifiers used for verifying OS image signatures.
    #[strum(to_string = "{0}")]
    Custom(CustomRole),
}

/// A `CustomRole` encodes a custom value for a [Role]
#[derive(Clone, Debug, PartialEq)]
pub struct CustomRole {
    context: String,
}

impl Display for CustomRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.context)
    }
}

impl CustomRole {
    /// Creates a new `CustomRole` instance.
    ///
    /// Returns `Error` if `value` contains illegal characters.
    pub fn new(value: String) -> Result<Self, Error> {
        if value.is_empty() {
            return Err(Error::IllegalIdentifier);
        }

        identifiers::check_identifier_part(&value)?;

        Ok(Self { context: value })
    }
}

/// Component of a [`Purpose`] to distinguish between direct artifact verifiers and trust anchors.
///
/// A [`Mode`] is always combined with a [`Role`] and in combination forms a [`Purpose`].
/// E.g. [`Role::Packages`] combined with [`Mode::TrustAnchor`] specify the purpose path
/// `trust-anchor-packages`.
///
/// See <https://uapi-group.org/specifications/specs/file_hierarchy_for_the_verification_of_os_artifacts/#purpose>
#[derive(Clone, Copy, Debug, IntoStaticStr, PartialEq)]
pub enum Mode {
    /// Identifies verifiers that are used to directly validate signatures on artifacts.
    #[strum(serialize = "")]
    ArtifactVerifier,

    /// Identifies verifiers that are used to ascertain the authenticity of verifiers used to
    /// directly validate signatures on artifacts.
    #[strum(serialize = "trust-anchor")]
    TrustAnchor,
}
