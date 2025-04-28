use std::path::PathBuf;

use alpm_pkgbuild::{bridge::run_bridge_script, error::Error};

/// Create a file according to a BUILDINFO schema
pub fn run_bridge(pkgbuild_path: PathBuf) -> Result<(), Error> {
    println!("{}", run_bridge_script(&pkgbuild_path)?);

    Ok(())
}
