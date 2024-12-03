use std::fmt::Display;
use std::fmt::Formatter;
use std::path::PathBuf;
use std::str::FromStr;

use alpm_types::Architecture;
use alpm_types::BuildDate;
use alpm_types::BuildDir;
use alpm_types::BuildEnv;
use alpm_types::BuildTool;
use alpm_types::BuildToolVersion;
use alpm_types::InstalledPackage;
use alpm_types::Name;
use alpm_types::PackageOption;
use alpm_types::Packager;
use alpm_types::StartDir;
use alpm_types::Version;
use clap::Args;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use strum::Display;

use crate::schema::Schema;
use crate::Error;

/// A type wrapping a PathBuf with a default value
///
/// This type is used in circumstances where an output file is required that defaults to
/// ".BUILDINFO"
#[derive(Clone, Debug)]
pub struct OutputFile(pub PathBuf);

impl Default for OutputFile {
    fn default() -> Self {
        OutputFile(PathBuf::from(".BUILDINFO"))
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

#[derive(Clone, Debug, Parser)]
#[command(about, author, name = "alpm-buildinfo", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Create a BUILDINFO file according to a schema
    ///
    /// If the input can be validated according to the schema, the program exits with no output and
    /// a return code of 0. If the input can not be validated according to the schema, an error
    /// is emitted on stderr and the program exits with a non-zero exit code.
    #[command()]
    Create {
        #[command(subcommand)]
        command: CreateCommand,
    },

    /// Validate a BUILDINFO file
    ///
    /// Validate a BUILDINFO file according to a schema.
    /// If the file can be validated, the program exits with no output and a return code of 0.
    /// If the file can not be validated, an error is emitted on stderr and the program exits with
    /// a non-zero exit code.
    #[command()]
    Validate {
        #[command(flatten)]
        args: ValidateArgs,
    },

    /// Parse a BUILDINFO file and output it in a different format
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
    },
}

/// Arguments for validating and parsing a BUILDINFO file
#[derive(Clone, Debug, Args)]
pub struct ValidateArgs {
    /// Provide the BUILDINFO schema version to use.
    ///
    /// If no schema version is provided, it will be deduced from the file itself.
    #[arg(short, long, value_name = "VERSION")]
    pub schema: Option<Schema>,
    /// Provide the file to read
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,
}

/// Arguments for creating a BUILDINFO file according to the format version 1 schema
///
/// This struct is defined separately for re-using it for both v1 and v2 since they have
/// an overlapping set of fields.
#[derive(Clone, Debug, Args)]
pub struct V1CreateArgs {
    /// Provide a builddate
    #[arg(env = "BUILDINFO_BUILDDATE", long, value_name = "BUILDDATE")]
    pub builddate: BuildDate,
    /// Provide a builddir
    #[arg(env = "BUILDINFO_BUILDDIR", long, value_name = "BUILDDIR")]
    pub builddir: BuildDir,
    /// Provide one or more buildenv
    #[arg(
        env = "BUILDINFO_BUILDENV",
        long,
        value_delimiter = ' ',
        value_name = "BUILDENV"
    )]
    pub buildenv: Vec<BuildEnv>,
    /// Provide one or more installed
    #[arg(
        env = "BUILDINFO_INSTALLED",
        long,
        value_delimiter = ' ',
        value_name = "INSTALLED"
    )]
    pub installed: Vec<InstalledPackage>,
    /// Provide one or more options
    #[arg(
        env = "BUILDINFO_OPTIONS",
        long,
        value_delimiter = ' ',
        value_name = "OPTIONS"
    )]
    pub options: Vec<PackageOption>,
    /// Provide a packager
    #[arg(env = "BUILDINFO_PACKAGER", long, value_name = "PACKAGER")]
    pub packager: Packager,
    /// Provide a pkgarch
    #[arg(env = "BUILDINFO_PKGARCH", long, value_name = "PKGARCH")]
    pub pkgarch: Architecture,
    /// Provide a pkgbase
    #[arg(env = "BUILDINFO_PKGBASE", long, value_name = "PKGBASE")]
    pub pkgbase: Name,
    /// Provide a pkgbuild_sha256sum
    #[arg(
        env = "BUILDINFO_PKGBUILD_SHA256SUM",
        long,
        value_name = "PKGBUILD_SHA256SUM"
    )]
    pub pkgbuild_sha256sum: String,
    /// Provide a pkgname
    #[arg(env = "BUILDINFO_PKGNAME", long, value_name = "PKGNAME")]
    pub pkgname: Name,
    /// Provide a pkgver
    #[arg(env = "BUILDINFO_PKGVER", long, value_name = "PKGVER")]
    pub pkgver: Version,
    /// Provide a file to write to
    #[arg(default_value_t = OutputFile::default(), value_name = "FILE")]
    pub output: OutputFile,
}

/// Create an BUILDINFO file according to a schema
///
/// If the input can be validated according to the schema, the program exits with no output and
/// a return code of 0. If the input can not be validated according to the schema, an error
/// is emitted on stderr and the program exits with a non-zero exit code.
#[derive(Clone, Debug, Subcommand)]
pub enum CreateCommand {
    /// Create a BUILDINFO version 1 file
    V1 {
        #[command(flatten)]
        args: V1CreateArgs,
    },
    /// Create a BUILDINFO version 2 file
    V2 {
        #[command(flatten)]
        args: V1CreateArgs,

        /// Provide a startdir
        #[arg(env = "BUILDINFO_STARTDIR", long, value_name = "STARTDIR")]
        startdir: StartDir,

        /// Provide a buildtool
        #[arg(env = "BUILDINFO_BUILDTOOL", long, value_name = "BUILDTOOL")]
        buildtool: BuildTool,

        /// Provide a buildtoolver
        #[arg(env = "BUILDINFO_BUILDTOOLVER", long, value_name = "BUILDTOOLVER")]
        buildtoolver: BuildToolVersion,
    },
}

/// Output format for the format command
#[derive(Clone, Debug, Default, Display, ValueEnum)]
#[non_exhaustive]
pub enum OutputFormat {
    #[default]
    #[strum(to_string = "json")]
    Json,
}
