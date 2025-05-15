#![cfg(feature = "_containerized-integration-test")]

use std::fs;

use log::{debug, warn};
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use testresult::TestResult;
use voa_core::{Context, LoadPaths, Mode, Os, Purpose, Role, Technology, Voa};

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

/// List information about opaque verifiers in the system load paths
/// for os=arch, purpose=packages, context=default, technology=openpgp.
#[test]
fn list_verifiers() -> TestResult {
    init_logger()?;

    fs::create_dir_all("/usr/local/share/voa/arch/packages/default/openpgp/")?;
    fs::File::create("/usr/local/share/voa/arch/packages/default/openpgp/foo.pgp")?;

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
