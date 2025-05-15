#![cfg(feature = "_containerized-integration-test")]

use std::{
    fs::{File, create_dir_all},
    os::unix::fs::symlink,
};

use log::{debug, warn};
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use testresult::TestResult;
use voa_core::{
    LoadPaths,
    Voa,
    types::{Context, CustomContext, Mode, Os, Purpose, Role, Technology},
};

fn init_logger() -> TestResult {
    if TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .is_err()
    {
        debug!("Not initializing another logger, as one is initialized already.");
    }

    Ok(())
}

/// Objects that can be created during test setup
#[derive(Debug)]
enum Thing {
    Path(&'static str),
    File(&'static str),
    SymLink(&'static str, &'static str),
}

/// Set up a test environment
fn setup(things: &[Thing]) -> std::io::Result<()> {
    for t in things {
        match t {
            Thing::Path(p) => create_dir_all(p)?,
            Thing::File(f) => {
                File::create(f)?;
            }
            Thing::SymLink(from, to) => symlink(from, to)?,
        }
    }

    Ok(())
}

/// List information about verifiers in the system load paths
/// for os=arch, purpose=packages, context=default, technology=openpgp.
#[test]
fn list_verifiers() -> TestResult {
    init_logger()?;

    const SETUP: &[Thing] = &[
        Thing::Path("/usr/local/share/voa/arch/packages/default/openpgp/"),
        Thing::File("/usr/local/share/voa/arch/packages/default/openpgp/foo.pgp"),
        Thing::Path("/etc/voa/arch/packages/default/openpgp/"),
        Thing::SymLink(
            "/usr/local/share/voa/arch/packages/default/openpgp/foo.pgp",
            "/etc/voa/arch/packages/default/openpgp/foo.pgp",
        ),
    ];

    setup(SETUP).expect("test setup");

    let voa = Voa::new(LoadPaths::System);
    let verifiers = voa.load(
        Os::new("arch".to_string(), None, None, None, None),
        Purpose::new(Role::Packages, Mode::ArtifactVerifier),
        Context::Default,
        Technology::OpenPGP,
    );

    warn!("Found verifiers: {:#?}", verifiers);

    Ok(())
}

/// A symlink that points to a verifier outside the load paths
#[test]
fn invalid_verifier_symlink() -> TestResult {
    init_logger()?;

    const SETUP: &[Thing] = &[
        Thing::Path("/usr/local/share/voa/arch/packages/default/openpgp/"),
        Thing::File("/tmp/foo.pgp"),
        Thing::SymLink(
            "/tmp/foo.pgp",
            "/usr/local/share/voa/arch/packages/default/openpgp/foo.pgp",
        ),
    ];

    setup(SETUP).expect("test setup");

    let voa = Voa::new(LoadPaths::System);
    let verifiers = voa.load(
        Os::new("arch".to_string(), None, None, None, None),
        Purpose::new(Role::Packages, Mode::ArtifactVerifier),
        Context::Default,
        Technology::OpenPGP,
    );

    assert!(verifiers.is_empty());

    Ok(())
}

/// VOA setup with a symlink that intermittently escapes outside the load paths
#[test]
fn invalid_dir_symlink() -> TestResult {
    init_logger()?;

    // We start with a verifier in the "default" context in
    // "/etc/voa/arch/packages/default/openpgp/"
    //
    // Then we set up a symlink contraption that tries to use this verifier from
    // "/usr/local/share/voa/arch/packages/custom/"
    //
    // However, the symlink setup intermediately escapes from the load path into "/tmp/custom".
    // So the verifier cannot be loaded via the "custom" context in VOA.
    const SETUP: &[Thing] = &[
        Thing::Path("/etc/voa/arch/packages/default/openpgp/"),
        Thing::File("/etc/voa/arch/packages/default/openpgp/foo.pgp"),
        Thing::Path("/usr/local/share/voa/arch/packages/"),
        Thing::Path("/tmp/custom/"),
        Thing::SymLink("/tmp/custom/", "/usr/local/share/voa/arch/packages/custom"),
        Thing::SymLink(
            "/etc/voa/arch/packages/default/openpgp/",
            "/tmp/custom/openpgp",
        ),
    ];

    setup(SETUP).expect("test setup");

    let voa = Voa::new(LoadPaths::System);

    // Try to load the verifier in the "custom" context, which will fail
    // because the symlink setup is invalid
    let verifiers = voa.load(
        Os::new("arch".to_string(), None, None, None, None),
        Purpose::new(Role::Packages, Mode::ArtifactVerifier),
        Context::Custom(CustomContext::new("custom".into()).expect("custom context")),
        Technology::OpenPGP,
    );

    assert!(verifiers.is_empty());

    Ok(())
}
