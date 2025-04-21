use std::process::ExitCode;

use alpm_soname::{
    cli::{Cli, Command},
    commands::{get_dependencies, get_provisions},
};
use clap::Parser;
use log::{LevelFilter, debug, error};
use simplelog::{Config, SimpleLogger};

fn main() -> ExitCode {
    let cli = Cli::parse();

    let log_level = match cli.verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    if SimpleLogger::init(log_level, Config::default()).is_err() {
        debug!("Not initializing another logger, as one is initialized already.");
    }

    let result = match cli.command {
        Command::GetProvisions { args } => get_provisions(args, &mut std::io::stdout()),
        Command::GetDependencies { args } => get_dependencies(args, &mut std::io::stdout()),
    };

    if let Err(error) = result {
        error!("{error}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
