use std::process::ExitCode;

use alpm_buildinfo::{
    cli::{Cli, Command},
    create_file,
    format,
    validate,
};
use clap::Parser;

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
