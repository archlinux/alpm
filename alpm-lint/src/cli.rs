//! Command-line argument handling for `alpm-lint`.

use std::path::PathBuf;

use clap::{ArgAction, Parser, ValueEnum};
use strum::Display;

use crate::{Level, LintScope};

/// Output format for all subcommand.
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

/// The command-line interface handling for `alpm-lint`.
#[derive(Debug, Parser)]
#[clap(name = "ALPM linting", about = "Linting for ALPM related files.")]
pub struct Cli {
    /// Verbose mode (-v, -vv)
    #[clap(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    /// The `alpm-lint` commands.
    ///
    /// Each subcommand handles linting for a specific file or context.
    #[clap(subcommand)]
    pub command: Command,
}

/// The `alpm-lint` sub-commands.
#[derive(Debug, Parser)]
pub enum Command {
    /// Run lints on a file or directory.
    ///
    /// By default, alpm-lint will try to determine the current linting scope based the given
    /// filename or on available files in the given directory.
    Check {
        /// An optional path to a file/directory to be linted.
        #[arg(value_name = "DIR")]
        path: Option<PathBuf>,

        /// Explicitly define the linting scope. This overrides the automatic detection.
        #[arg(short, long, value_name = "SCOPE")]
        scope: Option<LintScope>,

        /// The output format to use.
        ///
        /// If none is provided, any issues will be printed in human readable form.
        #[arg(short, long, value_name = "CHECK_FORMAT")]
        format: Option<OutputFormat>,

        /// Pretty-print the output
        ///
        /// Has no effect if the output format can not be pretty printed.
        #[arg(short, long)]
        pretty: bool,

        /// The level of lints to consider
        ///
        /// Any lints with this level and above (more severe) will be shown.
        /// If such lints are found, the command will return with an non-zero exit code.
        #[arg(short,
            long,
            value_name = "LEVEL",
            default_value_t = Level::Warn,
            )]
        level: Level,

        /// Supply a lint config path.
        ///
        /// This overwrites any options from the project wide configuration file.
        #[arg(short, long, value_name = "LEVEL")]
        config: Option<PathBuf>,

        /// Optional output file path. If not provided, output goes to stdout.
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Return the definition of all lint rules as structured data.
    Rules {
        /// The output format to use.
        #[arg(
            short,
            long,
            value_name = "FORMAT",
            default_value_t = OutputFormat::Json
        )]
        format: OutputFormat,

        /// Pretty-print the output
        ///
        /// Has no effect if the output format can not be pretty printed.
        #[arg(short, long)]
        pretty: bool,

        /// Optional output file path. If not provided, output goes to stdout.
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Return the definition of all options to configure individual linting rules.
    Options {
        /// The output format to use.
        #[arg(
            short,
            long,
            value_name = "FORMAT",
            default_value_t = OutputFormat::Json
        )]
        format: OutputFormat,

        /// Pretty-print the output
        ///
        /// Has no effect if the output format can not be pretty printed.
        #[arg(short, long)]
        pretty: bool,

        /// Optional output file path. If not provided, output goes to stdout.
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Return metadata about available lint groups, scopes, and levels for static site generators.
    Meta {
        /// The output format to use.
        #[arg(
            short,
            long,
            value_name = "FORMAT",
            default_value_t = OutputFormat::Json
        )]
        format: OutputFormat,

        /// Pretty-print the output
        ///
        /// Has no effect if the output format can not be pretty printed.
        #[arg(short, long)]
        pretty: bool,

        /// Optional output file path. If not provided, output goes to stdout.
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },
}
