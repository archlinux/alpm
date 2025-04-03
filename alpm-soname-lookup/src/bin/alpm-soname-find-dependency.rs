use alpm_soname_lookup::{Error, cli::DependencyCli};
use clap::Parser;

/// The entry point for the `alpm-soname-find-dependency` binary.
fn main() -> Result<(), Error> {
    let args = DependencyCli::parse();
    let sonames = alpm_soname_lookup::find_dependency(args.package, args.lookup_dir)?;
    for soname in sonames {
        println!("{soname}");
    }
    Ok(())
}
