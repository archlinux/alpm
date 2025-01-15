use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;

#[derive(Clone, Debug, Parser)]
#[command(about, author, name = "alpm-srcinfo", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Validate a SRCINFO file from a path or `stdin`.
    ///
    /// If the file can be validated, the program exits with no output and a return code of 0.
    /// If the file can not be validated, an error is emitted on stderr and the program exits with
    /// a non-zero exit status.
    #[command()]
    Validate {
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },

    /// Read a SRCINFO file from a path or `stdin` and perform linter checks on it.
    ///
    /// This ensures that the SRCINFO file is both **valid** and adheres to currently known best
    /// practices.
    ///
    /// Returns with a non-zero exit status as soon as any linting issue is encountered.
    #[command()]
    Check {
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },
}
