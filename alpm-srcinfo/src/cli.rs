//! Commandline argument handling.
use std::path::PathBuf;

use alpm_types::Architecture;
use clap::{Parser, Subcommand};

use crate::SourceInfoSchema;

/// The command-line interface handling for `alpm-srcinfo`.
#[derive(Clone, Debug, Parser)]
#[command(about, author, name = "alpm-srcinfo", version)]
pub struct Cli {
    /// The `alpm-srcinfo` commands.
    #[command(subcommand)]
    pub command: Command,
}

/// Output format for the `format-packages` command.
#[derive(Clone, Debug, Default, strum::Display, clap::ValueEnum)]
pub enum PackagesOutputFormat {
    /// The JSON output format.
    #[default]
    #[strum(serialize = "json")]
    Json,
}

/// Output format for the `format` command.
#[derive(Clone, Debug, Default, strum::Display, clap::ValueEnum)]
pub enum SourceInfoOutputFormat {
    /// The JSON output format.
    #[strum(serialize = "json")]
    Json,

    /// The SRCINFO output format
    #[default]
    #[strum(serialize = "srcinfo")]
    Srcinfo,
}

/// The `alpm-srcinfo` commands.
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Create a SRCINFO file from a PKGBUILD file at a given path.
    ///
    /// If the PKGBUILD can be created and validated, the program exits with no output and a return
    /// code of 0. If the file is missing or can not be validated, an error is emitted on stderr and
    /// the program exits with a non-zero exit status.
    #[command()]
    Create {
        /// An optional input file path to read from
        ///
        /// If no file is specified, stdin is read from and expected to contain PKGINFO data to
        /// validate.
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Provide the output format
        #[arg(
            short,
            long,
            value_name = "OUTPUT_FORMAT",
            default_value_t = SourceInfoOutputFormat::Srcinfo,
        )]
        output_format: SourceInfoOutputFormat,

        /// Pretty-print the output.
        ///
        /// Only applies to formats that support pretty output and is otherwise ignored.
        #[arg(short, long)]
        pretty: bool,
    },

    /// Validate a SRCINFO file from a path or `stdin`.
    ///
    /// If the file can be validated, the program exits with no output and a return code of 0.
    /// If the file can not be validated, an error is emitted on stderr and the program exits with
    /// a non-zero exit status.
    #[command()]
    Validate {
        /// An optional input file path to read from
        ///
        /// If no file is specified, stdin is read from and expected to contain PKGINFO data to
        /// validate.
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// Provide the SRCINFO schema version to use.
        ///
        /// If no schema version is provided, it will be deduced from the file itself.
        #[arg(short, long, value_name = "VERSION")]
        schema: Option<SourceInfoSchema>,
    },

    /// Format a SRCINFO file from a path or `stdin`.
    ///
    /// If the file is valid, the program prints the data in the
    /// requested file format to stdout and returns with an exit status of 0.
    #[command()]
    Format {
        /// The file to read from.
        ///
        /// If no file is provided, stdin is used instead.
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// Provide the SRCINFO schema version to use.
        ///
        /// If no schema version is provided, it will be deduced from the file itself.
        #[arg(short, long, value_name = "VERSION")]
        schema: Option<SourceInfoSchema>,

        /// Provide the output format
        #[arg(
            short,
            long,
            value_name = "OUTPUT_FORMAT",
            default_value_t = SourceInfoOutputFormat::Srcinfo,
        )]
        output_format: SourceInfoOutputFormat,

        /// Pretty-print the output.
        ///
        /// Only applies to formats that support pretty output and is otherwise ignored.
        #[arg(short, long)]
        pretty: bool,
    },

    /// Format a SRCINFO file's packages from a path or `stdin`
    ///
    /// Read, validate and print all of the SRCINFO's packages in their final representation for a
    /// specific architecture. If the file is valid, the program prints the data in the
    /// requested file format to stdout and returns with an exit status of 0.
    #[command()]
    FormatPackages {
        /// An optional input file path to read from
        ///
        /// If no file is specified, stdin is read from and expected to contain PKGINFO data to
        /// validate.
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
            default_value_t = PackagesOutputFormat::Json
        )]
        output_format: PackagesOutputFormat,

        /// Pretty-print the output.
        ///
        /// Only applies to formats that support pretty output and is otherwise ignored.
        #[arg(short, long)]
        pretty: bool,
    },
}
