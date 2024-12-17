use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;

#[derive(Clone, Debug, Parser)]
#[command(about, author, name = "alpm-srcinfo", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Validate a SRCINFO file
    ///
    /// Validate a SRCINFO file according to a schema.
    /// If the file can be validated, the program exits with no output and a return code of 0.
    /// If the file can not be validated, an error is emitted on stderr and the program exits with
    /// a non-zero exit code.
    #[command()]
    Validate {
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },
}
