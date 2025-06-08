//! The `alpm-pkginfo` CLI tool.

use std::process::ExitCode;

use alpm_pkginfo::{
    cli::{Cli, Command},
    commands::{create_file, format, validate},
};
use clap::Parser;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Create { command } => create_file(command),
        Command::Validate { file, schema } => validate(file, schema),
        Command::Format {
            file,
            schema,
            output_format,
            pretty,
        } => format(file, schema, output_format, pretty),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
