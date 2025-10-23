//! The `alpm-buildinfo` CLI tool.

use std::process::ExitCode;

use alpm_buildinfo::cli::{Cli, Command};
use clap::Parser;

mod commands;

use commands::{create_file, format, validate};

fluent_i18n::i18n!("locales");

/// The main entrypoint for the `alpm-buildinfo` executable.
///
/// Returns an [`ExitCode::SUCCESS`] if the chosen command succeeded.
/// Returns an [`ExitCode::FAILURE`] and prints an error on stderr if the chosen command failed.
fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Create { command } => create_file(command),
        Command::Validate { args } => validate(args),
        Command::Format {
            args,
            output_format,
            pretty,
        } => format(args, output_format, pretty),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
