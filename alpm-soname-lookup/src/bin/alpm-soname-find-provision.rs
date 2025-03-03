use alpm_soname_lookup::{Error, cli::ProvisionCli};
use clap::Parser;

/// The entry point for the `alpm-soname-find-provision` binary.
fn main() -> Result<(), Error> {
    let args = ProvisionCli::parse();
    alpm_soname_lookup::find_provision(args.package, args.lookup_dir)?;
    Ok(())
}
