use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
    str::FromStr,
};

use alpm_types::{
    Architecture,
    BuildDate,
    ExtraData,
    Group,
    InstalledSize,
    License,
    Name,
    OptionalDependency,
    PackageBaseName,
    PackageDescription,
    PackageRelation,
    Packager,
    Url,
    Version,
};
use clap::{Args, Parser, Subcommand, ValueEnum};
use strum::Display;

use crate::Error;

/// A type wrapping a PathBuf with a default value
///
/// This type is used in circumstances where an output file is required that defaults to
/// ".PKGINFO"
#[derive(Clone, Debug)]
pub struct OutputFile(pub PathBuf);

impl Default for OutputFile {
    fn default() -> Self {
        OutputFile(PathBuf::from(".DESC"))
    }
}

impl Display for OutputFile {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.0.display())
    }
}

impl FromStr for OutputFile {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(OutputFile(PathBuf::from(s)))
    }
}

/// The command-line interface handling for `alpm-db-desc`.
#[derive(Clone, Debug, Parser)]
#[command(about, author, name = "alpm-db-desc", version)]
pub struct Cli {
    /// The `alpm-db-desc` commands.
    #[command(subcommand)]
    pub command: Command,
}

/// The `alpm-db-desc` commands.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Create a DB DESC file according to a schema
    #[command()]
    Create {
        #[command(subcommand)]
        command: CreateCommand,
    },

    /// Validate a DB DESC file according to a schema
    #[command()]
    Validate {
        #[command(flatten)]
        args: ValidateArgs,
    },

    /// Parse and output a DB DESC file in a different format
    #[command()]
    Format {
        #[command(flatten)]
        args: ValidateArgs,

        /// Provide the output format
        #[arg(
            short,
            long,
            value_name = "OUTPUT_FORMAT",
            default_value_t = OutputFormat::Json
        )]
        output_format: OutputFormat,

        /// Pretty-print the output
        #[arg(short, long)]
        pretty: bool,
    },
}

/// Arguments for validating and parsing a DB DESC file
#[derive(Args, Clone, Debug)]
pub struct ValidateArgs {
    /// Provide the file to read
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,
}

/// Arguments for creating a DB DESC file according to the v1 schema
#[derive(Args, Clone, Debug)]
pub struct V1CreateArgs {
    #[arg(env = "DESCFILE_NAME", long)]
    pub name: Name,

    #[arg(env = "DESCFILE_VERSION", long)]
    pub version: Version,

    #[arg(env = "DESCFILE_BASE", long)]
    pub base: PackageBaseName,

    #[arg(env = "DESCFILE_DESC", long)]
    pub description: Option<PackageDescription>,

    #[arg(env = "DESCFILE_URL", long)]
    pub url: Option<Url>,

    #[arg(env = "DESCFILE_ARCH", long)]
    pub arch: Architecture,

    #[arg(env = "DESCFILE_BUILDDATE", long)]
    pub builddate: BuildDate,

    #[arg(env = "DESCFILE_INSTALLDATE", long)]
    pub installdate: BuildDate,

    #[arg(env = "DESCFILE_PACKAGER", long)]
    pub packager: Packager,

    #[arg(env = "DESCFILE_SIZE", long)]
    pub size: InstalledSize,

    #[arg(env = "DESCFILE_GROUPS", long, value_delimiter = ' ')]
    pub groups: Vec<Group>,

    #[arg(env = "DESCFILE_LICENSE", long, value_delimiter = ' ')]
    pub license: Vec<License>,

    #[arg(env = "DESCFILE_VALIDATION", long, value_delimiter = ' ')]
    pub validation: Vec<String>,

    #[arg(env = "DESCFILE_REPLACES", long, value_delimiter = ' ')]
    pub replaces: Vec<Name>,

    #[arg(env = "DESCFILE_DEPENDS", long, value_delimiter = ' ')]
    pub depends: Vec<PackageRelation>,

    #[arg(env = "DESCFILE_OPTDEPENDS", long, value_delimiter = ' ')]
    pub optdepends: Vec<OptionalDependency>,

    #[arg(env = "DESCFILE_CONFLICTS", long, value_delimiter = ' ')]
    pub conflicts: Vec<Name>,

    #[arg(env = "DESCFILE_PROVIDES", long, value_delimiter = ' ')]
    pub provides: Vec<Name>,

    #[arg(default_value_t = OutputFile::default(), env = "DESCFILE_OUTPUT_FILE")]
    pub output: OutputFile,
}

/// Create a DB DESC file according to a schema
#[derive(Clone, Debug, Subcommand)]
pub enum CreateCommand {
    /// Create a DB DESC version 1 file
    V1 {
        #[command(flatten)]
        args: V1CreateArgs,
    },

    /// Create a DB DESC version 2 file
    V2 {
        #[command(flatten)]
        args: V1CreateArgs,

        /// Structured extra metadata
        #[arg(env = "DESCFILE_XDATA", long, value_delimiter = ' ')]
        xdata: Vec<ExtraData>,
    },
}

/// Output format for the format command
#[derive(Clone, Debug, Default, Display, ValueEnum)]
#[non_exhaustive]
pub enum OutputFormat {
    /// The JSON output format.
    #[default]
    #[strum(to_string = "json")]
    Json,
}
