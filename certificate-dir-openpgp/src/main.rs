use uapi_certificate_directory_openpgp::CertificateDirectoryOpenPGP;

const ROOTS_TEST: &[&str] = &["/tmp/pki1/", "/tmp/pki2/"];

/// Toy usage of CertificateDirectoryOpenPGP, e.g. based on data seeded with `testkey_loader.sh`
fn main() {
    env_logger::init();

    let cdo = CertificateDirectoryOpenPGP::new(ROOTS_TEST);
    let certs = cdo.load("arch", "packages", "default").expect("FIXME");

    certs.iter().for_each(|c| eprintln!("{:#?}", c));
}
