//! Definition of the high-level binary entry points.

use std::path::PathBuf;

use alpm_pkgbuild::{
    bridge::{parser::BridgeOutput, run_bridge_script},
    error::Error, cli::OutputFormat, };
use alpm_srcinfo::SourceInfoV1;

/// Run the bridge script on a `PKGBUILD` and return the output.
pub fn run_bridge(pkgbuild_path: PathBuf) -> Result<(), Error> {
    println!("{}", run_bridge_script(&pkgbuild_path)?);

    Ok(())
}

/// Take a `PKGBUILD` and create a `SRCINFO` file from it.
pub fn print_source_info(
    pkgbuild_path: PathBuf,

    output_format: OutputFormat,
    pretty: bool,
) -> Result<(), Error> {
    let output = BridgeOutput::from_file(&pkgbuild_path)?;
    let source_info: SourceInfoV1 = output.try_into()?;

    match output_format {
        OutputFormat::Json => {
            let json = if pretty {
                serde_json::to_string_pretty(&source_info)?
            } else {
                serde_json::to_string(&source_info)?
            };
            println!("{json}");
        }
        OutputFormat::Srcinfo => {
            println!("{}", source_info.as_srcinfo())
        }
    }

    Ok(())
}
