//! The `dev-scripts` CLI tool.

use anyhow::{Context, Result};
use clap::Parser;
use cli::Cli;
use log::LevelFilter;
use simplelog::{Config, SimpleLogger};

use crate::commands::{compare_source_info, test_files};

mod cli;
mod cmd;
mod commands;
pub mod sync;
pub mod testing;
mod ui;

fn main() -> Result<()> {
    // Parse commandline options.
    let args = Cli::parse();

    // Init and set the verbosity level of the logger.
    let level = match args.verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    SimpleLogger::init(level, Config::default()).context("Failed to initialize simple logger")?;

    match args.cmd {
        cli::Command::TestFiles { cmd } => test_files(cmd)?,
        cli::Command::CompareSrcinfo {
            pkgbuild_path,
            srcinfo_path,
        } => compare_source_info(pkgbuild_path, srcinfo_path)?,
    }

    Ok(())
}
