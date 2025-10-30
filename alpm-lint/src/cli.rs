//! Command-line argument handling for `alpm-lint`.

use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use strum::Display;

use crate::{Level, LintScope};

/// Output format for the `alpm-lint check` subcommand.
#[derive(Clone, Debug, Display, ValueEnum)]
pub enum LintOutputFormat {
    /// Human readable text
    #[strum(to_string = "text")]
    Text,

    /// The JSON output format.
    #[strum(to_string = "json")]
    Json,
}

/// Output format for all subcommands that only output data.
#[derive(Clone, Debug, Display, ValueEnum)]
pub enum OutputFormat {
    /// The JSON output format.
    #[strum(to_string = "json")]
    Json,
}

/// The command-line interface handling for `alpm-lint`.
#[derive(Debug, Parser)]
#[clap(
    about = "Linting for ALPM related files.",
    author,
    name = "alpm-lint",
    version
)]
pub struct Cli {
    /// Log verbosity level
    #[command(flatten)]
    pub verbose: clap_verbosity::Verbosity,

    /// The `alpm-lint` commands.
    ///
    /// Each subcommand handles linting for a specific file or context.
    #[clap(subcommand)]
    pub command: Command,
}

/// The `alpm-lint` subcommands.
#[derive(Debug, Parser)]
pub enum Command {
    /// Run lints on a file or directory.
    ///
    /// By default, `alpm-lint` will try to determine the current linting scope based on the
    /// provided filename or on available files in the provided directory.
    Check {
        /// An optional path to a file or directory to be linted.
        #[arg(value_name = "DIR")]
        path: Option<PathBuf>,

        /// Explicitly define the linting scope.
        ///
        /// Using this option overrides the automatic detection.
        #[arg(short, long, value_name = "SCOPE")]
        scope: Option<LintScope>,

        /// The output format to use.
        ///
        /// If none is provided, any issues will be printed in human readable form.
        #[arg(short, long, value_name = "CHECK_FORMAT", default_value_t = LintOutputFormat::Text)]
        format: LintOutputFormat,

        /// Pretty-print the output.
        ///
        /// Has no effect if the output format can not be pretty printed.
        #[arg(short, long)]
        pretty: bool,

        /// The level of lints to consider.
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

        /// Pretty-print the output.
        ///
        /// Has no effect if the output format can not be pretty printed.
        #[arg(short, long)]
        pretty: bool,

        /// Optional output file path.
        ///
        /// If not provided, output goes to stdout.
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Return the definition of all options to configure individual linting rules.
    ///
    /// By default the definitions are returned on stdout.
    Options {
        /// The output format to use.
        #[arg(
            short,
            long,
            value_name = "FORMAT",
            default_value_t = OutputFormat::Json
        )]
        format: OutputFormat,

        /// Pretty-print the output.
        ///
        /// Has no effect if the output format can not be pretty printed.
        #[arg(short, long)]
        pretty: bool,

        /// Optional output file path.
        ///
        /// Instructs the output to be written to the provided file path.
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Return metadata about available lint groups, scopes, and levels for static site generators.
    ///
    /// By default the metadata is returned on stdout.
    Meta {
        /// The output format to use.
        #[arg(
            short,
            long,
            value_name = "FORMAT",
            default_value_t = OutputFormat::Json
        )]
        format: OutputFormat,

        /// Pretty-print the output.
        ///
        /// Has no effect if the output format can not be pretty printed.
        #[arg(short, long)]
        pretty: bool,

        /// Optional output file path.
        ///
        /// Instructs the output to be written to the provided file path.
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },
}
