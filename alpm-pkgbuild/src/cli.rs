//! Definition of the commandline logic.

use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand, ValueEnum};

/// The commandline arguments for the `alpm-pkgbuild` binary.
#[derive(Clone, Debug, Parser)]
#[command(about, author, name = "alpm-mtree", version)]
pub struct Cli {
    /// Verbosity of the output.
    ///
    /// Provide multiple times for increased verbosity.
    #[arg(short, long, action = ArgAction::Count)]
    pub verbosity: u8,

    /// The top-level subcommand.
    #[command(subcommand)]
    pub command: Command,
}

/// Output format for the parse command
#[derive(Clone, Debug, Default, strum::Display, ValueEnum)]
pub enum OutputFormat {
    /// JSON
    #[strum(serialize = "json")]
    Json,

    /// The ALPM [SRCINFO] output format.
    ///
    /// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
    #[default]
    #[strum(serialize = "srcinfo")]
    Srcinfo,
}

/// The top-level subcommand for the `alpm-pkgbuild` binary.
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// SRCINFO related actions.
    Srcinfo {
        /// The subcommand for the `srcinfo` subcommand.
        #[command(subcommand)]
        subcommand: SourceInfoCommand,
    },
}

/// The nested `srcinfo` subcommand for the `alpm-pkgbuild` binary.
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

    /// Compare the generated SRCINFO from a PKGBUILD with an existing .SRCINFO file
    #[command()]
    Compare {
        /// Path to the PKGBUILD file.
        #[arg(
            short,
            long = "pkgbuild",
            value_name = "PKGBUILD_PATH",
            default_value = "./PKGBUILD"
        )]
        pkgbuild_path: PathBuf,

        /// Path to the .SRCINFO file.
        #[arg(
            short,
            long = "srcinfo",
            value_name = "SRCINFO_PATH",
            default_value = "./.SRCINFO"
        )]
        srcinfo_path: PathBuf,
    },
}
