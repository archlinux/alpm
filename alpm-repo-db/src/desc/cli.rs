//! Command line interface setup for the `alpm-repo-desc` executable.

use std::path::PathBuf;

use alpm_types::{
    Architecture,
    Base64OpenPGPSignature,
    BuildDate,
    CompressedSize,
    FullVersion,
    Group,
    InstalledSize,
    License,
    Md5Checksum,
    Name,
    OptionalDependency,
    PackageBaseName,
    PackageDescription,
    PackageFileName,
    PackageRelation,
    Packager,
    RelationOrSoname,
    Sha256Checksum,
    Url,
};
use clap::{Args, Parser, Subcommand, ValueEnum};
use strum::Display;

use crate::desc::schema::RepoDescSchema;

/// The command-line interface handling for `alpm-repo-desc`.
#[derive(Clone, Debug, Parser)]
#[command(
    about = "Command line tool to handle alpm-repo-desc files",
    author,
    name = "alpm-repo-desc",
    version
)]
pub struct Cli {
    /// The `alpm-repo-desc` commands.
    #[command(subcommand)]
    pub command: Command,
}

/// The `alpm-repo-desc` commands.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Create a package repository desc file according to a schema.
    #[command()]
    Create {
        /// The create subcommand.
        #[command(subcommand)]
        command: CreateCommand,
    },

    /// Validate a package repository desc file according to a schema.
    #[command()]
    Validate {
        /// The validate arguments.
        #[command(flatten)]
        args: ValidateArgs,
    },

    /// Parse and output a package repository desc file in a different format.
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

/// Arguments for validating and parsing a package repository desc file.
#[derive(Args, Clone, Debug)]
pub struct ValidateArgs {
    /// Provide the schema version to use.
    ///
    /// If not provided, the schema version will be detected from the file.
    #[arg(short, long, value_name = "VERSION")]
    pub schema: Option<RepoDescSchema>,

    /// Provide the file to read.
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,
}

/// Arguments for creating a package repository desc file that are common for all versions.
#[derive(Args, Clone, Debug)]
pub struct CommonCreateArgs {
    /// The package file name.
    #[arg(env = "ALPM_REPO_DESC_FILENAME", long)]
    pub filename: PackageFileName,

    /// The package name.
    #[arg(env = "ALPM_REPO_DESC_NAME", long)]
    pub name: Name,

    /// The name of the package base, from which this package originates.
    #[arg(env = "ALPM_REPO_DESC_BASE", long)]
    pub base: PackageBaseName,

    /// The version of the package.
    #[arg(env = "ALPM_REPO_DESC_VERSION", long)]
    pub version: FullVersion,

    /// The description of the package.
    #[arg(env = "ALPM_REPO_DESC_DESC", long)]
    pub description: Option<PackageDescription>,

    /// The groups this package belongs to.
    #[arg(env = "ALPM_REPO_DESC_GROUPS", long, value_delimiter = ' ')]
    pub groups: Vec<Group>,

    /// The compressed size of the package in bytes.
    #[arg(env = "ALPM_REPO_DESC_CSIZE", long)]
    pub csize: CompressedSize,

    /// The size of the uncompressed and unpacked package contents in bytes.
    #[arg(env = "ALPM_REPO_DESC_ISIZE", long)]
    pub isize: InstalledSize,

    /// The SHA256 checksum of the package file.
    #[arg(env = "ALPM_REPO_DESC_SHA256SUM", long)]
    pub sha256sum: Sha256Checksum,

    /// The optional URL associated with the package.
    #[arg(env = "ALPM_REPO_DESC_URL", long)]
    pub url: Option<Url>,

    /// Set of licenses under which the package is distributed.
    #[arg(env = "ALPM_REPO_DESC_LICENSE", long, value_delimiter = ' ')]
    pub license: Vec<License>,

    /// The architecture of the package.
    #[arg(env = "ALPM_REPO_DESC_ARCH", long)]
    pub arch: Architecture,

    /// The date at wchich the build of the package started.
    #[arg(env = "ALPM_REPO_DESC_BUILDDATE", long)]
    pub builddate: BuildDate,

    /// The User ID of the entity, that built the package.
    #[arg(env = "ALPM_REPO_DESC_PACKAGER", long)]
    pub packager: Packager,

    /// Virtual components or packages that this package replaces upon installation.
    #[arg(env = "ALPM_REPO_DESC_REPLACES", long, value_delimiter = ' ')]
    pub replaces: Vec<PackageRelation>,

    /// Virtual components or packages that this package conflicts with.
    #[arg(env = "ALPM_REPO_DESC_CONFLICTS", long, value_delimiter = ' ')]
    pub conflicts: Vec<PackageRelation>,

    /// Virtual components or packages that this package provides.
    #[arg(env = "ALPM_REPO_DESC_PROVIDES", long, value_delimiter = ' ')]
    pub provides: Vec<RelationOrSoname>,

    /// Run-time dependencies required by the package.
    #[arg(env = "ALPM_REPO_DESC_DEPENDS", long, value_delimiter = ' ')]
    pub depends: Vec<RelationOrSoname>,

    /// Optional dependencies that are not strictly required by the package.
    ///
    /// Note: uses comma as value delimiter. If you need optional dependencies with descriptions
    /// that contain commas, use multiple arguments with quoted values.
    #[arg(env = "ALPM_REPO_DESC_OPTDEPENDS", long, value_delimiter = ',')]
    pub optdepends: Vec<OptionalDependency>,

    /// Dependencies for building the upstream software of the package.
    #[arg(env = "ALPM_REPO_DESC_MAKEDEPENDS", long, value_delimiter = ' ')]
    pub makedepends: Vec<PackageRelation>,

    /// A dependency for running tests of the package's upstream project.
    #[arg(env = "ALPM_REPO_DESC_CHECKDEPENDS", long, value_delimiter = ' ')]
    pub checkdepends: Vec<PackageRelation>,

    /// The output file.
    #[arg(env = "ALPM_REPO_DESC_OUTPUT_FILE")]
    pub output: Option<PathBuf>,
}

/// Create a package repository desc file according to a schema.
#[derive(Clone, Debug, Subcommand)]
pub enum CreateCommand {
    /// Create a package repository desc version 1 file.
    V1 {
        /// The common create arguments.
        #[command(flatten)]
        common: CommonCreateArgs,

        /// The MD5 checksum of the package file.
        #[arg(env = "ALPM_REPO_DESC_MD5SUM", long)]
        md5sum: Md5Checksum,

        /// The base64-encoded OpenPGP detached signature of the package file.
        #[arg(env = "ALPM_REPO_DESC_PGPSIG", long)]
        pgpsig: Base64OpenPGPSignature,
    },

    /// Create a package repository desc version 2 file.
    V2 {
        /// The common create arguments.
        #[command(flatten)]
        common: CommonCreateArgs,

        /// The base64-encoded OpenPGP detached signature of the package file.
        #[arg(env = "ALPM_REPO_DESC_PGPSIG", long)]
        pgpsig: Option<Base64OpenPGPSignature>,
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
