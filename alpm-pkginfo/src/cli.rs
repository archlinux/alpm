//! Command-line argument handling for `alpm-pkginfo`.

use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
    str::FromStr,
};

use alpm_types::{
    Architecture,
    Backup,
    BuildDate,
    ExtraData,
    FullVersion,
    Group,
    InstalledSize,
    License,
    Name,
    OptionalDependency,
    PackageDescription,
    PackageRelation,
    Packager,
    Url,
};
use clap::{Args, Parser, Subcommand, ValueEnum};
use strum::Display;

use crate::{Error, PackageInfoSchema, RelationOrSoname};

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

/// The command-line interface handling for `alpm-pkginfo`.
#[derive(Clone, Debug, Parser)]
#[command(about, author, name = "alpm-pkginfo", version)]
pub struct Cli {
    /// The `alpm-pkginfo` commands.
    #[command(subcommand)]
    pub command: Command,
}

/// The `alpm-pkginfo` commands.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    #[command()]
    /// Create a PKGINFO file according to a schema
    ///
    /// If the input can be validated according to the schema, the program writes a valid PKGINFO
    /// file and exits with no output and a return code of 0. If the input can not be validated
    /// according to the schema, an error is emitted on stderr and the program exits with a
    /// non-zero exit code.
    Create {
        /// The `create` command.
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
        /// An optional input file path to read from
        ///
        /// If no file is specified, stdin is read from and expected to contain PKGINFO data to
        /// validate.
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// Provide the PKGINFO schema version to use.
        ///
        /// If no schema version is provided, it will be deduced from the file itself.
        #[arg(short, long, value_name = "VERSION")]
        schema: Option<PackageInfoSchema>,
    },

    /// Parse a PKGINFO file and output it in a different file format
    ///
    /// If the input can be validated according to a known schema, the program writes the PKGINFO
    /// data to stdout in a different file format (optionally, a file path to write to may be
    /// provided) and exits with a return code of 0. Currently only JSON is supported as output
    /// format. If the input can not be validated according to a known schema, an error is
    /// emitted on stderr and the program exits with a non-zero exit code.
    #[command()]
    Format {
        /// An optional input file path to read from
        ///
        /// If no file path is specified, stdin is read from and expected to contain PKGINFO data
        /// to format.
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// Provide the PKGINFO schema version to use.
        ///
        /// If no schema version is provided, it will be deduced from the file itself.
        #[arg(short, long, value_name = "VERSION")]
        schema: Option<PackageInfoSchema>,

        /// The output format to use
        ///
        /// Currently only "json" (the default) is supported
        #[arg(
            short,
            long,
            value_name = "OUTPUT_FORMAT",
            default_value_t = OutputFormat::Json
        )]
        output_format: OutputFormat,

        /// Pretty-print the output
        ///
        /// Has no effect if the output format can not be pretty printed.
        #[arg(short, long)]
        pretty: bool,
    },
}

/// Arguments for creating a PKGINFO file according to the format version 1 schema
///
/// This struct is defined separately for reusing it for v1 and v2 because both share
/// a set of overlapping fields.
#[derive(Args, Clone, Debug)]
pub struct V1CreateArgs {
    /// The pkgname to use in the PKGINFO
    ///
    /// The pkgname must follow the alpm-package-name format (see `man 7 alpm-package-name`).
    #[arg(env = "PKGINFO_PKGNAME", long, value_name = "PKGNAME")]
    pub pkgname: Name,

    /// The pkgbase to use in the PKGINFO
    ///
    /// The pkgbase must follow the alpm-package-name format (see `man 7 alpm-package-name`).
    #[arg(env = "PKGINFO_PKGBASE", long, value_name = "PKGBASE")]
    pub pkgbase: Name,

    /// The pkgver to use in the PKGINFO
    ///
    /// The pkgver value must follow the alpm-pkgver format (see `man 7 alpm-pkgver`).
    #[arg(env = "PKGINFO_PKGVER", long, value_name = "PKGVER")]
    pub pkgver: FullVersion,

    /// The package description to use in the PKGINFO
    ///
    /// The value must follow the format described in the PKGINFO format (see `man 5 PKGINFO`).
    #[arg(env = "PKGINFO_PKGDESC", long, value_name = "PKGDESC")]
    pub pkgdesc: PackageDescription,

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
    pub provides: Vec<RelationOrSoname>,

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
    pub depend: Vec<RelationOrSoname>,

    /// Provide one or more optdepend
    #[arg(
        env = "PKGINFO_OPTDEPEND",
        value_delimiter = ',',
        long,
        value_name = "OPTDEPEND"
    )]
    pub optdepend: Vec<OptionalDependency>,

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

    /// An optional custom file to write to
    #[arg(default_value_t = OutputFile::default(), env = "PKGINFO_OUTPUT_FILE", value_name = "FILE")]
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
        /// Arguments for the `create v1` command.
        #[command(flatten)]
        args: V1CreateArgs,
    },
    /// Create a PKGINFO version 2 file
    V2 {
        /// Arguments for the `create v2` command.
        #[command(flatten)]
        args: V1CreateArgs,

        /// Provide one or more Xdata
        #[arg(env = "PKGINFO_XDATA", long, value_name = "XDATA")]
        xdata: Vec<ExtraData>,
    },
}

/// Output format for the format command
#[derive(Clone, Debug, Default, Display, ValueEnum)]
pub enum OutputFormat {
    /// The JSON output format.
    #[default]
    #[strum(to_string = "json")]
    Json,
}
