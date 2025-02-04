use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Clone, Debug, Parser)]
#[command(about, author, name = "alpm-mtree", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Output format for the parse command
#[derive(Clone, Debug, Default, ValueEnum, strum::Display)]
pub enum OutputFormat {
    #[default]
    Json,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Read an MTREE file and return it in another file format
    ///
    /// Reads and validates an MTREE file according to a schema and outputs it in another file
    /// format (currently, only JSON is supported). If the file can be validated, the program
    /// exits with the data returned in another file format on stdout and a return code of 0.
    /// If the file can not be validated, an error is emitted on stderr and the program exits with
    /// a non-zero exit code.
    #[command()]
    Format {
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// Provide the output format
        #[arg(
            short,
            long,
            value_name = "OUTPUT_FORMAT",
            default_value_t = OutputFormat::Json
        )]
        output_format: OutputFormat,

        /// Determines whether the output will be displayed in a pretty non-minimized fashion.
        ///
        /// Only applies to formats that support pretty output, otherwise it's just ignored.
        #[arg(short, long)]
        pretty: bool,
    },
    /// Validate an MTREE file
    ///
    /// Validate an MTREE file according to a schema.
    /// If the file can be validated, the program exits with no output and a return code of 0.
    /// If the file can not be validated, an error is emitted on stderr and the program exits with
    /// a non-zero exit code.
    #[command()]
    Validate {
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },
}
