//! The `dev-scripts` CLI tool.

use std::process::ExitCode;

use clap::Parser;
use cli::Cli;
use simplelog::{Config, SimpleLogger};

use crate::{
    cache::CacheDir,
    commands::{compare_source_info, test_files},
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

    match cli.cmd {
        cli::Command::TestFiles {
            cmd,
            cache_dir,
            local_db_path,
        } => {
            let cache_dir = if let Some(path) = cache_dir {
                CacheDir::from(path)
            } else {
                CacheDir::from_xdg()?
            };

            test_files(cmd, cache_dir, local_db_path)
        }
        cli::Command::CompareSrcinfo {
            pkgbuild_path,
            srcinfo_path,
        } => compare_source_info(pkgbuild_path, srcinfo_path),
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
