//! The `alpm-srcinfo` CLI tool.

use std::process::ExitCode;

use alpm_srcinfo::cli::{Cli, Command};
use clap::Parser;

mod commands;

fluent_i18n::i18n!("locales");

use crate::commands::{create, format_packages, format_source_info, validate};

/// The entry point for the `alpm-srcinfo` binary.
///
/// Parses the CLI arguments and calls the respective [`alpm_srcinfo`] library functions.
fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Create {
            file,
            output_format,
            pretty,
        } => create(&file, output_format, pretty),
        Command::Validate { file, schema } => validate(file.as_ref(), schema),
        Command::Format {
            file,
            schema,
            output_format,
            pretty,
        } => format_source_info(file.as_ref(), schema, output_format, pretty),
        Command::FormatPackages {
            file,
            schema,
            architecture,
            output_format,
            pretty,
        } => format_packages(file.as_ref(), schema, output_format, architecture, pretty),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
