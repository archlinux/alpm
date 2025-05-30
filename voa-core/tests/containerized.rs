#![cfg(feature = "_containerized-integration-test")]

use std::{
    ffi::OsStr,
    fs::{File, create_dir_all},
    io::{Read, Write},
    os::unix::fs::symlink,
};

use log::{debug, warn};
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use testresult::TestResult;
use voa_core::{
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

/// Objects to create during a VOA test setup:
/// Paths, (empty) files and symlinks.
#[derive(Debug)]
enum TestObject {
    Path(&'static str),
    File(&'static str),
    SymLink(&'static str, &'static str),
}

/// Set up a test environment
fn setup(objs: &[TestObject]) -> std::io::Result<()> {
    for obj in objs {
        match obj {
            TestObject::Path(p) => create_dir_all(p)?,
            TestObject::File(f) => {
                File::create(f)?;
            }
            TestObject::SymLink(orig, link) => symlink(orig, link)?,
        }
    }

    Ok(())
}

/// List information about verifiers in the system load paths
/// for os=arch, purpose=packages, context=default, technology=openpgp.
///
/// This test does nothing fancy, except follow a symlink. It mostly tests the happy path.
#[test]
fn list_verifiers() -> TestResult {
    init_logger()?;

    const SETUP: &[TestObject] = &[
        TestObject::Path("/usr/local/share/voa/arch/packages/default/openpgp/"),
        TestObject::File("/usr/local/share/voa/arch/packages/default/openpgp/foo.pgp"),
        TestObject::Path("/etc/voa/arch/packages/default/openpgp/"),
        TestObject::SymLink(
            "/usr/local/share/voa/arch/packages/default/openpgp/foo.pgp",
            "/etc/voa/arch/packages/default/openpgp/foo.pgp",
        ),
    ];

    setup(SETUP).expect("test setup");

    let voa = Voa::init();
    let verifiers = voa.lookup(
        Os::new("arch".to_string(), None, None, None, None).unwrap(),
        Purpose::new(Role::Packages, Mode::ArtifactVerifier),
        Context::Default,
        Technology::OpenPGP,
    );

    assert_eq!(verifiers.len(), 2);

    warn!("Found verifiers: {:#?}", verifiers);

    Ok(())
}

/// A symlink that points to a verifier outside the load paths
#[test]
fn invalid_verifier_symlink() -> TestResult {
    init_logger()?;

    const SETUP: &[TestObject] = &[
        TestObject::Path("/usr/local/share/voa/arch/packages/default/openpgp/"),
        TestObject::File("/tmp/foo.pgp"),
        TestObject::SymLink(
            "/tmp/foo.pgp",
            "/usr/local/share/voa/arch/packages/default/openpgp/foo.pgp",
        ),
    ];

    setup(SETUP).expect("test setup");

    let voa = Voa::init();
    let verifiers = voa.lookup(
        Os::new("arch".to_string(), None, None, None, None).unwrap(),
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
    const SETUP: &[TestObject] = &[
        TestObject::Path("/etc/voa/arch/packages/default/openpgp/"),
        TestObject::File("/etc/voa/arch/packages/default/openpgp/foo.pgp"),
        TestObject::Path("/usr/local/share/voa/arch/packages/"),
        TestObject::Path("/tmp/custom/"),
        TestObject::SymLink("/tmp/custom/", "/usr/local/share/voa/arch/packages/custom"),
        TestObject::SymLink(
            "/etc/voa/arch/packages/default/openpgp/",
            "/tmp/custom/openpgp",
        ),
    ];

    setup(SETUP).expect("test setup");

    let voa = Voa::init();

    // Try to load the verifier in the "custom" context, which will fail
    // because the symlink setup is invalid
    let verifiers = voa.lookup(
        Os::new("arch".to_string(), None, None, None, None).unwrap(),
        Purpose::new(Role::Packages, Mode::ArtifactVerifier),
        Context::Custom(CustomContext::new("custom".into()).expect("custom context")),
        Technology::OpenPGP,
    );

    assert!(verifiers.is_empty());

    Ok(())
}

/// VOA setup with a symlink that masks a verifier.
/// All copies of verifiers with that name should not be returned as a result.
#[test]
fn masking() -> TestResult {
    init_logger()?;

    const SETUP: &[TestObject] = &[
        TestObject::Path("/etc/voa/arch/packages/default/openpgp/"),
        TestObject::File("/etc/voa/arch/packages/default/openpgp/foo.pgp"),
        TestObject::File("/etc/voa/arch/packages/default/openpgp/bar.pgp"),
        TestObject::Path("/usr/local/share/voa/arch/packages/default/openpgp/"),
        TestObject::File("/usr/local/share/voa/arch/packages/default/openpgp/foo.pgp"),
        TestObject::Path("/run/voa/arch/packages/default/openpgp/"),
        TestObject::SymLink(
            "/dev/null",
            "/run/voa/arch/packages/default/openpgp/foo.pgp",
        ),
    ];

    setup(SETUP).expect("test setup");

    let voa = Voa::init();

    // Try to load verifiers, which should return no results because there is a masking
    let verifiers = voa.lookup(
        Os::new("arch".to_string(), None, None, None, None).unwrap(),
        Purpose::new(Role::Packages, Mode::ArtifactVerifier),
        Context::Default,
        Technology::OpenPGP,
    );

    assert_eq!(verifiers.len(), 1);
    assert_eq!(verifiers[0].path().file_name(), Some(OsStr::new("bar.pgp")));

    Ok(())
}

/// VOA setup with a file that is linked via a chain of two symlinks.
#[test]
fn symlink_multihop_file() -> TestResult {
    init_logger()?;

    const SETUP: &[TestObject] = &[
        TestObject::Path("/etc/voa/arch/packages/default/openpgp/"),
        TestObject::Path("/etc/voa/arch/packages/custom1/openpgp/"),
        TestObject::Path("/etc/voa/arch/packages/custom2/openpgp/"),
        TestObject::File("/etc/voa/arch/packages/default/openpgp/foo.pgp"),
        TestObject::SymLink(
            "/etc/voa/arch/packages/default/openpgp/foo.pgp",
            "/etc/voa/arch/packages/custom1/openpgp/foo.pgp",
        ),
        TestObject::SymLink(
            "/etc/voa/arch/packages/custom1/openpgp/foo.pgp",
            "/etc/voa/arch/packages/custom2/openpgp/foo.pgp",
        ),
    ];

    setup(SETUP).expect("test setup");

    let voa = Voa::init();

    // Try to load verifiers via "custom2", which should return foo.pgp,
    // which points to the default context via an intermediate hop in the "custom1" context.
    let verifiers = voa.lookup(
        Os::new("arch".to_string(), None, None, None, None).unwrap(),
        Purpose::new(Role::Packages, Mode::ArtifactVerifier),
        Context::Custom(CustomContext::new("custom2".into()).expect("custom context")),
        Technology::OpenPGP,
    );

    assert_eq!(verifiers.len(), 1);
    assert_eq!(verifiers[0].path().file_name(), Some(OsStr::new("foo.pgp")));

    // TODO: assert that we found the full canonicalized path

    Ok(())
}

/// VOA setup with a cycle of two file-symlinks.
/// Will not terminate without cycle detection!
#[test]
fn symlink_cycle_file() -> TestResult {
    init_logger()?;

    const SETUP: &[TestObject] = &[
        TestObject::Path("/etc/voa/arch/packages/custom1/openpgp/"),
        TestObject::Path("/etc/voa/arch/packages/custom2/openpgp/"),
        TestObject::SymLink(
            "/etc/voa/arch/packages/custom2/openpgp/foo.pgp",
            "/etc/voa/arch/packages/custom1/openpgp/foo.pgp",
        ),
        TestObject::SymLink(
            "/etc/voa/arch/packages/custom1/openpgp/foo.pgp",
            "/etc/voa/arch/packages/custom2/openpgp/foo.pgp",
        ),
    ];

    setup(SETUP).expect("test setup");

    let voa = Voa::init();

    // Try to load verifiers via "custom2", which should only find a symlink loop and return empty.
    let verifiers = voa.lookup(
        Os::new("arch".to_string(), None, None, None, None).unwrap(),
        Purpose::new(Role::Packages, Mode::ArtifactVerifier),
        Context::Custom(CustomContext::new("custom2".into()).expect("custom context")),
        Technology::OpenPGP,
    );

    assert!(verifiers.is_empty());

    Ok(())
}

/// VOA setup with a directory that is linked via a chain of two symlinks.
#[test]
fn symlink_multihop_dir() -> TestResult {
    init_logger()?;

    const SETUP: &[TestObject] = &[
        TestObject::Path("/etc/voa/arch/packages/default/openpgp/"),
        TestObject::Path("/etc/voa/arch/packages/custom1/"),
        TestObject::Path("/etc/voa/arch/packages/custom2/"),
        TestObject::File("/etc/voa/arch/packages/default/openpgp/foo.pgp"),
        TestObject::SymLink(
            "/etc/voa/arch/packages/default/openpgp",
            "/etc/voa/arch/packages/custom1/openpgp",
        ),
        TestObject::SymLink(
            "/etc/voa/arch/packages/custom1/openpgp",
            "/etc/voa/arch/packages/custom2/openpgp",
        ),
    ];

    setup(SETUP).expect("test setup");

    let voa = Voa::init();

    // Try to load verifiers via "custom2", which should return foo.pgp,
    // which points to the default context via an intermediate hop in the "custom1" context.
    let verifiers = voa.lookup(
        Os::new("arch".to_string(), None, None, None, None).unwrap(),
        Purpose::new(Role::Packages, Mode::ArtifactVerifier),
        Context::Custom(CustomContext::new("custom2".into()).expect("custom context")),
        Technology::OpenPGP,
    );

    assert_eq!(verifiers.len(), 1);
    assert_eq!(verifiers[0].path().file_name(), Some(OsStr::new("foo.pgp")));

    // TODO: assert that we found the full canonicalized path

    Ok(())
}

/// VOA setup with a cycle of two dir-symlinks.
/// Will not terminate without cycle detection!
#[test]
fn symlink_cycle_dir() -> TestResult {
    init_logger()?;

    const SETUP: &[TestObject] = &[
        TestObject::Path("/etc/voa/arch/packages/custom1/"),
        TestObject::Path("/etc/voa/arch/packages/custom2/"),
        TestObject::SymLink(
            "/etc/voa/arch/packages/custom2/openpgp",
            "/etc/voa/arch/packages/custom1/openpgp",
        ),
        TestObject::SymLink(
            "/etc/voa/arch/packages/custom1/openpgp",
            "/etc/voa/arch/packages/custom2/openpgp",
        ),
    ];

    setup(SETUP).expect("test setup");

    let voa = Voa::init();

    // Try to load verifiers via "custom2", which should only find a symlink loop and return empty.
    let verifiers = voa.lookup(
        Os::new("arch".to_string(), None, None, None, None).unwrap(),
        Purpose::new(Role::Packages, Mode::ArtifactVerifier),
        Context::Custom(CustomContext::new("custom2".into()).expect("custom context")),
        Technology::OpenPGP,
    );

    assert!(verifiers.is_empty());

    Ok(())
}

/// VOA setup with a verifier that contains data.
///
/// We find the verifier, read it, and check that it contains the expected data.
#[test]
fn read_verifier() -> TestResult {
    init_logger()?;

    // Set up the verifier and write data into it
    const SETUP: &[TestObject] = &[TestObject::Path("/etc/voa/arch/packages/default/openpgp/")];

    setup(SETUP).expect("test setup");

    let mut file = File::create("/etc/voa/arch/packages/default/openpgp/foo.pgp").unwrap();
    file.write_all(b"hello world").unwrap();

    // Find the verifier and read its data
    let voa = Voa::init();

    let verifiers = voa.lookup(
        Os::new("arch".to_string(), None, None, None, None).unwrap(),
        Purpose::new(Role::Packages, Mode::ArtifactVerifier),
        Context::Default,
        Technology::OpenPGP,
    );

    assert_eq!(verifiers.len(), 1);

    let mut verifier = &verifiers[0].open().expect("open file");

    let mut s = String::new();
    let _ = verifier.read_to_string(&mut s).expect("read file");

    assert_eq!(s, "hello world");

    Ok(())
}
