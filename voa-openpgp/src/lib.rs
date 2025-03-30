//! A VOA technology backend for OpenPGP based signature verification.
//!
//! DO NOT USE IN PRODUCTION: This backend is still in an early experimental development stage.
//!
//! For specification draft see: <https://github.com/uapi-group/specifications/pull/134>

use std::{
    fmt::{Debug, Formatter},
    ops::Add,
    path::Path,
    time::SystemTime,
};

use pgp::Signature;
use rpgpie::certificate::{Certificate, Checked};
use voa_core::{Context, OpaqueVerifier, Os, Purpose, Technology, Voa};

const FILE_ENDING: &str = ".openpgp";

/// Fingerprint of the certificate (i.e. the primary key fingerprint), as lower-case hex string
///
/// TODO: Move to rpgpie?
fn fingerprint(cert: &Certificate) -> String {
    hex::encode(cert.fingerprint().as_bytes())
}

/// An OpenPGP certificate for "Verification of OS Artifacts (VOA)"
pub struct OpenPGPCert {
    /// An OpenPGP Certificate that is synthesized from the data in `sources` below
    certificate: Certificate,

    /// Opaque verifiers, loaded from the filesystem.
    ///
    /// There may be multiple sources for one OpenPGP certificate, if multiple files contain data
    /// about one common Certificate (as defined by a shared primary key fingerprint).
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

        writeln!(f, "OpenPGP certificate {}", self.fingerprint())?;

        writeln!(f, "  Identities:")?;
        for user in checked.user_ids() {
            // Only show user ids that have some self-signature
            // TODO: consider revocations (in rpgpie)
            if !user.signatures.is_empty() {
                writeln!(f, "  - {}", user.id.id()).expect("TODO: panic message");
            }
        }

        // TODO: get creation time
        // TODO: get revocation/expiry status
        // TODO: get algorithm (via public params)

        writeln!(f)?;

        let verifiers = checked.valid_signing_capable_component_keys_at(&SystemTime::now().into());

        if !verifiers.is_empty() {
            writeln!(f, "  Valid signature verifiers:")?;

            for verifier in verifiers {
                let componentkey = verifier.as_componentkey();

                writeln!(
                    f,
                    "  * {:?} (created {:?})",
                    componentkey.fingerprint(),
                    componentkey.created_at()
                )?;

                // TODO: get revocation/expiry status
                // TODO: get algorithm (via public params)
            }
        } else {
            writeln!(f, "  No valid signature verifiers.")?;
        }

        writeln!(f)?;
        writeln!(
            f,
            "  Source(s):\n{}",
            self.sources
                .iter()
                .map(OpaqueVerifier::full_filename)
                .map(|s| format!("  - {}", s.to_string_lossy()))
                .collect::<Vec<_>>()
                .join("\n")
        )?;

        Ok(())
    }
}

impl OpenPGPCert {
    /// Fingerprint of the certificate (i.e. the primary key fingerprint), as lower-case hex string
    fn fingerprint(&self) -> String {
        fingerprint(&self.certificate)
    }

    /// Very basic signature verification:
    ///
    /// This checks if `self` has issued a cryptographically
    /// valid Signature (as listed in `sigs`) for `file`.
    ///
    /// TODO: Don't use the rPGP `Signature` type in this interface.
    ///  (Take a `&[u8]` with raw binary/armored signature data instead?)
    ///
    /// TODO: Currently only prints results on stdout, return structured information.
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
///
/// TODO: Should this struct include trust evaluations?
pub struct CertificateDirectoryOpenPGP<'a>(Voa<'a>);

impl<'a> CertificateDirectoryOpenPGP<'a> {
    pub fn new(load_paths: &'a [&'a str]) -> CertificateDirectoryOpenPGP<'a> {
        Self(Voa::new(load_paths))
    }

    pub fn load(&self, os: Os, purpose: Purpose, context: Context) -> Vec<OpenPGPCert> {
        let opaque = self.0.load(os, purpose, context, Technology::OpenPGP);

        // TODO: If we obtained different versions of the same certificate, merge them!
        //
        // This can e.g. prevent erroneously using a non-revoked copy of a certificate if we have a
        // revocation in a different copy of the certificate.
        //
        // In such cases, the `sources` field will contain the set of source certificate data files.

        let openpgp_certs = opaque
            .into_iter()
            .filter_map(|opaque| {
                log::trace!("Processing VOA folder {:?}", opaque.source_path());

                // TODO: Encode the `Technology` at the type level in OpaqueVerifierData?
                if opaque.source_path().technology() != Technology::OpenPGP {
                    // This should never happen
                    log::error!(
                        "Unexpected technology {:?} in {}, skipping.",
                        opaque.source_path().technology(),
                        opaque.full_filename().to_string_lossy()
                    );
                    return None;
                }

                let Ok(certificate) = Certificate::try_from(opaque.data()) else {
                    log::warn!(
                        "Failed to deserialize OpenPGP certificate {}, skipping",
                        opaque.full_filename().to_string_lossy(),
                    );
                    return None;
                };

                let expected_filename = fingerprint(&certificate).add(FILE_ENDING);

                // The filename must match the certificate fingerprint
                let source_filename = opaque.filename();
                if source_filename != expected_filename {
                    log::warn!(
                        "Filename {:?} doesn't match expectation ({}), skipping",
                        source_filename,
                        expected_filename
                    );
                    return None;
                }

                Some(OpenPGPCert {
                    certificate,
                    sources: vec![opaque],
                })
            })
            .collect();

        openpgp_certs
    }
}
