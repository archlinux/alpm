//! An example that runs a command using [`RootlesskitBackend`].
//!
//! # Note
//!
//! This example is very simple and mostly useful in the integration tests of this project.
//! However, it also illustrates how to make use of the [`RootlesskitBackend`].

use std::process::ExitCode;

use alpm_rootless::{RootlessBackend, RootlesskitBackend, RootlesskitOptions};
use clap::Parser;
use log::{LevelFilter, debug};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};

/// An error that may occur when executing this example.
#[derive(Debug, thiserror::Error)]
enum Error {
    /// A rootlesskit error.
    #[error(transparent)]
    Rootlesskit(#[from] alpm_rootless::Error),
    /// The call returned with non-zero return code.
    #[error("Non-zero return code:\n{stderr}")]
    NonZero {
        /// The stderr of the failed call.
        stderr: String,
    },
}

/// Command-line interface parsing.
#[derive(Debug, Parser)]
struct Cli {
    #[arg(help = "A command to run with rootlesskit")]
    cmd: Vec<String>,
}

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

/// Runs `cmd` using [`RootlesskitBackend`].
///
/// Prints the stdout of the `cmd` call to stdout.
///
/// # Errors
///
/// Returns an error if
///
/// - [`RootlesskitBackend::run`] fails,
/// - or if the executable called using [`RootlesskitBackend::run`] returns with a non-zero status
///   code.
fn run_cmd(cmd: &[&str]) -> Result<(), Error> {
    let backend = RootlesskitBackend::new(RootlesskitOptions::default());
    let output = backend.run(cmd)?;
    if !output.status.success() {
        return Err(Error::NonZero {
            stderr: format!("{}", String::from_utf8_lossy(&output.stderr)),
        });
    }

    print!("{}", String::from_utf8_lossy(&output.stdout));

    Ok(())
}

/// Runs a command using [`RootlesskitBackend`].
///
/// Accepts the arguments to this example executable as command to run using [`RootlesskitBackend`].
fn main() -> ExitCode {
    init_logger();

    let cli = Cli::parse();
    let cmd: Vec<&str> = cli.cmd.iter().map(|arg| arg.as_str()).collect();

    if let Err(error) = run_cmd(&cmd) {
        eprintln!("{error}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
