//! Definition of the high-level binary entry points.

use std::path::PathBuf;

use alpm_pkgbuild::{bridge::run_bridge_script, error::Error, cli::OutputFormat, };

/// Run the bridge script on a `PKGBUILD` and return the output.
pub fn run_bridge(pkgbuild_path: PathBuf) -> Result<(), Error> {
    println!("{}", run_bridge_script(&pkgbuild_path)?);

    Ok(())
}
