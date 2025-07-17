//! The `alpm-pkginfo` CLI tool.

use std::process::ExitCode;

mod commands;

use alpm_lint::cli::{Cli, Command};
use clap::Parser;
use simplelog::{Config, LevelFilter, SimpleLogger};

use crate::commands::{check, meta, options, rules};

fn main() -> ExitCode {
    let cli = Cli::parse();
    // Init and set the verbosity level of the logger.
    let level = match cli.verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    if let Err(error) = SimpleLogger::init(level, Config::default()) {
        eprintln!("Failed to initialize logger:\n{error}");
        return ExitCode::FAILURE;
    };

    let result = match cli.command {
        Command::Check {
            config,
            path,
            scope,
            level,
            format,
            output,
            pretty,
        } => check(config, path, scope, level, format, output, pretty),
        Command::Rules {
            format: output_format,
            pretty,
            output,
        } => rules(output_format, pretty, output),
        Command::Options {
            format: output_format,
            pretty,
            output,
        } => options(output_format, pretty, output),
        Command::Meta {
            format: output_format,
            pretty,
            output,
        } => meta(output_format, pretty, output),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
