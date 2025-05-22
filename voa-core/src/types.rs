//! File Hierarchy for the Verification of OS Artifacts (VOA)
//!
//! Types for voa-core

use std::{
    fmt::{Debug, Formatter},
    path::PathBuf,
};

use strum::IntoStaticStr;

use crate::VerifierSourcePath;

/// Error type for voa-core
#[derive(Debug)]
pub enum Error {
    // TODO: define error variants, use thiserror?
}

/// The Os identifier is used to uniquely identify an Operating System (OS), it relies on data
/// provided by [`os-release`].
///
/// [`os-release`]: https://man.archlinux.org/man/os-release.5.en
///
/// # Format
///
/// An Os identifier consists of up to five parts.
/// Each part of the identifier can consist of the characters "0–9", "a–z", ".", "_" and "-".
///
/// In the filesystem, the parts are concatenated into one path using `:` (colon) symbols
/// (e.g. `debian:12:server:company-x:25.01`).
///
/// Trailing colons must be omitted for all parts that are unset
/// (e.g. `arch` instead of `arch::::`).
#[derive(Clone, Debug, PartialEq)]
pub struct Os {
    id: String,
    version_id: Option<String>,
    variant_id: Option<String>,
    image_id: Option<String>,
    image_version: Option<String>,
}

impl Os {
    /// Creates a new operating system identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// use voa_core::types::Os;
    ///
    /// # fn main() -> Result<(), voa_core::types::Error> {
    /// // Arch Linux is a rolling release distribution.
    /// Os::new("arch".into(), None, None, None, None);
    ///
    /// // This Debian system is a special purpose image-based OS.
    /// Os::new(
    ///     "debian".into(),
    ///     Some("12".into()),
    ///     Some("workstation".into()),
    ///     Some("cashier-system".into()),
    ///     Some("1.0.0".into()),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        id: String,
        version_id: Option<String>,
        variant_id: Option<String>,
        image_id: Option<String>,
        image_version: Option<String>,
    ) -> Self {
        assert!(!id.is_empty()); // FIXME: do we want to enforce this, and return a [Result]?

        // TODO: enforce legal character sets for all parts ("0–9", "a–z", ".", "_" and "-")

        Self {
            id,
            version_id,
            variant_id,
            image_id,
            image_version,
        }
    }

    /// A string representation of this Os specifier.
    ///
    /// All parts are joined with `:`, trailing colons are omitted.
    /// Parts that are unset are represented as empty strings.
    pub(crate) fn path_segment(&self) -> PathBuf {
        let distro = format!(
            "{}:{}:{}:{}:{}",
            &self.id,
            self.version_id.as_deref().unwrap_or(""),
            self.variant_id.as_deref().unwrap_or(""),
            self.image_id.as_deref().unwrap_or(""),
            self.image_version.as_deref().unwrap_or(""),
        );

        distro.trim_end_matches(':').into()
    }
}

/// Combines a [`Role`] and a [`Mode`] to describe in what context a verifier is used.
///
/// Describes in what context signature verifiers in a directory structure are used.
///
/// The combination of [`Role`] and [`Mode`] reflects one directory layer in the VOA directory
/// hierarchy. Purpose paths have values such as: `packages`, `trust-anchor-packages`,
/// `repository-metadata`.
#[derive(Clone, Copy, Debug, PartialEq)]
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
    /// # fn main() -> Result<(), voa_core::types::Error> {
    /// Purpose::new(Role::Packages, Mode::ArtifactVerifier);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(role: Role, mode: Mode) -> Self {
        Self { role, mode }
    }

    pub(crate) fn path_segment(&self) -> PathBuf {
        let role: &str = self.role.into();
        let mode: &str = self.mode.into();

        match self.mode {
            Mode::TrustAnchor => format!("{}-{}", mode, role).into(),
            Mode::ArtifactVerifier => role.into(),
        }
    }
}

/// Acts as a trust domain that is associated with a set of verifiers.
///
/// A [`Role`] is always combined with a [`Mode`] and in combination forms a [`Purpose`].
/// E.g. [`Role::Packages`] combined with [`Mode::TrustAnchor`] specify the purpose path
/// `trust-anchor-packages`.
#[derive(Clone, Copy, Debug, PartialEq, IntoStaticStr)]
pub enum Role {
    /// Identifies verifiers used for verifying package signatures.
    #[strum(serialize = "packages")]
    Packages,

    /// Identifies verifiers used for verifying package repository metadata signatures.
    #[strum(serialize = "repository-metadata")]
    RepositoryMetadata,

    /// Identifies verifiers used for verifying OS image signatures.
    #[strum(serialize = "image")]
    Image,
}

/// Component of a [`Purpose`] to distinguish between direct artifact verifiers and trust anchors.
///
/// A [`Mode`] is always combined with a [`Role`] and in combination forms a [`Purpose`].
/// E.g. [`Role::Packages`] combined with [`Mode::TrustAnchor`] specify the purpose path
/// `trust-anchor-packages`.
#[derive(Clone, Copy, Debug, PartialEq, IntoStaticStr)]
pub enum Mode {
    /// Identifies verifiers that are used to directly validate signatures on artifacts.
    #[strum(serialize = "")]
    ArtifactVerifier,

    /// Identifies verifiers that are used to ascertain the authenticity of verifiers used to
    /// directly validate signatures on artifacts.
    #[strum(serialize = "trust-anchor")]
    TrustAnchor,
}

/// A context within a [`Purpose`] for more fine-grained verifier assignments.
///
/// An example for context is the name of a specific software repository when certificates are
/// used in the context of the packages purpose (e.g. "core").
///
/// If no specific context is required, the context `Default` must be used.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Context {
    /// The default context.
    #[default]
    Default,

    /// Defines a custom [`Context`] for verifiers within an [`Os`] and [`Purpose`].
    Custom(CustomContext),
}

impl Context {
    pub(crate) fn path_segment(&self) -> PathBuf {
        match self {
            Self::Default => "default".into(),
            Self::Custom(custom) => custom.as_ref().into(),
        }
    }
}

/// A `CustomContext` encodes a value for a [Context] that is not [Context::Default]
#[derive(Clone, Debug, PartialEq)]
pub struct CustomContext {
    context: String,
}

impl CustomContext {
    /// Creates a new `CustomContext` instance.
    /// Returns `Error` if `value` contains illegal characters.
    pub fn new(value: String) -> Result<Self, Error> {
        // FIXME: check validity of `value` based on limitation of allowed characters

        Ok(Self { context: value })
    }
}

impl AsRef<str> for CustomContext {
    fn as_ref(&self) -> &str {
        self.context.as_ref()
    }
}

/// The name of a cryptography technology for handling specific verifiers and the verification of
/// signatures.
#[derive(Clone, Copy, Debug, PartialEq, IntoStaticStr)]
pub enum Technology {
    /// The [OpenPGP] technology.
    ///
    /// [OpenPGP]: https://www.openpgp.org/
    #[strum(to_string = "openpgp")]
    OpenPGP,

    /// The [SSH] technology.
    ///
    /// [SSH]: https://www.openssh.com/
    #[strum(to_string = "ssh")]
    SSH,
}

impl Technology {
    pub(crate) fn path_segment(&self) -> PathBuf {
        let segment: &str = self.into();
        segment.into()
    }
}

impl VerifierSourcePath {
    /// The filesystem path that this [VerifierSourcePath] represents.
    /// This representation of the path doesn't canonicalize symlinks, if any.
    pub(crate) fn to_path_buf(&self) -> PathBuf {
        self.load_path
            .join(self.os.path_segment())
            .join(self.purpose.path_segment())
            .join(self.context.path_segment())
            .join(self.technology.path_segment())
    }

    /// The [`Os`] of the [`VerifierSourcePath`].
    pub fn os(&self) -> &Os {
        &self.os
    }

    /// The [`Purpose`] of the [`VerifierSourcePath`].
    pub fn purpose(&self) -> Purpose {
        self.purpose
    }

    /// The [`Context`] of the [`VerifierSourcePath`].
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// The [`Technology`] of the [`VerifierSourcePath`].
    pub fn technology(&self) -> Technology {
        self.technology
    }
}

/// Points to a signature verifier in the file system.
///
/// Depending on the technology, this may represent, e.g.:
/// - an individual, loose verifier
/// - a certificate complete with its trust chain
/// - a set of individual verifiers in one shared data structure
pub struct Verifier {
    /// Specification of the path from which the verifier was loaded
    pub(crate) path: VerifierSourcePath,

    /// Filename of the verifier file, in [`Verifier::path`]
    pub(crate) filename: String,
}

impl Debug for Verifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Path: {:#?}", self.path)?;
        writeln!(f, "Filename: {}", self.filename)?;

        Ok(())
    }
}

impl Verifier {
    /// The source path of this verifier
    pub fn source_path(&self) -> &VerifierSourcePath {
        &self.path
    }

    /// The filename (excluding the path)
    pub fn filename(&self) -> &str {
        &self.filename
    }

    /// Returns the [`PathBuf`] representation of the [`Verifier`] (not canonicalized).
    pub fn to_path_buf(&self) -> PathBuf {
        self.source_path().to_path_buf().join(&self.filename)
    }
}
