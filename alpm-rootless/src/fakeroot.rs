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
#[derive(Clone, Debug, Default, Eq, PartialEq)]
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
    fn run(&self, command: &[&str]) -> Result<Output, Error> {
        let mut args = self.0.to_vec();
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
    use rstest::rstest;

    use super::*;

    /// Ensures that [`FakerootOptions`] are constructed as [`String`] [`Vec`] properly.
    #[rstest]
    #[case::all_options(
        FakerootOptions{
            library: Some("custom-lib".to_string()),
            faked: Some("custom-faked".to_string()),
            save_file: Some("/custom/save/file".to_string()),
            load_file: Some("/custom/load/file".to_string()),
            unknown_is_real: true,
            fd: Some(1024),
        },
        vec![
            "--lib".to_string(),
            "custom-lib".to_string(),
            "--faked".to_string(),
            "custom-faked".to_string(),
            "-s".to_string(),
            "/custom/save/file".to_string(),
            "-i".to_string(),
            "/custom/load/file".to_string(),
            "--unknown-is-real".to_string(),
            "-b".to_string(),
            "1024".to_string(),
            "--".to_string(),
        ]
    )]
    #[case::default_options(FakerootOptions::default(), vec!["--".to_string()])]
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
