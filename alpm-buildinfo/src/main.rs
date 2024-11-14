use std::process::exit;

use alpm_buildinfo::{
    cli::{Cli, Command},
    create_file,
    validate,
    Error,
};
use clap::Parser;

/// Implements helper for exit code handling
trait ExitResult {
    fn handle_exit_code(&self);
}

impl ExitResult for Result<(), Error> {
    /// Handle a [`Result`] by differing exit code and potentially printing to stderr
    ///
    /// If `self` contains `()`, exit with an exit code of 0.
    /// If `self` contains [`Error`], print it to stderr and exit with an exit code of 1.
    fn handle_exit_code(&self) {
        match self {
            Ok(_) => exit(0),
            Err(error) => {
                eprintln!("{}", error);
                exit(1)
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Create { command } => create_file(command).handle_exit_code(),
        Command::Validate { file, schema } => {
            validate(file.as_ref(), schema.as_ref()).handle_exit_code()
        }
    }
}
