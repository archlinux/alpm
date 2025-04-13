use std::process::ExitCode;

use alpm_srcinfo::{
    cli::{Cli, Command},
    commands::{check, create, format_packages, validate},
};
use clap::Parser;

/// The entry point for the `alpm-srcinfo` binary.
///
/// Parses the CLI arguments and calls the respective [`alpm_srcinfo`] library functions.
fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Check { file, schema } => check(file.as_ref(), schema),
        Command::Validate { file, schema } => validate(file.as_ref(), schema),
        Command::FormatPackages {
            file,
            schema,
            architecture,
            output_format,
            pretty,
        } => format_packages(file.as_ref(), schema, output_format, architecture, pretty),
        Command::Create {
            pkgbuild_path,
            output,
        } => create(&pkgbuild_path, &output),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
