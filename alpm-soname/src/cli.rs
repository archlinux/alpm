//! Command line argument handling.

use std::path::PathBuf;

use alpm_types::SonameLookupDirectory;
use clap::{Parser, Subcommand, ValueEnum};

/// Command line argument handling for the `alpm-soname` executable.
#[derive(Clone, Debug, Parser)]
#[command(about, author, name = "alpm-soname", version)]
pub struct Cli {
    /// Log verbosity level
    #[command(flatten)]
    pub verbose: clap_verbosity::Verbosity,

    /// Available subcommands
    #[command(subcommand)]
    pub command: Command,
}

/// Available commands for the `alpm-soname` executable.
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Get provisions
    #[command()]
    GetProvisions {
        /// The lookup directory for shared libraries in `<prefix>:<directory>` format
        #[arg(short, long, default_value = "lib:/usr/lib", value_name = "LOOKUP_DIR")]
        lookup_dir: SonameLookupDirectory,

        /// Package arguments for the get-provisions command
        #[command(flatten)]
        args: PackageArgs,
    },

    /// Get dependencies
    #[command()]
    GetDependencies {
        /// The lookup directory for shared libraries in `<prefix>:<directory>` format
        #[arg(short, long, default_value = "lib:/usr/lib", value_name = "LOOKUP_DIR")]
        lookup_dir: SonameLookupDirectory,

        /// Package arguments for the get-dependencies command
        #[command(flatten)]
        args: PackageArgs,
    },

    /// Get raw dependencies without filtering by lookup directory
    GetRawDependencies {
        /// Package arguments for the get-raw-dependencies command
        #[command(flatten)]
        args: PackageArgs,
    },

    /// Generate depend and provide entries by reading shared libraries
    DetectSoname {
        /// Arguments for the detect-soname command
        #[command(flatten)]
        args: SonameDetectionArgs,
    },
}

/// Command line arguments for the soname detection command.
#[derive(Clone, Debug, Parser)]
#[command(
    about = "Generate dependency and provision entries by reading shared libraries",
    author,
    version
)]
pub struct SonameDetectionArgs {
    /// Package arguments for the detect-soname command
    #[command(flatten)]
    pub package_args: PackageArgs,

    /// The lookup directory for shared libraries in `<prefix>:<directory>` format
    ///
    /// Example: `lib:/usr/lib`
    #[arg(short, long, default_value = "lib:/usr/lib", value_name = "LOOKUP_DIR")]
    pub lookup_dir: SonameLookupDirectory,

    /// Only print provisions.
    #[arg(long, conflicts_with = "dependencies")]
    pub provisions: bool,

    /// Only print dependencies.
    #[arg(long, conflicts_with = "provisions")]
    pub dependencies: bool,
}

/// Common arguments for commands that inspect a package.
#[derive(Clone, Debug, Parser)]
#[command(author, version)]
pub struct PackageArgs {
    /// The package to inspect
    #[arg(value_name = "PACKAGE")]
    pub package: PathBuf,

    /// The output format
    #[arg(
        short,
        long,
        value_name = "OUTPUT_FORMAT",
        default_value_t = OutputFormat::Plain
    )]
    pub output_format: OutputFormat,

    /// Pretty-print the output
    ///
    /// Has no effect if the output format can not be pretty printed.
    #[arg(short, long)]
    pub pretty: bool,
}

/// Output format.
#[derive(Clone, Debug, Default, strum::Display, ValueEnum)]
pub enum OutputFormat {
    /// The plain text output format (line by line).
    #[default]
    #[strum(serialize = "plain")]
    Plain,

    /// The JSON output format.
    #[strum(serialize = "json")]
    Json,
}
