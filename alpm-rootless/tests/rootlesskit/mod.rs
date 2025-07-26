//! Integration tests for [rootlesskit].
//!
//! [rootlesskit]: https://github.com/rootless-containers/rootlesskit

use alpm_rootless::{RootlessBackend, RootlesskitBackend, RootlesskitOptions, detect_virt};
use log::{LevelFilter, debug};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use testresult::TestResult;

/// Initializes a global [`TermLogger`].
fn init_logger() {
    if TermLogger::init(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
    .is_err()
    {
        debug!("Not initializing another logger, as one is initialized already.");
    }
}

/// Ensures that on a Linux-based system, the [`FakerootBackend`] can be used to run a
/// command ([whoami]) as root.
///
/// [whoami]: https://man.archlinux.org/man/whoami.1
#[test]
#[cfg(target_os = "linux")]
fn rootlesskitbackend_run() -> TestResult {
    init_logger();

    let virt = detect_virt()?;
    if !virt.uses_namespaces() {
        let backend = RootlesskitBackend::new(RootlesskitOptions::default());

        let output = backend.run(&["whoami"])?;

        if !output.status.success() {
            return Err(format!(
                "The \"rootlesskit\" call exited with a non-zero return code:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        assert_eq!("root\n", String::from_utf8_lossy(&output.stdout));
    }
    Ok(())
}
