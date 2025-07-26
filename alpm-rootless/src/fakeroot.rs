//! A rootless backend that uses [fakeroot].
//!
//! [fakeroot]: https://man.archlinux.org/man/fakeroot.1

use std::{
    fmt::Display,
    process::{Command, Output},
};

use log::debug;

use crate::{Error, RootlessBackend, RootlessOptions, utils::get_command};

/// Options for [fakeroot].
///
/// [fakeroot]: https://man.archlinux.org/man/fakeroot.1
#[derive(Clone, Debug)]
pub struct FakerootOptions {
    /// An alternative wrapper library.
    ///
    /// Corresponds to `fakeroot`'s `-l`/`--lib` option.
    pub library: Option<String>,

    /// An alternative binary to use as `faked`.
    ///
    /// Corresponds to `fakeroot`'s `--faked` option.
    pub faked: Option<String>,

    /// A file to save the environment to.
    ///
    /// Corresponds to `fakeroot`'s `-s` option.
    pub save_file: Option<String>,

    /// A file to load a previous environment from.
    ///
    /// Corresponds to `fakeroot`'s `-i` option.
    pub load_file: Option<String>,

    /// Whether to use the real ownership of files.
    ///
    /// Corresponds to `fakeroot`'s `-u`/`--unknown-is-real` option.
    pub unknown_is_real: bool,

    /// The minimum file descriptor number for TCP connections.
    ///
    /// Corresponds to `fakeroot`'s `-b` option.
    pub fd: Option<usize>,
}

impl AsRef<FakerootOptions> for FakerootOptions {
    fn as_ref(&self) -> &FakerootOptions {
        self
    }
}

impl Display for FakerootOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_vec().join(" "))
    }
}

impl RootlessOptions for FakerootOptions {
    /// Returns the options as a [`String`] [`Vec`].
    ///
    /// This always appends a trailing `--` to the list of options.
    fn to_vec(&self) -> Vec<String> {
        let mut options = Vec::new();
        if let Some(option) = self.library.as_ref() {
            options.push("--lib".to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.faked.as_ref() {
            options.push("--faked".to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.save_file.as_ref() {
            options.push("-s".to_string());
            options.push(option.to_string());
        }
        if let Some(option) = self.load_file.as_ref() {
            options.push("-i".to_string());
            options.push(option.to_string());
        }
        if self.unknown_is_real {
            options.push("--unknown-is-real".to_string());
        }
        if let Some(option) = self.fd {
            options.push("-b".to_string());
            options.push(option.to_string());
        }
        options.push("--".to_string());

        options
    }
}

/// A rootless backend for running commands using [fakeroot].
///
/// [fakeroot]: https://man.archlinux.org/man/fakeroot.1
#[derive(Debug)]
pub struct FakerootBackend {
    options: FakerootOptions,
}

impl RootlessBackend<FakerootOptions> for FakerootBackend {
    type Err = Error;

    /// Creates a new [`FakerootBackend`].
    fn new(options: FakerootOptions) -> Self {
        debug!("Create a new fakeroot backend with options \"{options}\"");
        Self { options }
    }

    /// Returns the [`FakerootOptions`] used by the [`FakerootBackend`].
    fn options(&self) -> &FakerootOptions {
        &self.options
    }

    /// Runs a command using [fakeroot] and returns its [`Output`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - the [fakeroot] command cannot be found,
    /// - or the provided `command` cannot be run using [fakeroot].
    ///
    /// [fakeroot]: https://man.archlinux.org/man/fakeroot.1
    fn run(&self, command: &[&str]) -> Result<Output, Error> {
        let mut args = self.options.to_vec();
        for command_component in command.iter() {
            args.push(command_component.to_string());
        }

        let command_name = get_command("fakeroot")?;
        let mut command = Command::new(command_name);
        let command = command.args(args);
        debug!("Run rootless command: {command:?}");

        command
            .output()
            .map_err(|source| crate::Error::CommandExec {
                command: format!("{command:?}"),
                source,
            })
    }
}

#[cfg(test)]
mod tests {
    use log::{LevelFilter, debug};
    use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
    use testresult::TestResult;

    use super::*;

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

    /// Ensures that on a Linux-based system, the [`FakerootBackend`] can be used to run a command
    /// (`whoami`) as root.
    #[test]
    #[cfg(target_os = "linux")]
    fn run_example() -> TestResult {
        init_logger();

        let backend = FakerootBackend::new(FakerootOptions {
            library: None,
            faked: None,
            save_file: None,
            load_file: None,
            unknown_is_real: false,
            fd: None,
        });

        let output = backend.run(&["whoami"])?;

        assert_eq!("root\n", String::from_utf8_lossy(&output.stdout));
        Ok(())
    }
}
