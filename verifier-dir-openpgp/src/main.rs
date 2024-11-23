use uapi_certificate_directory_openpgp::CertificateDirectoryOpenPGP;

const ROOTS_TEST: &[&str] = &["/tmp/pki1/", "/tmp/pki2/"];

/// Toy usage of CertificateDirectoryOpenPGP, e.g. based on data seeded with `testkey_loader.sh`
fn main() {
    env_logger::init();

    let cdo = CertificateDirectoryOpenPGP::new(ROOTS_TEST);
    let certs = cdo.load("arch", "packages", "default");

    certs.iter().for_each(|c| eprintln!("{:#?}", c));

    // todo, validate packages
    // https://ftp.agdsn.de/pub/mirrors/archlinux/core/os/x86_64/

    // certs, packager+main signing: https://gitlab.archlinux.org/archlinux/archlinux-keyring/-/releases/20241015
}
