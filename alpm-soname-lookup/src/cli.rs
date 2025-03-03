use std::path::PathBuf;

use clap::Parser;

use crate::dir::LookupDirectory;

#[derive(Debug, Parser)]
#[command(about = "Finds ALPM soname provisions", author, version)]
pub struct ProvisionCli {
    /// The lookup directory for shared libraries
    #[arg(short, long, value_name = "DIR")]
    pub lookup_dir: LookupDirectory,

    /// The package to inspect
    #[arg(value_name = "PACKAGE")]
    pub package: PathBuf,
}
