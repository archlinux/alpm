//! Command line argument handling.

use std::path::PathBuf;

use alpm_types::SonameLookupDirectory;
use clap::{Parser, Subcommand};

/// Command line argument handling for the `alpm-soname` executable.
#[derive(Clone, Debug, Parser)]
#[command(about, author, name = "alpm-soname", version)]
pub struct Cli {
    /// Available commands
    #[command(subcommand)]
    pub command: Command,
}

/// Available commands for the `alpm-soname` executable.
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Get the provisions of a package
    #[command()]
    GetProvisions {
        /// Arguments for the get-provisions command
        #[command(flatten)]
        args: ProvisionArgs,
    },

    /// Get the dependencies of a package
    #[command()]
    GetDependencies {
        /// Arguments for the get-dependencies command
        #[command(flatten)]
        args: DependencyArgs,
    },
}

/// Command line arguments for provision functionality.
#[derive(Clone, Debug, Parser)]
#[command(about = "Finds ALPM soname provisions", author, version)]
pub struct ProvisionArgs {
    /// The lookup directory for shared libraries in `<prefix>:<directory>` format
    ///
    /// Example: `lib:/usr/lib`
    #[arg(short, long, default_value = "lib:/usr/lib", value_name = "DIR")]
    pub lookup_dir: SonameLookupDirectory,

    /// The package to inspect
    #[arg(value_name = "PACKAGE")]
    pub package: PathBuf,
}

/// Command line arguments for dependency functionality.
#[derive(Clone, Debug, Parser)]
#[command(about = "Finds ALPM soname dependencies", author, version)]
pub struct DependencyArgs {
    /// Show all dependencies, even those without matching provisions
    ///
    /// TODO: Use this
    #[arg(short, long)]
    pub all: bool,

    /// The lookup directory for shared libraries in `<prefix>:<directory>` format
    #[arg(short, long, default_value = "lib:/usr/lib", value_name = "DIR")]
    pub lookup_dir: SonameLookupDirectory,

    /// The package to inspect
    #[arg(value_name = "PACKAGE")]
    pub package: PathBuf,
}
