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
    /// use voa_core::identifiers::{Mode, Purpose, Role};
    ///
    /// # fn main() -> Result<(), voa_core::Error> {
    /// Purpose::new(Role::Packages, Mode::ArtifactVerifier);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(role: Role, mode: Mode) -> Self {
        Self { role, mode }
    }

    fn purpose_to_string(&self) -> String {
        match self.mode {
            Mode::TrustAnchor => format!("{}-{}", self.mode, self.role),
            Mode::ArtifactVerifier => format!("{}", self.role),
        }
    }

    pub(crate) fn path_segment(&self) -> PathBuf {
        self.purpose_to_string().into()
    }
}

impl Display for Purpose {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.purpose_to_string())
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
    role: String,
}

impl Display for CustomRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.role)
    }
}

impl CustomRole {
    /// Creates a new `CustomRole` instance.
    ///
    /// Returns `Error` if `value` contains illegal characters.
    pub fn new(role: String) -> Result<Self, Error> {
        identifiers::check_identifier_part(&role)?;

        Ok(Self { role })
    }
}

/// Component of a [`Purpose`] to distinguish between direct artifact verifiers and trust anchors.
///
/// A [`Mode`] is always combined with a [`Role`] and in combination forms a [`Purpose`].
/// E.g. [`Role::Packages`] combined with [`Mode::TrustAnchor`] specify the purpose path
/// `trust-anchor-packages`.
///
/// See <https://uapi-group.org/specifications/specs/file_hierarchy_for_the_verification_of_os_artifacts/#purpose>
#[derive(Clone, Copy, Debug, strum::Display, IntoStaticStr, PartialEq)]
pub enum Mode {
    /// Identifies verifiers that are used to directly validate signatures on artifacts.
    #[strum(serialize = "")]
    ArtifactVerifier,

    /// Identifies verifiers that are used to ascertain the authenticity of verifiers used to
    /// directly validate signatures on artifacts.
    #[strum(serialize = "trust-anchor")]
    TrustAnchor,
}

#[cfg(test)]
mod tests {
    use crate::identifiers::{CustomRole, Mode, Purpose, Role};

    #[test]
    fn purpose_display() {
        assert_eq!(format!("{}", Mode::ArtifactVerifier), "");
        assert_eq!(format!("{}", Mode::TrustAnchor), "trust-anchor");

        assert_eq!(format!("{}", Role::Packages), "packages");
        assert_eq!(format!("{}", Role::Image), "image");
        assert_eq!(
            format!("{}", Role::RepositoryMetadata),
            "repository-metadata"
        );
        assert_eq!(
            format!("{}", Role::Custom(CustomRole::new("foo".into()).unwrap())),
            "foo"
        );

        assert_eq!(
            format!("{}", Purpose::new(Role::Packages, Mode::ArtifactVerifier)),
            "packages"
        );
        assert_eq!(
            format!("{}", Purpose::new(Role::Packages, Mode::TrustAnchor)),
            "trust-anchor-packages"
        );
        assert_eq!(
            format!(
                "{}",
                Purpose::new(Role::RepositoryMetadata, Mode::TrustAnchor)
            ),
            "trust-anchor-repository-metadata"
        );

        assert_eq!(
            format!(
                "{}",
                Purpose::new(
                    Role::Custom(CustomRole::new("foo".into()).unwrap()),
                    Mode::ArtifactVerifier
                )
            ),
            "foo"
        );
        assert_eq!(
            format!(
                "{}",
                Purpose::new(
                    Role::Custom(CustomRole::new("foo".into()).unwrap()),
                    Mode::TrustAnchor
                )
            ),
            "trust-anchor-foo"
        );
    }
}
