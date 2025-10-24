//! Command line interface for alpm-soname.

use std::process::ExitCode;

use alpm_soname::cli::{Cli, Command};
use clap::Parser;
use log::{debug, error};
use simplelog::{Config, SimpleLogger};

mod commands;

use commands::{get_dependencies, get_provisions, get_raw_dependencies};

fn main() -> ExitCode {
    let cli = Cli::parse();

    if SimpleLogger::init(cli.verbose.log_level_filter(), Config::default()).is_err() {
        debug!("Not initializing another logger, as one is initialized already.");
    }

    let result = match cli.command {
        Command::GetProvisions { args, lookup_dir } => {
            get_provisions(args, lookup_dir, &mut std::io::stdout())
        }
        Command::GetDependencies { args, lookup_dir } => {
            get_dependencies(args, lookup_dir, &mut std::io::stdout())
        }
        Command::GetRawDependencies { args } => get_raw_dependencies(args, &mut std::io::stdout()),
    };

    if let Err(error) = result {
        error!("{error}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
