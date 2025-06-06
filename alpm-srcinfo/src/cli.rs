//! Commandline argument handling.
use std::path::PathBuf;

use alpm_types::Architecture;
use clap::{Parser, Subcommand};

use crate::SourceInfoSchema;

#[derive(Clone, Debug, Parser)]
#[command(about, author, name = "alpm-srcinfo", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Output format for the parse command
#[derive(Clone, Debug, Default, strum::Display, clap::ValueEnum)]
pub enum OutputFormat {
    #[default]
    #[strum(serialize = "json")]
    Json,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Validate a SRCINFO file from a path or `stdin`.
    ///
    /// If the file can be validated, the program exits with no output and a return code of 0.
    /// If the file can not be validated, an error is emitted on stderr and the program exits with
    /// a non-zero exit status.
    #[command()]
    Validate {
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// Provide the SRCINFO schema version to use.
        ///
        /// If no schema version is provided, it will be deduced from the file itself.
        #[arg(short, long, value_name = "VERSION")]
        schema: Option<SourceInfoSchema>,
    },
    /// Format a SRCINFO file from a path or `stdin`
    ///
    /// Read, validate and print all of the SRCINFO's packages in their final representation for a
    /// specific architecture. If the file is valid, the program prints the data in the
    /// requested file format to stdout and returns with an exit status of 0.
    #[command()]
    FormatPackages {
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// Provide the SRCINFO schema version to use.
        ///
        /// If no schema version is provided, it will be deduced from the file itself.
        #[arg(short, long, value_name = "VERSION")]
        schema: Option<SourceInfoSchema>,

        /// The selected architecture that should be used to interpret the SRCINFO file.
        ///
        /// Only [split-]packages that are applicable for this architecture will be returned.
        #[arg(short, long, alias = "arch")]
        architecture: Architecture,

        /// Provide the output format
        #[arg(
            short,
            long,
            value_name = "OUTPUT_FORMAT",
            default_value_t = OutputFormat::Json
        )]
        output_format: OutputFormat,

        /// Pretty-print the output.
        ///
        /// Only applies to formats that support pretty output and is otherwise ignored.
        #[arg(short, long)]
        pretty: bool,
    },
}
