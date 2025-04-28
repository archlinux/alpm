//! Definition of the high-level binary entry points.

use std::path::PathBuf;

use alpm_pkgbuild::{bridge::run_bridge_script, cli::OutputFormat, error::Error};

/// Run the bridge script on a `PKGBUILD` and return the output.
///
/// # Errors
///
/// Returns an error if `run_bridge_script` fails.
pub fn run_bridge(pkgbuild_path: PathBuf) -> Result<(), Error> {
    println!("{}", run_bridge_script(&pkgbuild_path)?);

    Ok(())
}
