use rpgpie::certificate::{Certificate, Checked};
use std::fmt::{Debug, Formatter};
use std::time::SystemTime;
use uapi_certificate_directory::{CertificateDirectory, OpaqueCertData};

const TECHNOLOGY_OPENPGP: &str = "openpgp";

/// An OpenPGP certificate for "Verification of Distribution Artifacts"
pub struct OpenPGPCert {
    certificate: Certificate,
    sources: Vec<OpaqueCertData>, // source file(s) [may be multiple, if the certificate got merged]
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

        for sv in checked.valid_signing_capable_component_keys_at(&SystemTime::now().into()) {
            let ckp = sv.as_componentkey();

            writeln!(f, "  valid signature verification component keys:")?;
            writeln!(f, "  * {:?}", ckp.fingerprint())?;
            writeln!(
                f,
                // "  {}, created {:?}",
                "  created {:?}",
                // sk.public_params(),
                ckp.created_at()
            )?;
        }

        for user in checked.user_ids() {
            // don't show user ids that aren't self-bound
            if !user.signatures.is_empty() {
                writeln!(f, " {}", user.id.id()).expect("TODO: panic message");
            }
        }

        writeln!(f)?;
        writeln!(
            f,
            " {:?}",
            self.sources.iter().map(|x| x.source()).collect::<Vec<_>>()
        )?;
        writeln!(f)?;

        Ok(())
    }
}

impl OpenPGPCert {
    fn fingerprint(&self) -> String {
        hex::encode(self.certificate.fingerprint().as_bytes())
    }
}

/// An OpenPGP specific view onto a CertificateDirectory
pub struct CertificateDirectoryOpenPGP<'a> {
    cd: CertificateDirectory<'a>,
}

impl<'a> CertificateDirectoryOpenPGP<'a> {
    pub fn new(roots: &'a [&'a str]) -> CertificateDirectoryOpenPGP<'a> {
        let cd = CertificateDirectory::new(roots);
        Self { cd }
    }

    pub fn load(
        &self,
        distribution: &str,
        role: &str,
        context: &str,
    ) -> Result<Vec<OpenPGPCert>, std::io::Error> {
        let certs = self
            .cd
            .load(distribution, role, context, TECHNOLOGY_OPENPGP)?;

        // TODO: If different copies of the same certificate exist in this directory, merge them
        //
        // This can e.g. prevent erroneously using a non-revoked copy of a certificate if we have a
        // revocation in a different copy of the certificate.
        //
        // In such cases, the `sources` field will contain the set of source certificate data files.

        let openpgp_certs = certs
            .into_iter()
            .map(|loaded| {
                let certificate = Certificate::try_from(loaded.data()).expect("FIXME");

                OpenPGPCert {
                    certificate,
                    sources: vec![loaded],
                }
            })
            .collect();

        Ok(openpgp_certs)
    }
}
