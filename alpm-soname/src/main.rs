use std::process::ExitCode;

use alpm_soname::{
    cli::{Cli, Command},
    commands::{get_dependencies, get_provisions},
};
use clap::Parser;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::GetProvisions { args } => get_provisions(args, &mut std::io::stdout()),
        Command::GetDependencies { args } => get_dependencies(args, &mut std::io::stdout()),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
