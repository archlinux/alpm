//! Command line argument handling.

use std::path::PathBuf;

use alpm_types::SonameLookupDirectory;
use clap::{ArgAction, Parser, Subcommand};

/// Command line argument handling for the `alpm-soname` executable.
#[derive(Clone, Debug, Parser)]
#[command(about, author, name = "alpm-soname", version)]
pub struct Cli {
    /// Log verbosity level
    #[clap(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    /// Available subcommands
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
    /// Generate depend and provide entries by reading shared libraries
    AutoDeps {
        /// Arguments for the autodeps command
        #[command(flatten)]
        args: AutoDepsArgs,
    },
}

/// Command line arguments for provision functionality.
#[derive(Clone, Debug, Parser)]
#[command(about = "Finds ALPM soname provisions", author, version)]
pub struct ProvisionArgs {
    /// The lookup directory for shared libraries in `<prefix>:<directory>` format
    ///
    /// Example: `lib:/usr/lib`
    #[arg(short, long, default_value = "lib:/usr/lib", value_name = "LOOKUP_DIR")]
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
    #[arg(short, long)]
    pub all: bool,

    /// The lookup directory for shared libraries in `<prefix>:<directory>` format
    #[arg(short, long, default_value = "lib:/usr/lib", value_name = "LOOKUP_DIR")]
    pub lookup_dir: SonameLookupDirectory,

    /// The package to inspect
    #[arg(value_name = "PACKAGE")]
    pub package: PathBuf,
}

/// Command line arguments for autodeps functionality.
#[derive(Clone, Debug, Parser)]
#[command(
    about = "Generate depend and provide entries by reading shared libraries",
    author,
    version
)]
pub struct AutoDepsArgs {
    /// The lookup directory for shared libraries in `<prefix>:<directory>` format
    ///
    /// Example: `lib:/usr/lib`
    #[arg(short, long, default_value = "lib:/usr/lib", value_name = "LOOKUP_DIR")]
    pub lookup_dir: SonameLookupDirectory,

    /// Only print provides.
    #[arg(short, long, conflicts_with = "depends")]
    pub provides: bool,

    /// Only print depends.
    #[arg(short, long, conflicts_with = "provides")]
    pub depends: bool,

    /// Suppress additional output, only print the depend/provide.
    #[arg(short, long)]
    pub quiet: bool,

    // The autodeps feature in pacman only generates depend entries if the
    // generated depend has a local satisfier (a package is installed with the matching provide)
    //
    // This can be checked with pacman via `pacman -T <depend>`.
    // Or via alpm with `alpm_find_satisfier(alpm_db_get_pkgcache(localdb), depend)`.
    //
    // This doesn't really effect the library portion of this command as the unsatisfied
    // deps and be filtered out at a higher level.
    //
    // But for the command line, it's useful to test the output against what pacman
    // would generate.
    //
    // As we don't want to link to alpm.so or require pacman be installed, this argument
    // provides a clunky way to enable this functionality by passing `--satisfied-command 'pacman
    // -T --'`.
    //
    // This argument should be removed down the line when we have a pacman alternative but is
    // useful for now.
    /// Only print dependencies that are satisfied.
    ///
    /// If this argument is specified, instead of printing all dependencies,
    /// each dependency is passed to the specified command. the dependency is
    /// only printed if the command succeeds (returns 0).
    #[arg(long, value_name = "COMMAND")]
    pub test_satisfied_command: Option<String>,

    /// The package input directory to inspect
    #[arg(value_name = "PKGDIR")]
    pub package: PathBuf,
}
