//! Integration tests for [`alpm_rootless`].

use log::{LevelFilter, debug};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};

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

mod fakeroot {
    use alpm_rootless::{FakerootBackend, FakerootOptions, RootlessBackend};
    use testresult::TestResult;

    use super::*;

    /// Ensures that on a Linux-based system, the [`FakerootBackend`] can be used to run a
    /// command ([whoami]) as root.
    ///
    /// [whoami]: https://man.archlinux.org/man/whoami.1
    #[test]
    #[cfg(target_os = "linux")]
    fn fakerootbackend_run() -> TestResult {
        init_logger();

        let backend = FakerootBackend::new(FakerootOptions::default());

        let output = backend.run(&["whoami"])?;
        if !output.status.success() {
            return Err(format!(
                "The \"fakeroot\" call exited with a non-zero return code:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        assert_eq!("root\n", String::from_utf8_lossy(&output.stdout));
        Ok(())
    }
}

mod utils {
    use alpm_rootless::{detect_virt, get_command};
    use testresult::TestResult;

    /// Ensures that the "whoami" command can be found on a Linux system.
    #[test]
    #[cfg(target_os = "linux")]
    fn get_command_succeeds() -> TestResult {
        let command = "whoami";
        if let Err(error) = get_command(command) {
            panic!("Should have found command \"{command}\", but got error instead:\n{error}")
        };

        Ok(())
    }

    /// Ensures that a command unlikely to ever exist cannot be found on a Linux system.
    #[test]
    #[cfg(target_os = "linux")]
    fn get_command_fails() -> TestResult {
        let command = "d202d7951df2c4b711ca44b4bcc9d7b363fa4252127e058c1a910ec05b6cd038d71cc21221c031c0359f993e746b07f5965cf8c5c3746a58337ad9ab65278e77";

        if let Ok(path) = get_command(command) {
            panic!("Should not have found command {path:?}, but succeeded");
        };

        Ok(())
    }

    /// Ensures that the current environment is successfully detected using [systemd-detect-virt].
    ///
    /// [systemd-detect-virt]: https://man.archlinux.org/man/systemd-detect-virt.1
    #[test]
    #[cfg(target_os = "linux")]
    fn detect_virt_succeeds() -> TestResult {
        detect_virt()?;

        Ok(())
    }
}
