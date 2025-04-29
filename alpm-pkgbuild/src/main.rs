#![doc = include_str!("../README.md")]

use std::process::ExitCode;

use alpm_pkgbuild::cli::{Cli, Command, SourceInfoCommand};
use clap::Parser;
use commands::{print_source_info, run_bridge};
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};

mod commands;

/// The entry point for the alpm-pkgbuild binary.
///
/// Parse the cli arguments and call the respective alpm-pkgbuild command functions
fn main() -> ExitCode {
    let cli = Cli::parse();
    init_logger(cli.verbosity);
    let result = match cli.command {
        Command::Srcinfo { subcommand } => match subcommand {
            SourceInfoCommand::Format { pkgbuild_path } => print_source_info(pkgbuild_path),
            SourceInfoCommand::RunBridge { pkgbuild_path } => run_bridge(pkgbuild_path),
        },
    };

    if let Err(error) = result {
        eprintln!("{error}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Initializes a global logger once.
fn init_logger(verbosity: u8) {
    let level = match verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    if let Err(err) = TermLogger::init(
        level,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    ) {
        eprintln!("Failed to initialize logger:\n{err}")
    }
}
