//! Definition of the high-level binary entry points.

use std::path::PathBuf;

use alpm_common::MetadataFile;
use alpm_pkgbuild::{
    bridge::{parser::BridgeOutput, run_bridge_script},
    cli::OutputFormat,
    error::Error,
};
use alpm_srcinfo::{SourceInfo, SourceInfoV1};

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
    let output = BridgeOutput::from_file(&pkgbuild_path)?;
    let pkgbuild_source_info: SourceInfoV1 = output.try_into()?;

    let source_info = SourceInfo::from_file_with_schema(srcinfo_path, None)?;
    let SourceInfo::V1(source_info) = source_info;

    if source_info != pkgbuild_source_info {
        let pkgbuild_source_info = serde_json::to_string_pretty(&pkgbuild_source_info)?;
        let source_info = serde_json::to_string_pretty(&source_info)?;

        let pkgbuild_json_path = PathBuf::from("pkgbuild.json");
        std::fs::write("pkgbuild.json", pkgbuild_source_info).map_err(|source| Error::IoPath {
            path: pkgbuild_json_path,
            context: "writing pkgbuild.json file",
            source,
        })?;
        let srcinfo_json_path = PathBuf::from("srcinfo.json");
        std::fs::write("srcinfo.json", source_info).map_err(|source| Error::IoPath {
            path: srcinfo_json_path,
            context: "writing srcinfo.json file",
            source,
        })?;

        eprintln!("Generated .SRCINFO content differs to .SRCINFO read from disk.");
        eprintln!("Compare the two generated files srcinfo.json and pkgbuild.json for details");
        std::process::exit(1);
    } else {
        println!("The generated content matches that read from the disk.");
    }

    Ok(())
}
