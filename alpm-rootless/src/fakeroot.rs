//! A rootless backend that uses [fakeroot].
//!
//! [fakeroot]: https://man.archlinux.org/man/fakeroot.1

use std::{
    fmt::Display,
    path::PathBuf,
    process::{Command, Output},
};

use log::debug;

use crate::{Error, RootlessBackend, RootlessOptions, utils::get_command};

/// The `fakeroot` `-l`/`--lib` option.
const ARG_LIBRARY: &str = "--lib";
/// The `fakeroot` `--faked` option.
const ARG_FAKED: &str = "--faked";
/// The `fakeroot` `-s` option.
const ARG_SAVE_FILE: &str = "-s";
/// The `fakeroot` `-i` option.
const ARG_LOAD_FILE: &str = "-i";
/// The `fakeroot` `--unknown-is-real` option.
const ARG_UNKNOWN_IS_REAL: &str = "--unknown-is-real";
/// The `fakeroot` `-b` option.
const ARG_FD: &str = "-b";
/// The separator between `fakeroot` options and the command run with `fakeroot` (`--`).
const ARG_SEPARATOR: &str = "--";

/// Options for [fakeroot].
///
/// [fakeroot]: https://man.archlinux.org/man/fakeroot.1
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FakerootOptions {
    /// An alternative wrapper library.
    ///
    /// Corresponds to `fakeroot`'s `-l`/`--lib` option.
    pub library: Option<PathBuf>,

    /// An alternative binary to use as `faked`.
    ///
    /// Corresponds to `fakeroot`'s `--faked` option.
    pub faked: Option<PathBuf>,

    /// A file to save the environment to.
    ///
    /// Corresponds to `fakeroot`'s `-s` option.
    pub save_file: Option<PathBuf>,

    /// A file to load a previous environment from.
    ///
    /// Corresponds to `fakeroot`'s `-i` option.
    pub load_file: Option<PathBuf>,

    /// Whether to use the real ownership of files.
    ///
    /// Corresponds to `fakeroot`'s `-u`/`--unknown-is-real` option.
    pub unknown_is_real: bool,

    /// The minimum file descriptor number for TCP connections.
    ///
    /// Corresponds to `fakeroot`'s `-b` option.
    pub fd: Option<usize>,
}

impl Display for FakerootOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_vec().join(" "))
    }
}

impl RootlessOptions for FakerootOptions {
    /// Returns the options as a [`String`] [`Vec`].
    ///
    /// # Notes
    ///
    /// All [`PathBuf`] options are represented using [`PathBuf::to_string_lossy`].
    ///
    /// The last entry is always the string `"--"`.
    fn to_vec(&self) -> Vec<String> {
        let mut options = Vec::new();
        if let Some(option) = self.library.as_ref() {
            options.push(ARG_LIBRARY.to_string());
            options.push(option.to_string_lossy().to_string());
        }
        if let Some(option) = self.faked.as_ref() {
            options.push(ARG_FAKED.to_string());
            options.push(option.to_string_lossy().to_string());
        }
        if let Some(option) = self.save_file.as_ref() {
            options.push(ARG_SAVE_FILE.to_string());
            options.push(option.to_string_lossy().to_string());
        }
        if let Some(option) = self.load_file.as_ref() {
            options.push(ARG_LOAD_FILE.to_string());
            options.push(option.to_string_lossy().to_string());
        }
        if self.unknown_is_real {
            options.push(ARG_UNKNOWN_IS_REAL.to_string());
        }
        if let Some(option) = self.fd {
            options.push(ARG_FD.to_string());
            options.push(option.to_string());
        }
        options.push(ARG_SEPARATOR.to_string());

        options
    }
}

/// A rootless backend for running commands using [fakeroot].
///
/// [fakeroot]: https://man.archlinux.org/man/fakeroot.1
#[derive(Debug)]
pub struct FakerootBackend(FakerootOptions);

impl RootlessBackend<FakerootOptions> for FakerootBackend {
    type Err = Error;

    /// Creates a new [`FakerootBackend`].
    fn new(options: FakerootOptions) -> Self {
        debug!("Creating a new fakeroot backend with options: \"{options}\"");
        Self(options)
    }

    /// Returns the [`FakerootOptions`] used by the [`FakerootBackend`].
    fn options(&self) -> &FakerootOptions {
        &self.0
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
    fn run(&self, cmd: &[&str]) -> Result<Output, Error> {
        let command_name = get_command("fakeroot")?;
        let mut command = Command::new(command_name);

        // Add all options to fakeroot as arguments.
        if let Some(option) = self.0.library.as_ref() {
            command.arg(ARG_LIBRARY);
            command.arg(option);
        }
        if let Some(option) = self.0.faked.as_ref() {
            command.arg(ARG_FAKED);
            command.arg(option);
        }
        if let Some(option) = self.0.save_file.as_ref() {
            command.arg(ARG_SAVE_FILE);
            command.arg(option);
        }
        if let Some(option) = self.0.load_file.as_ref() {
            command.arg(ARG_LOAD_FILE);
            command.arg(option);
        }
        if self.0.unknown_is_real {
            command.arg(ARG_UNKNOWN_IS_REAL);
        }
        if let Some(option) = self.0.fd {
            command.arg(ARG_FD);
            command.arg(option.to_string());
        }
        command.arg(ARG_SEPARATOR);

        // Add input cmd as arguments to fakeroot.
        for command_component in cmd.iter() {
            command.arg(command_component);
        }

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
    use rstest::rstest;

    use super::*;

    /// Ensures that [`FakerootOptions`] are constructed as [`String`] [`Vec`] properly.
    #[rstest]
    #[case::all_options(
        FakerootOptions{
            library: Some(PathBuf::from("custom-lib")),
            faked: Some(PathBuf::from("custom-faked")),
            save_file: Some(PathBuf::from("/custom/save/file")),
            load_file: Some(PathBuf::from("/custom/load/file")),
            unknown_is_real: true,
            fd: Some(1024),
        },
        vec![
            ARG_LIBRARY.to_string(),
            "custom-lib".to_string(),
            ARG_FAKED.to_string(),
            "custom-faked".to_string(),
            ARG_SAVE_FILE.to_string(),
            "/custom/save/file".to_string(),
            ARG_LOAD_FILE.to_string(),
            "/custom/load/file".to_string(),
            ARG_UNKNOWN_IS_REAL.to_string(),
            ARG_FD.to_string(),
            "1024".to_string(),
            ARG_SEPARATOR.to_string(),
        ]
    )]
    #[case::default_options(FakerootOptions::default(), vec![ARG_SEPARATOR.to_string()])]
    fn fakeroot_options_to_vec(#[case] options: FakerootOptions, #[case] to_vec: Vec<String>) {
        assert_eq!(options.to_vec(), to_vec);
    }

    /// Ensures that [`FakerootOptions`] is returned from [`FakerootBackend::options`].
    #[test]
    fn fakeroot_backend_options() {
        let backend = FakerootBackend::new(FakerootOptions::default());
        assert_eq!(backend.options(), &FakerootOptions::default());
    }
}
