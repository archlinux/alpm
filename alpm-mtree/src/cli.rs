use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use crate::MtreeSchema;

/// The command-line interface handling for `alpm-mtree`.
#[derive(Clone, Debug, Parser)]
#[command(about, author, name = "alpm-mtree", version)]
pub struct Cli {
    /// The `alpm-mtree` commands.
    #[command(subcommand)]
    pub command: Command,
}

/// Output format for the parse command
#[derive(Clone, Debug, Default, strum::Display, ValueEnum)]
pub enum OutputFormat {
    /// The JSON output format.
    #[default]
    #[strum(serialize = "json")]
    Json,
}

/// The `alpm-mtree` commands.
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
        /// An optional file to read from.
        ///
        /// If no file is provided, stdin is used instead.
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// Provide the MTREE schema version to use.
        ///
        /// If no schema version is provided, it will be deduced from the file itself.
        ///
        /// Valid values are ['1', '2'].
        #[arg(short, long, value_name = "VERSION")]
        schema: Option<MtreeSchema>,

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
        /// An optional file to read from.
        ///
        /// If no file is provided, stdin is used instead.
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// Provide the MTREE schema version to use.
        ///
        /// If no schema version is provided, it will be deduced from the file itself.
        ///
        /// Valid values are ['1', '2'].
        #[arg(short, long, value_name = "VERSION")]
        schema: Option<MtreeSchema>,
    },
}
