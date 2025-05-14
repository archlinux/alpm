//! Definition of the commandline logic.

use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand, ValueEnum};

#[derive(Clone, Debug, Parser)]
#[command(about, author, name = "alpm-mtree", version)]
pub struct Cli {
    /// Verbose mode (-v, -vv, -vvv)
    #[arg(short, long, action = ArgAction::Count)]
    pub verbosity: u8,

    #[command(subcommand)]
    pub command: Command,
}

/// Output format for the parse command
#[derive(Clone, Debug, Default, ValueEnum, strum::Display)]
pub enum OutputFormat {
    #[strum(serialize = "json")]
    Json,

    #[default]
    #[strum(serialize = "srcinfo")]
    Srcinfo,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// SRCINFO related actions.
    Srcinfo {
        #[command(subcommand)]
        subcommand: SourceInfoCommand,
    },
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Subcommand)]
pub enum SourceInfoCommand {
    /// Take a PKGBUILD file and create a SRCINFO file from it.
    #[command()]
    Format {
        /// Path to the PKGBUILD file.
        #[arg(value_name = "PKGBUILD_PATH", default_value = "./PKGBUILD")]
        pkgbuild_path: PathBuf,

        /// Provide the output format
        #[arg(
            short,
            long,
            value_name = "OUTPUT_FORMAT",
            default_value_t = OutputFormat::Srcinfo
        )]
        output_format: OutputFormat,

        /// Pretty-print the output.
        ///
        /// Only applies to formats that support pretty output and is otherwise ignored.
        #[arg(short, long)]
        pretty: bool,
    },

    /// Run the bridge script on a PKGBUILD file and print the raw and unfiltered output.
    ///
    /// This is mostly for debugging the bridge script and can be ignored in day-to-day usage.
    #[command()]
    RunBridge {
        /// Path to the PKGBUILD file.
        #[arg(value_name = "PKGBUILD_PATH", default_value = "./PKGBUILD")]
        pkgbuild_path: PathBuf,
    },
}
