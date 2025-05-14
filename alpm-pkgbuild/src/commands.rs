use std::path::PathBuf;

use alpm_pkgbuild::{
    bridge::{
        parser::BridgeOutput,
        run_bridge_script,
        source_info::source_info_from_bridge_output,
    },
    error::Error,
};
use winnow::Parser;

/// Run the bridge script on a `PKGBUILD` and return the output.
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

    let srcinfo = source_info_from_bridge_output(output)?;
    println!("{srcinfo:#?}");

    Ok(())
}
