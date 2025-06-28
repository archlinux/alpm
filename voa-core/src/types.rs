//! File Hierarchy for the Verification of OS Artifacts (VOA)
//!
//! Types for voa-core

use std::{
    fmt::{Debug, Display, Formatter},
    fs::File,
    path::{Path, PathBuf},
};

use strum::IntoStaticStr;

use crate::error::Error;
pub use crate::load_path::LoadPath;

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
///
/// See <https://uapi-group.org/specifications/specs/file_hierarchy_for_the_verification_of_os_artifacts/#os>
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
    /// Returns `Error` if any parameter contains illegal characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use voa_core::types::Os;
    ///
    /// # fn main() -> Result<(), voa_core::error::Error> {
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
    ) -> Result<Self, Error> {
        if id.is_empty() {
            // FIXME: define an appropriate error type
            return Err(Error::IllegalIdentifier);
        }

        // FIXME: check validity of inputs based on limitation of allowed characters
        // legal character sets for all parts: ("0–9", "a–z", ".", "_" and "-")

        Ok(Self {
            id,
            version_id,
            variant_id,
            image_id,
            image_version,
        })
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
    /// # fn main() -> Result<(), voa_core::error::Error> {
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
        // FIXME: check validity of `value` based on limitation of allowed characters

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

/// A context within a [`Purpose`] for more fine-grained verifier assignments.
///
/// An example for context is the name of a specific software repository when certificates are
/// used in the context of the packages purpose (e.g. "core").
///
/// If no specific context is required, the context `Default` must be used.
///
/// See <https://uapi-group.org/specifications/specs/file_hierarchy_for_the_verification_of_os_artifacts/#context>
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
    ///
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

/// The name of a technology backend.
///
/// Technology-specific backends implement the logic for each supported verification technology
/// in VOA.
///
/// See <https://uapi-group.org/specifications/specs/file_hierarchy_for_the_verification_of_os_artifacts/#technology>
#[derive(Clone, Debug, IntoStaticStr, PartialEq)]
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

    /// Defines a custom [`Technology`] name.
    CustomTechnology(CustomTechnology),
}

impl Technology {
    pub(crate) fn path_segment(&self) -> PathBuf {
        let segment: &str = self.into();
        segment.into()
    }
}

/// A `CustomTechnology` defines a technology name that is not covered by the variants defined in
/// [Technology].
#[derive(Clone, Debug, PartialEq)]
pub struct CustomTechnology {
    technology: String,
}

impl CustomTechnology {
    /// Creates a new `CustomTechnology` instance.
    ///
    /// Returns `Error` if `value` contains illegal characters.
    pub fn new(value: String) -> Result<Self, Error> {
        // FIXME: check validity of `value` based on limitation of allowed characters

        Ok(Self { technology: value })
    }
}

impl AsRef<str> for CustomTechnology {
    fn as_ref(&self) -> &str {
        self.technology.as_ref()
    }
}

/// Specifies a logical location in a VOA directory structure.
#[derive(Clone, Debug, PartialEq)]
pub struct VerifierSourcePath {
    load_path: LoadPath,
    os: Os,
    purpose: Purpose,
    context: Context,
    technology: Technology,
}

impl VerifierSourcePath {
    pub(crate) fn new(
        load_path: LoadPath,
        os: Os,
        purpose: Purpose,
        context: Context,
        technology: Technology,
    ) -> Self {
        Self {
            load_path,
            os,
            purpose,
            context,
            technology,
        }
    }

    /// The filesystem path that this [VerifierSourcePath] represents.
    /// This representation of the path doesn't canonicalize symlinks, if any.
    #[allow(dead_code)]
    pub(crate) fn to_path_buf(&self) -> PathBuf {
        self.load_path
            .path
            .join(self.os.path_segment())
            .join(self.purpose.path_segment())
            .join(self.context.path_segment())
            .join(self.technology.path_segment())
    }

    /// The [`LoadPath`] of the [`VerifierSourcePath`].
    ///
    /// TODO: maybe return just a &Path, and keep the [LoadPath] type private?
    pub fn load_path(&self) -> &LoadPath {
        &self.load_path
    }

    /// The [`Os`] of the [`VerifierSourcePath`].
    pub fn os(&self) -> &Os {
        &self.os
    }

    /// The [`Purpose`] of the [`VerifierSourcePath`].
    pub fn purpose(&self) -> &Purpose {
        &self.purpose
    }

    /// The [`Context`] of the [`VerifierSourcePath`].
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// The [`Technology`] of the [`VerifierSourcePath`].
    pub fn technology(&self) -> &Technology {
        &self.technology
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
    pub(crate) verifier_path: VerifierSourcePath,

    /// Canonicalized path of the verifier file, in [`Verifier::path`]
    pub(crate) canonicalized: PathBuf,
}

impl Debug for Verifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Verifier source path: {:#?}", self.verifier_path)?;
        writeln!(f, "Canonicalized verifier path: {:?}", self.canonicalized)?;

        Ok(())
    }
}

impl Verifier {
    /// The verifier source path definition that this verifier file was found through
    pub fn verifier_path(&self) -> &VerifierSourcePath {
        &self.verifier_path
    }

    /// The canonicalized filename (excluding the path)
    pub(crate) fn filename(&self) -> Option<&std::ffi::OsStr> {
        self.canonicalized.file_name()
    }

    /// The canonicalized [`Path`] representation of this [`Verifier`]
    pub fn path(&self) -> &Path {
        &self.canonicalized
    }

    /// Open this verifier as a file in read-only mode
    pub fn open(&self) -> std::io::Result<File> {
        File::open(&self.canonicalized)
    }
}

#[test]
fn role_to_string() {
    assert_eq!(Role::Image.to_string(), "image");

    let foo = CustomRole::new("foo".to_string()).unwrap();
    let custom_role = Role::Custom(foo);
    assert_eq!(custom_role.to_string(), "foo");
}
