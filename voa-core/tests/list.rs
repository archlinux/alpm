use voa_core::{Context, LoadPaths, Mode, Os, Purpose, Role, Technology, Voa};

/// List information about opaque verifiers in the system load paths
/// for os=arch, purpose=packages, context=default, technology=openpgp.
#[test]
fn list_verifiers() {
    let voa = Voa::new(LoadPaths::System);
    let verifiers = voa.load(
        Os::new("arch".to_string(), None, None, None, None),
        Purpose::new(Role::Packages, Mode::ArtifactVerifier),
        Context::Default,
        Technology::OpenPGP,
    );

    eprintln!("{:#?}", verifiers);
}
