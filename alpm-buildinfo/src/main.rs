use std::process::ExitCode;

use alpm_buildinfo::{
    cli::{Cli, Command},
    create_file,
    validate,
};
use clap::Parser;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Create { command } => create_file(command),
        Command::Validate { file, schema } => validate(file.as_ref(), schema.as_ref()),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
