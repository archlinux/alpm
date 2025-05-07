//! File Hierarchy for the Verification of OS Artifacts (VOA)
//!
//! A mechanism for signing technology-agnostic storage and retrieval of signature verifiers.
//! For specification draft see: <https://github.com/uapi-group/specifications/pull/134>
//!
//! The VOA hierarchy acts as structured storage for files that contain "signature verifiers"
//! (such as OpenPGP certificates, aka "public keys").
//!
//! At the top level of the hierarchy, a set of "load paths" can exist and contain different sets
//! of verifier files. A VOA access library provides an abstract unified view of this set of the
//! signature verifiers in that set of load paths.
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
//! For example, OpenPGP certificates must be stored using filenames based on their fingerprint.
//!
//! Signing technology-specific libraries will warn or error when a verifier filename is
//! inconsistent with the contained verifier.

#![warn(missing_docs)]

use std::{
    fmt::{Debug, Formatter},
    path::{Path, PathBuf},
};

use log::{debug, trace, warn};

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
    /// Create a new operating system specifier
    ///
    /// `id`: Name of the OS (e.g. arch or debian)
    /// `version_id`: The version of the OS (e.g. 1.0.0 or 24.12)
    /// `variant_id`: The variant of the OS (e.g. server or workstation)
    /// `image_id`: The image of an OS (e.g. cashier-system)
    /// `image_version`: Version of the image (e.g. 1.0.0 or 24.12)
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
    fn path_segment(&self) -> String {
        let distro = format!(
            "{}:{}:{}:{}:{}",
            &self.id,
            self.version_id.as_deref().unwrap_or(""),
            self.variant_id.as_deref().unwrap_or(""),
            self.image_id.as_deref().unwrap_or(""),
            self.image_version.as_deref().unwrap_or(""),
        );

        distro.trim_end_matches(':').to_string()
    }
}

/// A `Purpose` combines a [Role] and a [Mode].
/// It describes in what context the signature verifiers in that directory tree are used.
///
/// The combination of [Role] and [Mode] reflects one directory layer in the VOA file hierarchy.
/// Purpose paths have values such as: `packages`, `trust-anchor-packages`, `repository-metadata`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Purpose {
    role: Role,
    mode: Mode,
}

impl Purpose {
    /// Create a new `Purpose` object, which combines a [Role] and a [Mode].
    /// A `Purpose` describes in what context the signature verifiers are used.
    pub fn new(role: Role, mode: Mode) -> Self {
        Self { role, mode }
    }

    fn path_segment(&self) -> String {
        let base = match self.role {
            Role::Packages => "packages",
            Role::RepositoryMetadata => "repository-metadata",
            Role::Image => "image",
        };

        match self.mode {
            Mode::TrustAnchor => format!("trust-anchor-{base}"),
            Mode::ArtifactVerifier => base.to_string(),
        }
    }
}

/// A Role acts as a trust domain that is associated with one set of verifiers.
///
/// A [Role] is always combined with a [Mode]. The combination of both forms a [Purpose].
/// E.g. [Role::Packages] combined with [Mode::TrustAnchor] specify the purpose path
/// `trust-anchor-packages`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Role {
    /// For verifying signatures for packages
    Packages,

    /// For verifying signatures for repository metadata
    RepositoryMetadata,

    /// For verifying signatures for OS images
    Image,
}

/// The Mode of a [Purpose] distinguishes between direct artifact verifiers and trust anchors.
///
/// A [Mode] is always combined with a [Role]. The combination of both forms a [Purpose].
/// E.g. [Role::Packages] combined with [Mode::TrustAnchor] specify the purpose path
/// `trust-anchor-packages`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    /// `ArtifactVerifier`s are used directly for the validation of signatures on artifacts
    ArtifactVerifier,

    /// `TrustAnchor`s are used to ascertain the authenticity of [Mode::ArtifactVerifier]s.
    TrustAnchor,
}

/// The context layer allows defining specific verifiers for a particular context within a
/// [Purpose].
///
/// An example for context is the name of a specific software repository when certificates are
/// used in the context of the packages purpose (e.g. "core").
///
/// If no specific context is required, the context `Default` must be used.
#[derive(Clone, Debug, PartialEq)]
pub enum Context {
    /// The default context
    Default,

    /// Defines a specific [Context] for verifiers within an [Os] and [Purpose]
    ///
    /// TODO: enforce limitation to legal characters
    Specified(String),
}

impl Context {
    fn path_segment(&self) -> &str {
        match self {
            Self::Default => "default",
            Self::Specified(context) => context,
        }
    }
}

/// Technology-specific backends implement the logic for each supported verification technology
/// in VOA.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Technology {
    /// The OpenPGP signature verification technology
    OpenPGP,

    /// The SSH signature verification technology
    SSH,
}

impl Technology {
    fn path_segment(&self) -> &str {
        match self {
            Self::OpenPGP => "openpgp",
            Self::SSH => "ssh",
        }
    }
}

/// Specification of a path in a VOA structure at the "leaf" level (where verifier files are
/// stored).
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
    fn path(&self) -> PathBuf {
        self.load_path
            .join(self.os.path_segment())
            .join(self.purpose.path_segment())
            .join(self.context.path_segment())
            .join(self.technology.path_segment())
    }

    /// The [Os] uniquely identifies the Operating System of this [VerifierSourcePath].
    pub fn os(&self) -> &Os {
        &self.os
    }

    /// The [Purpose] specifies both the [Role] and the [Mode] of the verifiers in this
    /// [VerifierSourcePath].
    pub fn purpose(&self) -> Purpose {
        self.purpose
    }

    /// The [Context] may define a specific namespace for verifiers within an [Os] and [Purpose].
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// The signature verification technology of this [VerifierSourcePath].
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
    // Check that VerifierSourcePath forms a legal path on the local filesystem, and all involved
    // symlinks (if any) conform to VOA's symlink restrictions
    fn new(verifier_source_path: VerifierSourcePath) -> std::io::Result<Self> {
        // Canonicalized base load path
        // (any potential internal symlinks of this top level "load_path" are not checked)
        let base_path = verifier_source_path.load_path.canonicalize()?;

        // Append a segment to a path, and check that the resulting path conforms to the symlinking
        // constraints laid out in the VOA spec
        //
        // FIXME: This closure needs a list of (canonicalized) load paths that this one may link to
        let append = |p: &Path, segment: &str| -> std::io::Result<PathBuf> {
            let mut buf = p.join(segment);
            if buf.is_symlink() {
                // Check that the symlink-canonicalized path is acceptable, including this segment
                let canon = buf.canonicalize()?;

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
                if !canon.starts_with(&base_path) {
                    trace!(
                        "CheckedVerifierSourcePath::new illegal path segment {segment:?} following {buf:?}"
                    );

                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "{segment:?} is not valid following {buf:?}",
                    ));
                }

                buf = canon;
            }

            if !buf.is_dir() {
                trace!("CheckedVerifierSourcePath::new {buf:?} is not a directory");

                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotADirectory,
                    "{buf:?} is not a directory",
                ));
            }

            Ok(buf)
        };

        let mut path = append(&base_path, &verifier_source_path.os.path_segment())?;
        path = append(&path, &verifier_source_path.purpose.path_segment())?;
        path = append(&path, verifier_source_path.context.path_segment())?;
        path = append(&path, verifier_source_path.technology.path_segment())?;

        trace!("CheckedVerifierSourcePath::new canonicalized path: {path:?}");

        Ok(Self {
            verifier_source_path,
            canonicalized_target: path,
        })
    }
}

/// A signature verifier, loaded as an opaque blob of data.
///
/// Depending on the technology, this may represent, e.g.:
/// - an individual, loose verifier
/// - a certificate complete with its trust chain
/// - a set of individual verifiers in one shared data structure
pub struct OpaqueVerifier {
    /// Opaque data representing the verifier
    verifier_data: Vec<u8>,

    /// Specification of the path from which the verifier was loaded
    path: VerifierSourcePath,

    /// Filename of the verifier file, in [`OpaqueVerifier::path`]
    filename: String,
}

impl Debug for OpaqueVerifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "OpaqueVerifier [{} bytes]", self.verifier_data.len())?;
        writeln!(f, "Path: {:#?}", self.path)?;
        writeln!(f, "Filename: {}", self.filename)?;

        Ok(())
    }
}

impl OpaqueVerifier {
    /// The raw data of this verifier
    pub fn data(&self) -> &[u8] {
        &self.verifier_data
    }

    /// The source path of this verifier
    pub fn source_path(&self) -> &VerifierSourcePath {
        &self.path
    }

    /// The filename (excluding the path)
    pub fn filename(&self) -> &str {
        &self.filename
    }

    /// The filename complete with the full path
    pub fn full_filename(&self) -> PathBuf {
        let mut file = self.source_path().path();
        file.push(&self.filename);

        file
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

/// A Voa object, which is based on a set of load paths.
///
/// Voa acts as an accessor to certificates stored in the filesystem.
/// It is agnostic to the signing technology, and handles all certificates as opaque objects.
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

    /// Load a set of (opaque) signature verifiers from the VOA hierarchy.
    ///
    /// Paths in a VerifierDirectory have the shape:
    /// LOAD_PATH/$os/$purpose/$context/$technology
    ///
    /// os: e.g. "arch"
    /// purpose: e.g. "trust-anchor-packages", "packages"
    /// context: e.g. "default"
    /// technology: e.g. "openpgp"
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
    ) -> Vec<OpaqueVerifier> {
        let mut certs = vec![];

        for load_path in self.load_paths() {
            debug!("Looking for signature verifiers in the load path {load_path:?}");

            let res = CheckedVerifierSourcePath::new(VerifierSourcePath {
                load_path: load_path.clone(),
                os: os.clone(),
                purpose,
                context: context.clone(),
                technology,
            });

            let Ok(checked_path) = res else {
                warn!(
                    "Error while checking load path {load_path:?}: {:?}",
                    res.err().expect("error case")
                );

                continue;
            };

            // This has been checked to be legal according to VOA symlinking rules, and a directory
            let source_path = &checked_path.canonicalized_target;

            trace!("Loading from VOA path {source_path:?}");

            let res = std::fs::read_dir(source_path);
            let Ok(dir) = res else {
                trace!("⤷ Can't read path as a directory {res:?}");
                continue; // try next load path
            };

            for res in dir {
                match res {
                    Ok(entry) => {
                        if let Ok(file_type) = entry.file_type() {
                            let mut try_load = |path: &Path| {
                                trace!("Loading verifier file {path:?}");

                                match std::fs::read(path) {
                                    Ok(verifier_data) => {
                                        certs.push(OpaqueVerifier {
                                            verifier_data,
                                            path: checked_path.verifier_source_path.clone(),
                                            filename: entry
                                                .file_name()
                                                .to_str()
                                                .expect("utf8 problem")
                                                .to_string(), // FIXME!
                                        });
                                    }
                                    Err(err) => trace!("⤷ Error while loading file {err}"),
                                }
                            };

                            if file_type.is_file() {
                                try_load(&entry.path());
                            } else if file_type.is_symlink() {
                                match std::fs::read_link(entry.path()) {
                                    Ok(path) => {
                                        if path.as_path().to_str() == Some("/dev/null") {
                                            // Individual _signature verifiers_ may be masked using
                                            // a symlink to `/dev/null`, independent of
                                            // [technology].

                                            unimplemented!("FIXME: handle masking")
                                        } else {
                                            // FIXME: check that `path` is legal:
                                            // [..] However, symlinks can be used in the VOA
                                            // hierarchy to point to files or directories below one
                                            // of the [load paths] in descending priority.
                                            // Symlinks to files or directories below ephemeral load
                                            // paths (i.e. `/run/voa/` and `$XDG_RUNTIME_DIR/voa/`)
                                            // are prohibited, as they could lead to dangling
                                            // references. [..]

                                            try_load(&entry.path());
                                        }
                                    }
                                    Err(e) => trace!("⤷ Error for symlink {entry:?}: {e:?}"),
                                }
                            }
                        }
                    }
                    Err(err) => trace!("⤷ DirEntry error {err}"),
                }
            }
        }

        certs
    }
}
