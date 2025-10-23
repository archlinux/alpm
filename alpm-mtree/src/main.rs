//! The `alpm-mtree` CLI tool.

use std::process::ExitCode;

use alpm_mtree::cli::{Cli, Command};
use clap::Parser;

mod commands;

use commands::{format, validate};

fluent_i18n::i18n!("locales");

/// The entry point for the `alpm-mtree` binary.
///
/// Parse the cli arguments and call the respective alpm-mtree library functions
fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Validate { file, schema } => validate(file.as_ref(), schema),
        Command::Format {
            file,
            schema,
            output_format,
            pretty,
        } => format(file.as_ref(), schema, output_format, pretty),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
