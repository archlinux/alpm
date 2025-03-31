//! HACK: This is a temporary testing hack to facilitate design of the VDA library APIs.
//! It should be dropped and replaced with proper integration tests.

use std::{fs::File, io::BufReader, path::Path};

use voa_core::{Context, LoadPaths, Mode, Os, Purpose, Role};
use voa_openpgp::CertificateDirectoryOpenPGP;

/// Toy usage of CertificateDirectoryOpenPGP, e.g. based on data seeded with `testkey_loader.sh`
fn main() {
    env_logger::init();

    // NOTE: certs, packager+main signing: https://gitlab.archlinux.org/archlinux/archlinux-keyring/-/releases/20241015

    // Load OpenPGP certificate directory
    let dir = CertificateDirectoryOpenPGP::new(LoadPaths::System);
    let verifiers = dir.load(
        Os::new("arch".to_string(), None, None, None, None),
        Purpose::new(Role::Packages, Mode::ArtifactVerifier),
        Context::Default,
    );

    eprintln!("loaded {} verifiers:", verifiers.len());

    // Debug print all OpenPGP certificates from `dir`
    verifiers.iter().for_each(|c| eprintln!("{:#?}", c));

    eprintln!("-------------\n");

    // Verify signature for a test-package
    // (from https://ftp.agdsn.de/pub/mirrors/archlinux/core/os/x86_64/)

    // wget https://ftp.agdsn.de/pub/mirrors/archlinux/core/os/x86_64/acl-2.3.2-1-x86_64.pkg.tar.zst
    const PKG: &str = "/tmp/arch/acl-2.3.2-1-x86_64.pkg.tar.zst";

    // wget https://ftp.agdsn.de/pub/mirrors/archlinux/core/os/x86_64/acl-2.3.2-1-x86_64.pkg.tar.zst.sig
    const SIG: &str = "/tmp/arch/acl-2.3.2-1-x86_64.pkg.tar.zst.sig";

    let file = File::open(SIG).expect("file open");
    let mut buf_reader = BufReader::new(file);

    let sigs = rpgpie::signature::load(&mut buf_reader).expect("read signature file");

    for v in verifiers {
        v.verify(Path::new(PKG), &sigs)
    }
}
