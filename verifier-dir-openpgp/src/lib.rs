use pgp::Signature;
use rpgpie::certificate::{Certificate, Checked};
use std::fmt::{Debug, Formatter};
use std::ops::Add;
use std::path::Path;
use std::time::SystemTime;
use uapi_verifier_directory::{OpaqueVerifier, Technology, VerifierDirectory};

const FILE_ENDING: &str = ".openpgp";

/// An OpenPGP certificate for "Verification of Distribution Artifacts"
pub struct OpenPGPCert {
    /// An OpenPGP Certificate that is synthesized from the data in `sources` below
    certificate: Certificate,

    /// Opaque source verifiers, loaded from the filesystem.
    ///
    /// There may be multiple sources for one OpenPGP certificate, if the files contain data about
    /// the same Certificate (as detected by a shared primary key fingerprint).
    sources: Vec<OpaqueVerifier>,
}

impl Debug for OpenPGPCert {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // FIXME:
        //  This is only a very rough draft of representing an OpenPGP certificate
        //  specifically in the context of data signature verification.
        //  The goal is to get a first sense of what data would be good to show,
        //  and which of it is easily available from rpgpie.

        let checked = Checked::from(&self.certificate);

        writeln!(f, "Certificate {}", self.fingerprint())?;

        // TODO: get creation time
        // TODO: get revocation/expiry status
        // TODO: get algorithm (via public params)

        for verifier in checked.valid_signing_capable_component_keys_at(&SystemTime::now().into()) {
            let ckey = verifier.as_componentkey();

            writeln!(f, "  valid signature verifiers:")?;
            writeln!(f, "  * {:?}", ckey.fingerprint())?;
            writeln!(f, "  created {:?}", ckey.created_at())?;

            // TODO: get revocation/expiry status
            // TODO: get algorithm (via public params)
        }

        for user in checked.user_ids() {
            // Only show user ids that have some self-signature
            // TODO: consider revocations (in rpgpie)
            if !user.signatures.is_empty() {
                writeln!(f, " {}", user.id.id()).expect("TODO: panic message");
            }
        }

        writeln!(f)?;
        writeln!(
            f,
            " {:?}",
            self.sources
                .iter()
                .map(OpaqueVerifier::source)
                .collect::<Vec<_>>()
        )?;
        writeln!(f)?;

        Ok(())
    }
}

impl OpenPGPCert {
    /// Fingerprint of the certificate (i.e. the primary key fingerprint), as lower-case hex string
    fn fingerprint(&self) -> String {
        hex::encode(self.certificate.fingerprint().as_bytes())
    }

    /// Dummy package verification function.
    ///
    /// FIXME: Currently only prints results on stdout, return structured information.
    pub fn verify(&self, file: &Path, sigs: &[Signature]) {
        let checked = Checked::from(&self.certificate);

        let data = std::fs::read(file).expect("read package data");

        for verifier in checked.valid_signing_capable_component_keys_at(&SystemTime::now().into()) {
            for sig in sigs {
                if verifier.verify(sig, &data).is_ok() {
                    println!(
                        "Good signature for {:?} by signer {}, issued at {:?}",
                        file,
                        self.fingerprint(),
                        sig.created().unwrap()
                    )
                }
            }
        }
    }
}

/// An OpenPGP specific view onto a VerifierDirectory
pub struct CertificateDirectoryOpenPGP<'a> {
    dir: VerifierDirectory<'a>,
}

impl<'a> CertificateDirectoryOpenPGP<'a> {
    pub fn new(roots: &'a [&'a str]) -> CertificateDirectoryOpenPGP<'a> {
        let dir = VerifierDirectory::new(roots);
        Self { dir }
    }

    pub fn load(&self, distribution: &str, purpose: &str, context: &str) -> Vec<OpenPGPCert> {
        let certs = self
            .dir
            .load(distribution, purpose, context, Technology::OpenPGP);

        let openpgp_certs = certs
            .into_iter()
            .filter_map(|opaque| {
                log::debug!("Processing {:?}", opaque.source());

                // TODO: should the `Technology` be encoded at the type level in OpaqueVerifierData?
                if opaque.technology() != Technology::OpenPGP {
                    log::warn!(
                        "Unexpected technology {:?} in {:?}, skipping",
                        opaque.technology(),
                        opaque.source()
                    );
                    return None;
                }

                let Ok(certificate) = Certificate::try_from(opaque.data()) else {
                    log::warn!(
                        "Failed to deserialize Certificate from {:?}, skipping",
                        opaque.source()
                    );
                    return None;
                };

                let fingerprint = hex::encode(certificate.fingerprint().as_bytes()); // TODO: move to rpgpie?
                let expected_filename = fingerprint.add(FILE_ENDING);

                // The filename must match the certificate fingerprint
                let source_filename = opaque.file_name();
                if source_filename.as_deref() != Some(&expected_filename) {
                    log::warn!(
                        "Filename {:?} doesn't match expectation ({}), skipping",
                        source_filename,
                        expected_filename
                    );
                    return None;
                }

                // TODO: If we obtained different versions of the same certificate, merge them
                //
                // This can e.g. prevent erroneously using a non-revoked copy of a certificate if we have a
                // revocation in a different copy of the certificate.
                //
                // In such cases, the `sources` field will contain the set of source certificate data files.

                Some(OpenPGPCert {
                    certificate,
                    sources: vec![opaque],
                })
            })
            .collect();

        openpgp_certs
    }
}
