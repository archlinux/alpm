//! "Hierarchy for the Verification of Distribution Artifacts (VDA)"
//!
//! A mechanism for technology-agnostic storage and retrieval or signature verifiers.
//!
//! (Specification link pending,
//! see <https://github.com/uapi-group/specifications/issues/115> for initial discussion)

use std::path::PathBuf;

/// Earlier entries have precedence over later entries.
///
/// NOTE: Depending on the technology, we will want to either "shadow" one version with another,
/// or merge the data from different versions into one coherent view.
/// Shadowing is technology-specific and must be handled in the technology layer.
///
/// We expect that the filename is a strong identifier that can also be used to check if two
/// verifiers are the same, between roots, based on their filename.
///
///E.g. OpenPGP certificates must have names based in fingerprints.
///
/// The technology-specific layers are expected to warn or error when filenames and their content
/// are inconsistent.
const _ROOTS_DEFAULT: &[&str] = &[
    "/etc/pki/",
    "/run/pki/",
    "/usr/local/share/pki/",
    "/usr/share/pki/",
];

/// Top level directory of the "Verification of Distribution Artifacts" hierarchy
const VDA: &str = "vda";

/// Version specifier, currently only version 1 of VDA is defined
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Version {
    V1,
}

impl Version {
    fn path(&self) -> &str {
        match self {
            Self::V1 => "v1",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Distribution {
    id: String,
    version_id: Option<String>,
    variant_id: Option<String>,
    image_id: Option<String>,
    image_version: Option<String>,
}

impl Distribution {
    pub fn new(
        id: String,
        version_id: Option<String>,
        variant_id: Option<String>,
        image_id: Option<String>,
        image_version: Option<String>,
    ) -> Self {
        Self {
            id,
            version_id,
            variant_id,
            image_id,
            image_version,
        }
    }

    /// A string representation of this Distribution specifier.
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

/// The current fixed default value for version (used to form verifier paths)
const VERSION: Version = Version::V1;

#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum Purpose {
    /// For verifying signatures for packages
    Packages,

    /// For verifying signatures for repository metadata
    RepositoryMetadata,

    /// For verifying signatures for installation media
    InstallationMedia,

    /// For verifying signatures for virtual machines
    VirtualMachines,

    /// For verifying signatures for updates to image-based machines
    ImageUpdate,
}

impl Purpose {
    fn path(&self, trust_anchor: bool) -> String {
        let base = match self {
            Self::Packages => "packages",
            Self::RepositoryMetadata => "repository-metadata",
            Self::InstallationMedia => "installation-media",
            Self::VirtualMachines => "virtual-machines",
            Self::ImageUpdate => "image-update",
        };

        match trust_anchor {
            true => format! { "trust-anchor-{base}" },
            false => base.to_string(),
        }
    }
}

/// The context layer allows defining specific verifiers for a particular context within a
/// distribution’s [Purpose].
///
/// An example for context is the name of a specific software repository when certificates are
/// used in the context of the packages purpose (e.g. "core").
///
/// If no specific context is required, the context `Default` must be used.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Context {
    Default,
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

/// Specification of a path in a VDA structure, at the "leaf" level, where verifier files are stored
#[derive(Clone, Debug, PartialEq)]
pub struct VerifierSourcePath {
    root: String,
    version: Version,
    distribution: Distribution,
    purpose: Purpose,
    trust_anchor: bool,
    context: Context,
    technology: Technology,
}

impl VerifierSourcePath {
    fn path(&self) -> PathBuf {
        PathBuf::from(&self.root)
            .join(VDA)
            .join(self.version.path())
            .join(self.distribution.path())
            .join(self.purpose.path(self.trust_anchor))
            .join(self.context.path())
            .join(self.technology.path())
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn distribution(&self) -> &Distribution {
        &self.distribution
    }

    pub fn purpose(&self) -> Purpose {
        self.purpose
    }

    pub fn trust_anchor(&self) -> bool {
        self.trust_anchor
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
    data: Vec<u8>,
    path: VerifierSourcePath,
    filename: String,
}

impl OpaqueVerifier {
    /// The raw certificate data of this file
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// The source file path of this certificate data
    pub fn source_path(&self) -> &VerifierSourcePath {
        &self.path
    }

    /// The filename, without the full path
    pub fn filename(&self) -> &str {
        &self.filename
    }

    /// The filename complete with the full path
    pub fn full_filename(&self) -> String {
        let mut file = self.source_path().path();
        file.push(&self.filename);

        file.to_str().unwrap().to_string() // FIXME: unwrap
    }
}

/// A VerifierDirectory object, which is based on a set of root directories.
///
/// VerifierDirectory acts as an accessor to certificates stored in the filesystem.
/// It is agnostic to the signing technology, and handles all certificates as opaque object.
pub struct VerifierDirectory<'a> {
    roots: &'a [&'a str],
}

impl<'a> VerifierDirectory<'a> {
    /// Initialize a VerifierDirectory object, based on a set of root directories
    ///
    /// TODO: Always passing the roots explicitly is definitely wrong here.
    ///       Should the roots always be implicit, and use the hardcoded `ROOTS_DEFAULT`?
    pub fn new(roots: &'a [&'a str]) -> Self {
        Self { roots }
    }

    /// Load a set of (opaque) signature verifiers from the filesystem.
    ///
    /// Paths in a VerifierDirectory have the shape:
    /// ROOT/VDA/$version/$distribution/purpose/$context/$technology
    ///
    /// $distribution: e.g. "arch"
    /// purpose: e.g. "trust-anchor-packages", "packages"
    /// $context: e.g. "default"
    /// $technology: e.g. "openpgp"
    ///
    /// TODO: How should the `trust_anchor` parameter work in this API?
    ///  (At least for some callers it would probably be convenient not to pass it at all, and
    ///  get the combination of basic verifiers and trust anchors back.)
    ///
    /// TODO: Should the return type have more specialized lookup functionality?
    ///   (e.g. grouped by common filename, or organized by `trust_anchor` value?)
    pub fn load(
        &self,
        distribution: Distribution,
        purpose: Purpose,
        trust_anchor: bool,
        context: Context,
        technology: Technology,
    ) -> Vec<OpaqueVerifier> {
        let mut certs = vec![];

        for root in self.roots {
            log::trace!("Looking for signature verifiers in root dir '{root}'");

            let path = VerifierSourcePath {
                root: root.to_string(),
                version: VERSION,
                distribution: distribution.clone(),
                purpose,
                trust_anchor,
                context: context.clone(),
                technology,
            };

            let source_path = path.path();

            log::trace!("Opening VDA path {:?}", source_path);

            let res = std::fs::read_dir(source_path);
            let Ok(dir) = res else {
                log::trace!("  Can't read path as a directory {:?}", res);
                continue; // try next root
            };

            for entry in dir {
                match entry {
                    Ok(file) => {
                        log::trace!("Loading verifier file {:?}", file);

                        match std::fs::read(file.path()) {
                            Ok(data) => {
                                certs.push(OpaqueVerifier {
                                    data,
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
