use std::process::ExitCode;

use alpm_mtree::{
    cli::{Cli, Command},
    validate,
};
use clap::Parser;

/// The entry point for the alpm-mtree binary.
///
/// Parse the cli arguments and call the respective alpm-mtree library functions
fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Validate { file } => validate(file.as_ref()),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
