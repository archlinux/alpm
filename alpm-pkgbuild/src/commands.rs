use std::path::PathBuf;

use alpm_common::MetadataFile;
use alpm_pkgbuild::{
    bridge::{
        parser::BridgeOutput,
        run_bridge_script,
        source_info::source_info_from_bridge_output,
    },
    error::Error,
};
use alpm_srcinfo::SourceInfo;
use winnow::Parser;

use crate::cli::OutputFormat;

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
    let bridge_output = run_bridge_script(&pkgbuild_path)?;

    let output = BridgeOutput::parser
        .parse(&bridge_output)
        .map_err(|err| Error::BridgeParseError(format!("{err}")))?;

    let source_info = source_info_from_bridge_output(output)?;

    match output_format {
        OutputFormat::Json => {
            let json = if pretty {
                serde_json::to_string_pretty(&source_info)?
            } else {
                serde_json::to_string(&source_info)?
            };
            println!("{json}");
        }
    }

    Ok(())
}

/// Run the bridge script on a `PKGBUILD` and return the output.
///
/// This is a development and debug command to better understand the inner workings of this binary.
///
/// If the generated and read SRCINFO representations are not equal, place two files into the
/// current directory: `srcinfo.json` and `pkgbuild.json`.
///
/// These files contain pretty-printed json, which accurately depict the internal representation
/// used to compare the two files.
pub fn compare_source_info(pkgbuild_path: PathBuf, srcinfo_path: PathBuf) -> Result<(), Error> {
    let bridge_output = run_bridge_script(&pkgbuild_path)?;
    let output = BridgeOutput::parser
        .parse(&bridge_output)
        .map_err(|err| Error::BridgeParseError(format!("{err}")))?;
    let pkgbuild_source_info = source_info_from_bridge_output(output)?;

    let source_info = SourceInfo::from_file_with_schema(srcinfo_path, None)?;
    let SourceInfo::V1(source_info) = source_info;

    if source_info != pkgbuild_source_info {
        let pkgbuild_source_info = serde_json::to_string_pretty(&pkgbuild_source_info)?;
        let source_info = serde_json::to_string_pretty(&source_info)?;

        let pkgbuild_json_path = PathBuf::from("pkgbuild.json");
        std::fs::write("pkgbuild.json", pkgbuild_source_info).map_err(|source| {
            Error::IoPath(pkgbuild_json_path, "writing pkgbuild.json file", source)
        })?;
        let srcinfo_json_path = PathBuf::from("srcinfo.json");
        std::fs::write("srcinfo.json", source_info).map_err(|source| {
            Error::IoPath(srcinfo_json_path, "writing srcinfo.json file", source)
        })?;
    }

    Ok(())
}
