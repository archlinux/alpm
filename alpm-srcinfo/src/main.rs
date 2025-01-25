use std::process::ExitCode;

use alpm_srcinfo::{
    cli::{Cli, Command},
    commands::{check, format, validate},
};
use clap::Parser;

/// The entry point for the alpm-srcinfo binary.
///
/// Parse the cli arguments and call the respective alpm-srcinfo library functions
fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Check { file } => check(file.as_ref()),
        Command::Validate { file } => validate(file.as_ref()),
        Command::Format {
            file,
            architecture,
            output_format,
            pretty,
        } => format(file.as_ref(), output_format, architecture, pretty),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
