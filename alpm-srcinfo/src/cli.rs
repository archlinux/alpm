use std::path::PathBuf;

use alpm_types::Architecture;
use clap::Parser;
use clap::Subcommand;

#[derive(Clone, Debug, Parser)]
#[command(about, author, name = "alpm-srcinfo", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Output format for the parse command
#[derive(Clone, Debug, Default, clap::ValueEnum, strum::Display)]
pub enum OutputFormat {
    #[default]
    #[strum(serialize = "json")]
    Json,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Validate a SRCINFO file
    ///
    /// Validate a SRCINFO file according to a schema.
    /// If the file can be validated, the program exits with no output and a return code of 0.
    /// If the file can not be validated, an error is emitted on stderr and the program exits with
    /// a non-zero exit code.
    #[command()]
    Validate {
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },
    /// Read an SRCINFO file and return all packages in their final representation for a specific
    /// architecture.
    ///
    /// Reads and validates an SRCINFO file according to a schema and outputs it in another file
    /// format (currently, only JSON is supported). If the file can be validated, the program
    /// exits with the data returned in another file format on stdout and a return code of 0.
    /// If the file can not be validated, an error is emitted on stderr and the program exits with
    /// a non-zero exit code.
    #[command()]
    Format {
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// The selected architecture that should be used to interpret the SRCINFO file.
        /// Only packages that are specified for this architecture
        #[arg(short, long)]
        architecture: Architecture,

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
    /// Read an SRCINFO file and perform linter checks on it.
    ///
    /// This ensures that the SRCINFO file is both **valid** and adheres to currently known best
    /// practices.
    ///
    /// This returns a non-zero exit code as soon as any lint issue is found.
    #[command()]
    Check {
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },
}
