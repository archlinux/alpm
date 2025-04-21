//! "File Hierarchy for the Verification of OS Artifacts (VOA)"
//!
//! A mechanism for technology-agnostic storage and retrieval of` signature verifiers.
//! For specification draft see: <https://github.com/uapi-group/specifications/pull/134>

use std::{
    fmt::{Debug, Formatter},
    path::PathBuf,
};

/// Load paths for "system mode" operation of VOA.
///
/// Each load path can contain a VOA hierarchy of verifier files.
/// Earlier load paths have precedence over later entries (in some technologies).
///
/// NOTE: Depending on the technology, multiple versions of the same verifier will either "shadow"
/// one another, or get merged into one coherent view that represents the totality of available
/// information about the verifier.
///
/// Shadowing/merging is technology-specific and must be handled in the technology layer.
///
/// VOA expects that filenames are a strong identifier, which signal if two verifier files deal
/// with "the same" logical verifier. Verifiers in different load paths can be identified as
/// related by their filenames.
///
/// (E.g. OpenPGP certificates must be stored using filenames based on their fingerprint.)
///
/// The technology-specific layers are expected to warn or error when a verifier filename is
/// inconsistent with the contained verifier.
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
/// provided by `os-release`.
///
/// Anb Os identifier consists of (up to) five parts.
///
///  - id: name of OS (e.g. arch or debian)
///  - version_id: the version of the OS (e.g. 1.0.0 or 24.12)
///  - variant_id: the variant of the OS (e.g. server or workstation)
///  - image_id: the image of an OS (e.g. cashier-system)
///  - image_version: version of the image (e.g. 1.0.0 or 24.12)
///
///  Each part can consist of the characters "0–9", "a–z", ".", "_" and "-".
#[derive(Clone, Debug, PartialEq)]
pub struct Os {
    id: String,
    version_id: Option<String>,
    variant_id: Option<String>,
    image_id: Option<String>,
    image_version: Option<String>,
}

impl Os {
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
    fn path(&self) -> String {
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

/// A Purpose combines a [Role] and a [Mode]. The combination reflects one directory layer in the
/// VOA file hierarchy.
///
/// Purpose paths have values such as: `packages`, `trust-anchor-packages`, `repository-metadata`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Purpose {
    role: Role,
    mode: Mode,
}

impl Purpose {
    pub fn new(role: Role, mode: Mode) -> Self {
        Self { role, mode }
    }

    fn path(&self) -> String {
        let base = match self.role {
            Role::Packages => "packages",
            Role::RepositoryMetadata => "repository-metadata",
            Role::Image => "image",
        };

        match self.mode {
            Mode::TrustAnchor => format! { "trust-anchor-{base}" },
            Mode::ArtifactVerifier => base.to_string(),
        }
    }
}

/// A Role acts as a trust domain that is associated with one set of verifiers.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum Role {
    /// For verifying signatures for packages
    Packages,

    /// For verifying signatures for repository metadata
    RepositoryMetadata,

    /// For verifying signatures for OS images
    Image,
}

/// The Mode (of a [Purpose]) distinguishes between direct artifact verifiers and trust anchors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    ArtifactVerifier,
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
#[non_exhaustive]
pub enum Context {
    Default,

    // TODO: enforce limitation to legal characters
    Specified(String),
}

impl Context {
    fn path(&self) -> &str {
        match self {
            Self::Default => "default",
            Self::Specified(context) => context,
        }
    }
}

/// Technology-specific backends implement the logic for each supported verification technology
/// in VOA.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum Technology {
    OpenPGP,
    SSH,
}

impl Technology {
    fn path(&self) -> &str {
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
    fn path(&self) -> PathBuf {
        self.load_path
            .join(self.os.path())
            .join(self.purpose.path())
            .join(self.context.path())
            .join(self.technology.path())
    }

    pub fn os(&self) -> &Os {
        &self.os
    }

    pub fn purpose(&self) -> Purpose {
        self.purpose
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn technology(&self) -> Technology {
        self.technology
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

    /// Filename of the verifier file, in `path`
    filename: String,
}

impl Debug for OpaqueVerifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "OpaqueVerifier [{} bytes]", self.verifier_data.len())?;
        writeln!(f, "Path: {:#?}", self.path)?;
        writeln!(f, "Filename: {:?}", self.filename)?;

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
    System,
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
            log::trace!(
                "Looking for signature verifiers in the load path '{:?}'",
                load_path
            );

            let path = VerifierSourcePath {
                load_path,
                os: os.clone(),
                purpose,
                context: context.clone(),
                technology,
            };

            let source_path = path.path();

            log::trace!("Opening VOA path {:?}", source_path);

            let res = std::fs::read_dir(source_path);
            let Ok(dir) = res else {
                log::trace!("  Can't read path as a directory {:?}", res);
                continue; // try next load path
            };

            for entry in dir {
                match entry {
                    Ok(file) => {
                        log::trace!("Loading verifier file {:?}", file);

                        match std::fs::read(file.path()) {
                            Ok(verifier_data) => {
                                certs.push(OpaqueVerifier {
                                    verifier_data,
                                    path: path.clone(),
                                    filename: file.file_name().to_str().unwrap().to_string(), // FIXME!
                                });
                            }
                            Err(err) => log::debug!("  Error while loading file {err}"),
                        }
                    }
                    Err(err) => log::debug!("  DirEntry error {err}"),
                }
            }
        }

        certs
    }
}
