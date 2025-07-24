//! Command-line argument handling for `alpm-linting`.

use std::path::PathBuf;

use clap::{ArgAction, Parser, ValueEnum};
use strum::Display;

use crate::LintingScopes;

/// The command-line interface handling for `alpm-linting`.
#[derive(Debug, Parser)]
#[clap(name = "ALPM Dev Scripts", about = "Dev scripts for the ALPM project")]
pub struct Cli {
    /// Verbose mode (-v, -vv)
    #[clap(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    /// The `alpm-linting` commands.
    ///
    /// Each subcommand handles linting for a specific file or context.
    #[clap(subcommand)]
    pub cmd: Command,
}

/// The `alpm-linting` commands.
///
/// Each subcommand handles linting for a specific file or context.
#[derive(Debug, Parser)]
pub enum Command {
    /// Run lints on `.SRCINFO` files.
    Srcinfo {},
}
