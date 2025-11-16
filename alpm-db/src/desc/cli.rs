//! Command line interface setup for the `alpm-db-desc` executable.

use std::path::PathBuf;

use alpm_types::{
    Architecture,
    BuildDate,
    ExtraDataEntry,
    Group,
    InstalledSize,
    License,
    Name,
    OptionalDependency,
    PackageBaseName,
    PackageDescription,
    PackageInstallReason,
    PackageRelation,
    PackageValidation,
    Packager,
    RelationOrSoname,
    Url,
    Version,
};
use clap::{Args, Parser, Subcommand, ValueEnum};
use strum::Display;

use crate::desc::schema::DbDescSchema;

/// The command-line interface handling for `alpm-db-desc`.
#[derive(Clone, Debug, Parser)]
#[command(
    about = "Command line tool to handle alpm-db-desc files",
    author,
    name = "alpm-db-desc",
    version
)]
pub struct Cli {
    /// The `alpm-db-desc` commands.
    #[command(subcommand)]
    pub command: Command,
}

/// The `alpm-db-desc` commands.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Create a DB desc file according to a schema.
    #[command()]
    Create {
        /// The create subcommand.
        #[command(subcommand)]
        command: CreateCommand,
    },

    /// Validate a DB desc file according to a schema.
    #[command()]
    Validate {
        /// The validate arguments.
        #[command(flatten)]
        args: ValidateArgs,
    },

    /// Parse and output a DB desc file in a different format.
    #[command()]
    Format {
        /// The validate arguments.
        #[command(flatten)]
        args: ValidateArgs,

        /// Provide the output format.
        #[arg(
            short,
            long,
            value_name = "OUTPUT_FORMAT",
            default_value_t = OutputFormat::Json
        )]
        output_format: OutputFormat,

        /// Pretty-print the output.
        #[arg(short, long)]
        pretty: bool,
    },
}

/// Arguments for validating and parsing a DB desc file.
#[derive(Args, Clone, Debug)]
pub struct ValidateArgs {
    /// Provide the schema version to use.
    ///
    /// If not provided, the schema version will be detected from the file.
    #[arg(short, long, value_name = "VERSION")]
    pub schema: Option<DbDescSchema>,

    /// Provide the file to read.
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,
}

/// Arguments for creating a DB desc file according to the v1 schema.
#[derive(Args, Clone, Debug)]
pub struct V1CreateArgs {
    /// The package name.
    #[arg(env = "ALPM_DB_DESC_NAME", long)]
    pub name: Name,

    /// The package version.
    #[arg(env = "ALPM_DB_DESC_VERSION", long)]
    pub version: Version,

    /// The package base.
    #[arg(env = "ALPM_DB_DESC_BASE", long)]
    pub base: PackageBaseName,

    /// The package description.
    ///
    /// If not provided, an empty string is used.
    #[arg(env = "ALPM_DB_DESC_DESC", long)]
    pub description: Option<PackageDescription>,

    /// The package URL.
    #[arg(env = "ALPM_DB_DESC_URL", long)]
    pub url: Option<Url>,

    /// The package architecture.
    #[arg(env = "ALPM_DB_DESC_ARCH", long)]
    pub arch: Architecture,

    /// The package build date.
    #[arg(env = "ALPM_DB_DESC_BUILDDATE", long)]
    pub builddate: BuildDate,

    /// The package install date.
    #[arg(env = "ALPM_DB_DESC_INSTALLDATE", long)]
    pub installdate: BuildDate,

    /// The packager.
    #[arg(env = "ALPM_DB_DESC_PACKAGER", long)]
    pub packager: Packager,

    /// The installed size.
    #[arg(env = "ALPM_DB_DESC_SIZE", long)]
    pub size: InstalledSize,

    /// The package groups.
    #[arg(env = "ALPM_DB_DESC_GROUPS", long, value_delimiter = ' ')]
    pub groups: Vec<Group>,

    /// The package install reason.
    ///
    /// If unset, assumes an install reason of "0" (explicitly installed).
    #[arg(env = "ALPM_DB_DESC_REASON", long)]
    pub reason: Option<PackageInstallReason>,

    /// The package licenses.
    #[arg(env = "ALPM_DB_DESC_LICENSE", long, value_delimiter = ' ')]
    pub license: Vec<License>,

    /// The package validation methods.
    #[arg(env = "ALPM_DB_DESC_VALIDATION", long)]
    pub validation: PackageValidation,

    /// The replaces.
    #[arg(env = "ALPM_DB_DESC_REPLACES", long, value_delimiter = ' ')]
    pub replaces: Vec<PackageRelation>,

    /// The dependencies.
    #[arg(env = "ALPM_DB_DESC_DEPENDS", long, value_delimiter = ' ')]
    pub depends: Vec<RelationOrSoname>,

    /// The optional dependencies.
    #[arg(env = "ALPM_DB_DESC_OPTDEPENDS", long, value_delimiter = ' ')]
    pub optdepends: Vec<OptionalDependency>,

    /// The conflicts.
    #[arg(env = "ALPM_DB_DESC_CONFLICTS", long, value_delimiter = ' ')]
    pub conflicts: Vec<PackageRelation>,

    /// The provides.
    #[arg(env = "ALPM_DB_DESC_PROVIDES", long, value_delimiter = ' ')]
    pub provides: Vec<RelationOrSoname>,

    /// The output file.
    #[arg(env = "ALPM_DB_DESC_OUTPUT_FILE")]
    pub output: Option<PathBuf>,
}

/// Create a DB desc file according to a schema.
#[derive(Clone, Debug, Subcommand)]
pub enum CreateCommand {
    /// Create a DB desc version 1 file.
    V1 {
        /// The create arguments.
        #[command(flatten)]
        args: V1CreateArgs,
    },

    /// Create a DB desc version 2 file.
    V2 {
        /// The create arguments.
        #[command(flatten)]
        args: V1CreateArgs,

        /// Structured extra metadata
        #[arg(
            env = "ALPM_DB_DESC_XDATA",
            long,
            value_delimiter = ' ',
            required = true
        )]
        xdata: Vec<ExtraDataEntry>,
    },
}

/// Output format for the format command.
#[derive(Clone, Debug, Default, Display, ValueEnum)]
pub enum OutputFormat {
    /// The JSON output format.
    #[default]
    #[strum(to_string = "json")]
    Json,
}
