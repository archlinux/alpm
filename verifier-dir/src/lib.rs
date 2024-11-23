//! "Hierarchy for the Verification of Distribution Artifacts (VDA)"
//!
//! A specification for technology-agnostic storage and retrieval or signature verifiers.
//!
//! (Specification link pending.)
//!
//! (Also see https://github.com/uapi-group/specifications/issues/115 for initial discussion)

use std::path::PathBuf;

/// Earlier entries have precedence over later entries.
///
/// TODO: depending on the technology, we will want to either "shadow" one version with another,
///       or merge the data from different versions into one coherent view.
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

/// A signature verifier, loaded as an opaque blob of data.
///
/// Depending on the technology, this may represent, e.g.:
/// - an individual, loose verifier
/// - a certificate complete with its trust chain
/// - a set of individual verifiers in one shared data structure
pub struct OpaqueVerifier {
    data: Vec<u8>,
    source: PathBuf,
    technology: Technology,
}

impl OpaqueVerifier {
    /// The raw certificate data of this file
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// The source file path of this certificate data
    pub fn source(&self) -> &PathBuf {
        &self.source
    }

    pub fn file_name(&self) -> Option<String> {
        // FIXME: such panic, wow!
        Some(
            self.source
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        )
    }

    /// The technology of this certificate data
    pub fn technology(&self) -> Technology {
        self.technology
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
    pub fn new(roots: &'a [&'a str]) -> Self {
        Self { roots }
    }

    /// Load a set of (opaque) signature verifiers from the filesystem.
    ///
    /// Paths in a VerifierDirectory have the shape: ROOT/VDA/$distribution/purpose/$context/$technology
    ///
    /// $distribution: e.g. "arch"
    /// purpose: e.g. "trust-anchor-packages", "packages"
    /// $context: e.g. "default"
    /// $technology: e.g. "openpgp"
    pub fn load(
        &self,
        distribution: &str,
        purpose: &str,
        context: &str,
        technology: Technology,
    ) -> Vec<OpaqueVerifier> {
        let mut certs = vec![];

        // FIXME: don't error out (or panic) for most cases, just warn and proceed on most issues

        for root in self.roots {
            log::debug!("Looking for signature verifiers in root dir {root}");

            let path = PathBuf::from(root)
                .join(VDA)
                .join(distribution)
                .join(purpose)
                .join(context)
                .join(technology.path());

            log::trace!("opening path {:?}", path.to_str().unwrap_or("-"));

            if !path.is_dir() {
                log::trace!("  path is not a dir in this root");
                continue; // try next root
            }

            log::trace!("  path is a dir");

            if let Ok(dir) = std::fs::read_dir(path) {
                for entry in dir {
                    match entry {
                        Ok(file) => {
                            log::debug!("loading {:?}", file);

                            let source = file.path();

                            match std::fs::read(&source) {
                                Ok(data) => {
                                    certs.push(OpaqueVerifier {
                                        data,
                                        source,
                                        technology,
                                    });
                                }
                                Err(err) => log::debug!("  file loading error {err}"),
                            }
                        }
                        Err(err) => log::debug!("  dir entry error {err}"),
                    }
                }
            }
        }

        certs
    }
}
