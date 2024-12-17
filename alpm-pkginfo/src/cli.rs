use std::fmt::Display;
use std::fmt::Formatter;
use std::path::PathBuf;
use std::str::FromStr;

use alpm_types::Architecture;
use alpm_types::Backup;
use alpm_types::BuildDate;
use alpm_types::ExtraData;
use alpm_types::Group;
use alpm_types::InstalledSize;
use alpm_types::License;
use alpm_types::Name;
use alpm_types::OptDepend;
use alpm_types::PackageRelation;
use alpm_types::Packager;
use alpm_types::PkgDesc;
use alpm_types::Url;
use alpm_types::Version;
use clap::Args;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
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
        OutputFile(PathBuf::from(".PKGINFO"))
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
#[command(about, author, name = "alpm-pkginfo", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    #[command()]
    /// Create a PKGINFO file according to a schema
    ///
    /// If the input can be validated according to the schema, the program exits with no output and
    /// a return code of 0. If the input can not be validated according to the schema, an error
    /// is emitted on stderr and the program exits with a non-zero exit code.
    Create {
        #[command(subcommand)]
        command: CreateCommand,
    },
    #[command()]
    /// Validate a PKGINFO file
    ///
    /// Validate a PKGINFO file according to a schema.
    /// If the file can be validated, the program exits with no output and a return code of 0.
    /// If the file can not be validated, an error is emitted on stderr and the program exits with
    /// a non-zero exit code.
    Validate {
        /// Provide the input file
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },

    /// Parse a PKGINFO file and output it in a different format
    #[command()]
    Format {
        /// Provide the input file
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

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

/// Arguments for creating a PKGINFO file according to the format version 1 schema
///
/// This struct is defined separately for re-using it for both v1 and v2 since they have
/// an overlapping set of fields.
#[derive(Clone, Debug, Args)]
pub struct V1CreateArgs {
    /// Provide a pkgname
    #[arg(env = "PKGINFO_PKGNAME", long, value_name = "PKGNAME")]
    pub pkgname: Name,

    /// Provide a pkgbase
    #[arg(env = "PKGINFO_PKGBASE", long, value_name = "PKGBASE")]
    pub pkgbase: Name,

    /// Provide a pkgver
    #[arg(env = "PKGINFO_PKGVER", long, value_name = "PKGVER")]
    pub pkgver: Version,

    /// Provide a pkgdesc
    #[arg(env = "PKGINFO_PKGDESC", long, value_name = "PKGDESC")]
    pub pkgdesc: PkgDesc,

    /// Provide a url
    #[arg(env = "PKGINFO_URL", long, value_name = "URL")]
    pub url: Url,

    /// Provide a builddate
    #[arg(env = "PKGINFO_BUILDDATE", long, value_name = "BUILDDATE")]
    pub builddate: BuildDate,

    /// Provide a packager
    #[arg(env = "PKGINFO_PACKAGER", long, value_name = "PACKAGER")]
    pub packager: Packager,

    /// Provide a size
    #[arg(env = "PKGINFO_SIZE", long, value_name = "SIZE")]
    pub size: InstalledSize,

    /// Provide a architecture
    #[arg(env = "PKGINFO_ARCH", long, value_name = "ARCH")]
    pub arch: Architecture,

    /// Provide one or more licenses
    #[arg(
        env = "PKGINFO_LICENSE",
        value_delimiter = ' ',
        long,
        value_name = "LICENSE"
    )]
    pub license: Vec<License>,

    /// Provide one or more replaces
    #[arg(
        env = "PKGINFO_REPLACES",
        value_delimiter = ' ',
        long,
        value_name = "REPLACES"
    )]
    pub replaces: Vec<PackageRelation>,

    /// Provide one or more groups
    #[arg(
        env = "PKGINFO_GROUP",
        value_delimiter = ' ',
        long,
        value_name = "GROUP"
    )]
    pub group: Vec<Group>,

    /// Provide one or more conflicts
    #[arg(
        env = "PKGINFO_CONFLICT",
        value_delimiter = ' ',
        long,
        value_name = "CONFLICT"
    )]
    pub conflict: Vec<PackageRelation>,

    /// Provide one or more provides
    #[arg(
        env = "PKGINFO_PROVIDES",
        value_delimiter = ' ',
        long,
        value_name = "PROVIDES"
    )]
    pub provides: Vec<PackageRelation>,

    /// Provide one or more backups
    #[arg(
        env = "PKGINFO_BACKUP",
        value_delimiter = ' ',
        long,
        value_name = "BACKUP"
    )]
    pub backup: Vec<Backup>,

    /// Provide one or more depends
    #[arg(
        env = "PKGINFO_DEPEND",
        value_delimiter = ' ',
        long,
        value_name = "DEPEND"
    )]
    pub depend: Vec<PackageRelation>,

    /// Provide one or more optdepend
    #[arg(
        env = "PKGINFO_OPTDEPEND",
        value_delimiter = ',',
        long,
        value_name = "OPTDEPEND"
    )]
    pub optdepend: Vec<OptDepend>,

    /// Provide one or more makedepend
    #[arg(
        env = "PKGINFO_MAKEDEPEND",
        value_delimiter = ' ',
        long,
        value_name = "MAKEDEPEND"
    )]
    pub makedepend: Vec<PackageRelation>,

    /// Provide one or more checkdepend
    #[arg(
        env = "PKGINFO_CHECKDEPEND",
        value_delimiter = ' ',
        long,
        value_name = "CHECKDEPEND"
    )]
    pub checkdepend: Vec<PackageRelation>,

    /// Provide a file to write to
    #[arg(default_value_t = OutputFile::default(), value_name = "FILE")]
    pub output: OutputFile,
}

/// Create an PKGINFO file according to a schema
///
/// If the input can be validated according to the schema, the program exits with no output and
/// a return code of 0. If the input can not be validated according to the schema, an error
/// is emitted on stderr and the program exits with a non-zero exit code.
#[derive(Clone, Debug, Subcommand)]
pub enum CreateCommand {
    /// Create a PKGINFO version 1 file
    V1 {
        #[command(flatten)]
        args: V1CreateArgs,
    },
    /// Create a PKGINFO version 2 file
    V2 {
        #[command(flatten)]
        args: V1CreateArgs,

        /// Provide one or more Xdata
        #[arg(env = "PKGINFO_XDATA", long, value_name = "XDATA")]
        xdata: Vec<ExtraData>,
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
