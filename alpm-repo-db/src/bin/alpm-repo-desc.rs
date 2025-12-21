//! The `alpm-repo-desc` CLI tool.

use std::process::ExitCode;

use alpm_repo_db::desc::{
    cli::{Cli, Command},
    commands::{create_file, format, validate},
};
use clap::Parser;

// Initialize i18n support.
fluent_i18n::i18n!("locales");

/// The main entrypoint for the `alpm-repo-desc` executable.
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
