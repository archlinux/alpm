//! File Hierarchy for the Verification of OS Artifacts (VOA)
//!
//! A mechanism for the storage and retrieval of cryptography technology-agnostic signature
//! verifiers. For specification draft see: <https://github.com/uapi-group/specifications/pull/134>
//!
//! The VOA hierarchy acts as structured storage for files that contain "signature verifiers"
//! (such as [OpenPGP certificates], aka "public keys").
//!
//! A set of "load paths" may exist on a system, each containing different sets of verifier files.
//! This library provides an abstract, unified view of this set of signature verifiers in that set
//! of load paths.
//!
//! Libraries dealing with a specific cryptographic technology can rely on this library to collect
//! the paths of all verifier files relevant to them.
//!
//! Each load path contains a VOA hierarchy of verifier files.
//! Earlier load paths have precedence over later entries (in some technologies).
//!
//! NOTE: Depending on the technology, multiple versions of the same verifier will either "shadow"
//! one another, or get merged into one coherent view that represents the totality of available
//! information about the verifier.
//!
//! Shadowing/merging is specific to each signing technology and must be handled in the
//! technology-specific library.
//! For more details see e.g. the `voa-openpgp` implementation and the VOA specification.
//!
//! VOA expects that filenames are a strong identifier that signals whether two verifier files
//! contain variants of "the same" logical verifier.
//! Verifiers from different load paths can be identified as related via their filenames.
//!
//! For example, [OpenPGP certificates] must be stored using filenames based on their fingerprint.
//!
//!
//! [OpenPGP certificates]: https://openpgp.dev/book/certificates.html

#![warn(missing_docs)]

use std::{
    fmt::{Debug, Formatter},
    fs::{read_dir, read_link},
    path::{Path, PathBuf},
};

use log::{debug, trace, warn};
use strum::IntoStaticStr;

/// Error type for voa-core
#[derive(Debug)]
pub enum Error {
    // TODO: define error variants, use thiserror?
}

/// Load paths for "system mode" operation of VOA.
pub const LOAD_PATHS_SYSTEM_MODE: &[&str] = &[
    "/etc/voa/",
    "/run/voa/",
    "/usr/local/share/voa/",
    "/usr/share/voa/",
];

// TODO: const LOAD_PATHS_USER_MODE
//
// $XDG_CONFIG_HOME/voa/
// the ./voa/ directory in each directory defined in $XDG_CONFIG_DIRS
// $XDG_RUNTIME_DIR/voa/
// $XDG_DATA_HOME/voa/
// the ./voa/ directory in each directory defined in $XDG_DATA_DIRS

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
    /// use voa_core::Os;
    ///
    /// # fn main() -> Result<(), voa_core::Error> {
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
    fn path_segment(&self) -> PathBuf {
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
    /// use voa_core::{Mode, Purpose, Role};
    ///
    /// # fn main() -> Result<(), voa_core::Error> {
    /// Purpose::new(Role::Packages, Mode::ArtifactVerifier);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(role: Role, mode: Mode) -> Self {
        Self { role, mode }
    }

    fn path_segment(&self) -> PathBuf {
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
    fn path_segment(&self) -> PathBuf {
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
    fn path_segment(&self) -> PathBuf {
        let segment: &str = self.into();
        segment.into()
    }
}

/// A path in a VOA structure in which verifier files are stored.
#[derive(Clone, Debug, PartialEq)]
pub struct VerifierSourcePath {
    load_path: PathBuf,
    os: Os,
    purpose: Purpose,
    context: Context,
    technology: Technology,
}

impl VerifierSourcePath {
    /// The filesystem path that this [VerifierSourcePath] represents.
    /// This representation of the path doesn't canonicalize symlinks, if any.
    fn to_path_buf(&self) -> PathBuf {
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

/// This structure represents a [VerifierSourcePath] that has been checked for "legality":
///
/// Constructing it checks the legality of symlinks (if any) in the VOA path structure, and persists
/// the canonicalized path to the target directory.
struct CheckedVerifierSourcePath {
    verifier_source_path: VerifierSourcePath,

    // The canonicalized target path of this VerifierSourcePath (i.e. with resolved symlinks)
    canonicalized_target: PathBuf,
}

impl CheckedVerifierSourcePath {
    /// Creates a new `CheckedVerifierSourcePath` from a `VerifierSourcePath`.
    ///
    /// Ensures, that the provided [`VerifierSourcePath`] represents a legal path on the local
    /// filesystem, and any involved symlinks conform to the VOA symlink restrictions.
    fn new(verifier_source_path: VerifierSourcePath) -> std::io::Result<Self> {
        // Canonicalized base load path
        // (any potential internal symlinks of this top level "load_path" are not checked)
        let base_path = verifier_source_path.load_path.canonicalize()?;

        /// Append a segment to a path and ensure that the resulting path conforms
        /// to the VOA symlink constraints.
        fn append(
            p: &Path,
            segment: &Path,
            // FIXME: This actually needs a set of load paths that this verifier path may use
            base_path: &Path,
        ) -> std::io::Result<PathBuf> {
            let mut buf = p.join(segment);
            if buf.is_symlink() {
                // Check that the symlink-canonicalized path is acceptable, including this segment.
                let canon = buf.canonicalize()?;

                // FIXME: how does this interact with chains of symlinks? Does it process them at
                // once? If so, we still need to check legality of each intermediate hop!

                // TODO: This check should actually allow links into some of the other current load
                //       paths (but not all of them!)
                //        -> we should actually check "starts_with" against canonicalized forms of
                //           all currently legal-to-link-into load paths
                //
                // [..] However, symlinks can be used in the VOA hierarchy to point
                // to files or directories below one of the [load paths] in
                // descending priority.
                // Symlinks to files or directories below ephemeral load paths
                // (i.e. `/run/voa/` and `$XDG_RUNTIME_DIR/voa/`) are prohibited, as they
                // could lead to dangling references. [..]
                if !canon.starts_with(base_path) {
                    trace!("append: illegal path segment {segment:?} following {buf:?}");

                    return Err(std::io::Error::other(format!(
                        "{segment:?} is not valid following {buf:?} (it points to {canon:?})"
                    )));
                }

                buf = canon;
            }

            if !buf.is_dir() {
                trace!("CheckedVerifierSourcePath::new {buf:?} is not a directory");

                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotADirectory,
                    format!("{buf:?} is not a directory"),
                ));
            }

            Ok(buf)
        }

        let mut path = append(
            &base_path,
            &verifier_source_path.os.path_segment(),
            &base_path,
        )?;
        path = append(
            &path,
            &verifier_source_path.purpose.path_segment(),
            &base_path,
        )?;
        path = append(
            &path,
            &verifier_source_path.context.path_segment(),
            &base_path,
        )?;
        path = append(
            &path,
            &verifier_source_path.technology.path_segment(),
            &base_path,
        )?;

        trace!("CheckedVerifierSourcePath::new canonicalized path: {path:?}");

        Ok(Self {
            verifier_source_path,
            canonicalized_target: path,
        })
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
    path: VerifierSourcePath,

    /// Filename of the verifier file, in [`Verifier::path`]
    filename: String,
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

/// VOA defines a list of _load paths_ with descending priority for system mode and user mode.
///
/// The following load paths are considered, depending on the load path mode:
///
/// System Mode:
/// - /etc/voa/
/// - /run/voa/
/// - /usr/local/share/voa/
/// - /usr/share/voa/
///
/// User Mode:
/// - `$XDG_CONFIG_HOME/voa/`
/// - the `./voa/` directory in each directory defined in `$XDG_CONFIG_DIRS`
/// - `$XDG_RUNTIME_DIR/voa/`
/// - `$XDG_DATA_HOME/voa/`
/// - the `./voa/` directory in each directory defined in `$XDG_DATA_DIRS`
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LoadPaths {
    /// Load paths for "system mode"
    System,

    /// Load paths for "user mode"
    User,
}

/// Access to the "File Hierarchy for the Verification of OS Artifacts (VOA)".
///
/// [`Voa`] provides file access to the verifiers stored in one or more VOA hierarchies.
/// Depending on the calling system user, a set of system-wide or user + system-wide load paths is
/// used. This access is agnostic to the cryptographic technology later using the verifiers.
pub struct Voa {
    load_paths: LoadPaths,
}

impl Voa {
    /// Initialize a Voa object, based on a set of load paths in either system mode or user mode.
    pub fn new(load_paths: LoadPaths) -> Self {
        Self { load_paths }
    }

    fn load_paths(&self) -> Vec<PathBuf> {
        match self.load_paths {
            LoadPaths::System => LOAD_PATHS_SYSTEM_MODE.iter().map(Into::into).collect(),
            LoadPaths::User => unimplemented!(),
        }
    }

    /// Find verifiers in all available VOA hierarchies.
    ///
    /// Detection of verifiers is based on the provided [`Os`], [`Purpose`], [`Context`] and
    /// [`Technology`]. Finds all available verifier files and emits warnings for all files and
    /// directories that cannot be used.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), voa_core::Error> {
    /// use voa_core::{Context, LoadPaths, Mode, Os, Purpose, Role, Technology, Voa};
    ///
    /// let voa = Voa::new(LoadPaths::System); // FIXME
    ///
    /// let verifiers = voa.load(
    ///     Os::new("arch".into(), None, None, None, None),
    ///     Purpose::new(Role::Packages, Mode::ArtifactVerifier),
    ///     Context::Default,
    ///     Technology::OpenPGP,
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// TODO: How should the `trust_anchor` parameter work in this API?
    ///  (At least for some callers it would probably be convenient not to pass it at all, and
    ///  get the combination of basic verifiers and trust anchors back.)
    ///
    /// TODO: Should the return type have more specialized lookup functionality?
    ///   (e.g. grouped by common filename, or organized by `trust_anchor` value?)
    pub fn load(
        &self,
        os: Os,
        purpose: Purpose,
        context: Context,
        technology: Technology,
    ) -> Vec<Verifier> {
        let mut certs = vec![];

        for load_path in self.load_paths() {
            debug!("Looking for signature verifiers in the load path {load_path:?}");

            let checked_path = match CheckedVerifierSourcePath::new(VerifierSourcePath {
                load_path: load_path.clone(),
                os: os.clone(),
                purpose,
                context: context.clone(),
                technology,
            }) {
                Ok(checked_path) => checked_path,
                Err(err) => {
                    warn!("Error while checking load path {load_path:?}: {err:?}",);
                    continue;
                }
            };

            // This has been checked to be legal according to VOA symlinking rules, and a directory
            let source_path = &checked_path.canonicalized_target;

            trace!("Loading from VOA path {source_path:?}");

            let dir = match read_dir(source_path) {
                Ok(dir) => dir,
                Err(err) => {
                    trace!("⤷ Can't read path as a directory {err:?}");
                    continue; // try next load path
                }
            };

            for res in dir {
                let entry = match res {
                    Ok(entry) => entry,
                    Err(err) => {
                        warn!("⤷ Cannot get directory entry:\n{err}");
                        continue;
                    }
                };

                let Ok(file_type) = entry.file_type() else {
                    warn!("⤷ Cannot get file type of directory entry {entry:?}");
                    continue;
                };

                let path = if file_type.is_file() {
                    &entry.path()
                } else if file_type.is_symlink() {
                    match read_link(entry.path()) {
                        // FIXME: how does this interact with chains of symlinks? Does it process
                        // them at once? If so, we still need to check legality of each intermediate
                        // hop!
                        Ok(path) => {
                            if path.as_path().to_str() == Some("/dev/null") {
                                // Individual _signature verifiers_ may be masked using
                                // a symlink to `/dev/null`, independent of
                                // [technology].

                                unimplemented!("FIXME: handle masking")
                            } else {
                                // Check that path is legal

                                // TODO: This check should actually allow links into some of the
                                // other current load paths (but not all of them!) [...]

                                let Ok(canon) = path.canonicalize() else {
                                    unimplemented!()
                                };

                                // FIXME: [..] However, symlinks can be used in the VOA
                                // hierarchy to point to files or directories below one
                                // of the [load paths] in descending priority.
                                // Symlinks to files or directories below ephemeral load
                                // paths (i.e. `/run/voa/` and `$XDG_RUNTIME_DIR/voa/`)
                                // are prohibited, as they could lead to dangling
                                // references. [..]
                                if self.load_paths().iter().any(|p| canon.starts_with(p)) {
                                    &entry.path()
                                } else {
                                    trace!("Illegal symlink target in {entry:?}");
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            warn!("⤷ Cannot get information on target of symlink {entry:?}: {e:?}");
                            continue;
                        }
                    }
                } else {
                    warn!("⤷ Unexpected file type {file_type:?} for entry {entry:?}");
                    continue;
                };

                trace!("Checking verifier file {path:?}");

                // FIXME: resolve further symlinks here?

                if path.is_file() {
                    certs.push(Verifier {
                        path: checked_path.verifier_source_path.clone(),
                        filename: entry
                            .file_name()
                            .to_str()
                            .expect("utf8 problem")
                            .to_string(), // FIXME!
                    });
                } else {
                    trace!("⤷ Verifier path is not a file {path:?}");
                }
            }
        }

        certs
    }
}
