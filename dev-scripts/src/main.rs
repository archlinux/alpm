//! The `dev-scripts` CLI tool.

use std::process::ExitCode;

use clap::Parser;
use cli::Cli;
use simplelog::{Config, SimpleLogger};

use crate::{
    cache::CacheDir,
    cli::Command,
    commands::{clean_files, compare_source_info, download_files, test_files},
    error::Error,
};

mod cache;
mod cli;
mod cmd;
mod commands;
mod consts;
mod error;
pub mod sync;
pub mod testing;
mod ui;

/// Runs a command of the `dev-scripts` executable.
fn run_command() -> Result<(), Error> {
    let cli = Cli::parse();
    SimpleLogger::init(cli.verbose.log_level_filter(), Config::default())?;

    let cache_dir = if let Some(path) = cli.cache_dir {
        CacheDir::from(path)
    } else {
        CacheDir::from_xdg()?
    };

    match cli.cmd {
        Command::CompareSrcinfo {
            pkgbuild_path,
            srcinfo_path,
        } => compare_source_info(pkgbuild_path, srcinfo_path),
        Command::TestFormat {
            repositories,
            file_type,
        } => test_files(cache_dir, file_type, repositories),
        Command::Download {
            repositories,
            source,
        } => download_files(cache_dir, source, repositories),
        Command::Clean { target } => clean_files(target, cache_dir),
    }
}

fn main() -> ExitCode {
    if let Err(error) = run_command() {
        eprintln!("{error}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
