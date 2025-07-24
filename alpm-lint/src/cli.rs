//! Command-line argument handling for `alpm-linting`.

use std::path::PathBuf;

use clap::{ArgAction, Parser, ValueEnum};
use strum::Display;

use crate::LintScope;

/// Output format for the `rules` subcommand.
#[derive(Clone, Debug, Default, Display, ValueEnum)]
pub enum OutputFormat {
    /// The JSON output format.
    #[default]
    #[strum(to_string = "json")]
    Json,

    /// The TOML output format.
    #[strum(to_string = "toml")]
    Toml,
}

/// The command-line interface handling for `alpm-linting`.
#[derive(Debug, Parser)]
#[clap(name = "ALPM linting", about = "Linting for ALPM related files.")]
pub struct Cli {
    /// Verbose mode (-v, -vv)
    #[clap(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    /// The `alpm-linting` commands.
    ///
    /// Each subcommand handles linting for a specific file or context.
    #[clap(subcommand)]
    pub command: Command,
}

/// The `alpm-linting` sub-commands.
#[derive(Debug, Parser)]
pub enum Command {
    /// Run lints on a file or directory.
    ///
    /// By default, alpm-linting will try to determine the current linting scope based the given
    /// filename or on available files in the given directory.
    Check {
        /// An optional path to a file/directory to be linted.
        #[arg(short, long, value_name = "DIR")]
        path: Option<PathBuf>,

        /// Explicitly define the linting scope. This overrides the automatic detection.
        #[arg(short, long, value_name = "SCOPE")]
        scope: Option<LintScope>,
    },

    /// Return the definition of all lint rules as structured data.
    Rules {
        /// The output format to use.
        #[arg(
            short,
            long,
            value_name = "OUTPUT_FORMAT",
            default_value_t = OutputFormat::Json
        )]
        output_format: OutputFormat,

        /// Pretty-print the output
        ///
        /// Has no effect if the output format can not be pretty printed.
        #[arg(short, long)]
        pretty: bool,
    },
    /// Return the definition of all options to configure individual linting rules.
    Options {
        /// The output format to use.
        #[arg(
            short,
            long,
            value_name = "OUTPUT_FORMAT",
            default_value_t = OutputFormat::Json
        )]
        output_format: OutputFormat,

        /// Pretty-print the output
        ///
        /// Has no effect if the output format can not be pretty printed.
        #[arg(short, long)]
        pretty: bool,
    },
}
