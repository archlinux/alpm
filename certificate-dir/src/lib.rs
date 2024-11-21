use std::path::PathBuf;

const _ROOTS_DEFAULT: &[&str] = &["/etc/pki/", "/usr/local/share/pki/", "/run/pki/"];

/// Top level directory of the "Verification of Distribution Artifacts" hierarchy
const VDA: &str = "vda";

/// A certificate file, loaded as an opaque blob of data.
///
/// Depending on the technology, this may represent, e.g.:
/// - an individual, loose certificate
/// - a certificate complete with its trust chain
/// - a set of individual certificates in one shared data structure
pub struct OpaqueCertData {
    data: Vec<u8>,
    source: PathBuf,
}

impl OpaqueCertData {
    /// The raw certificate data of this file
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// The source file path of this certificate data
    pub fn source(&self) -> &PathBuf {
        &self.source
    }
}

/// A CertificateDirectory object, which is based on a set of root directories.
///
/// CertificateDirectory acts as an accessor to certificates stored in the filesystem.
/// It is agnostic to the signing technology, and handles all certificates as opaque object.
pub struct CertificateDirectory<'a> {
    roots: &'a [&'a str],
}

impl<'a> CertificateDirectory<'a> {
    /// Initialize a CertificateDirectory object, based on a set of root directories
    pub fn new(roots: &'a [&'a str]) -> Self {
        Self { roots }
    }

    /// Load a set of (opaque) certificates from the filesystem.
    ///
    /// Paths in a CertificateDirectory have the shape: ROOT/VDA/$distribution/$role/$context/$technology
    ///
    /// $distribution: e.g. "arch"
    /// $role: e.g. "trust-anchor-packages", "packages"
    /// $context: e.g. "default"
    /// $technology: e.g. "openpgp"
    pub fn load(
        &self,
        distribution: &str,
        role: &str,
        context: &str,
        technology: &str,
    ) -> Result<Vec<OpaqueCertData>, std::io::Error> {
        let mut certs = vec![];

        for root in self.roots {
            log::debug!("processing root {}", root);

            let mut path = PathBuf::from(root);
            path.push(VDA);
            path.push(distribution);
            path.push(role);
            path.push(context);
            path.push(technology);

            log::debug!("opening path {:?}", path.to_str().unwrap_or("-"));

            if path.is_dir() {
                log::debug!("path is a dir");

                let dir = std::fs::read_dir(path)?;
                for entry in dir {
                    match entry {
                        Ok(file) => {
                            log::debug!("loading {:?}", file);

                            let source = file.path();
                            let data = std::fs::read(&source).expect("FIXME");

                            certs.push(OpaqueCertData { data, source });
                        }
                        Err(err) => log::debug!("error {}", err),
                    }
                }
            } else {
                log::debug!("path is not a dir");
            }
        }

        Ok(certs)
    }
}
