//! Definition of the high-level binary entry points.

use std::path::PathBuf;

use alpm_pkgbuild::{
    bridge::{parser::BridgeOutput, run_bridge_script}, cli::OutputFormat, error::Error};
use winnow::Parser;

/// Run the bridge script on a `PKGBUILD` and return the output.
///
/// # Errors
///
/// Returns an error if `run_bridge_script` fails.
pub fn run_bridge(pkgbuild_path: PathBuf) -> Result<(), Error> {
    println!("{}", run_bridge_script(&pkgbuild_path)?);

    Ok(())
}

/// Take a `PKGBUILD` and create a `SRCINFO` file from it.
pub fn print_source_info(pkgbuild_path: PathBuf) -> Result<(), Error> {
    let bridge_output = run_bridge_script(&pkgbuild_path)?;

    let output = BridgeOutput::parser
        .parse(&bridge_output)
        .map_err(|err| Error::BridgeParseError(format!("{err}")))?;

    println!("{output:#?}");

    Ok(())
}
