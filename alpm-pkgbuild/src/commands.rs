//! Definition of the high-level binary entry points.

use std::path::PathBuf;

use alpm_pkgbuild::{Error, cli::OutputFormat, run_bridge_script, source_info_v1_from_pkgbuild};

/// Run the bridge script on a `PKGBUILD` and return the output.
///
/// # Errors
///
/// Returns an error if `run_bridge_script` fails.
pub fn run_bridge(pkgbuild_path: PathBuf) -> Result<(), Error> {
    println!("{}", run_bridge_script(&pkgbuild_path)?);

    Ok(())
}

/// Takes a [PKGBUILD], creates [SRCINFO] data from it and prints it.
///
/// # Errors
///
/// Returns an error if
///
/// - running the `alpm-pkgbuild-bridge` script fails,
/// - or parsing the output of the `alpm-pkgbuild-bridge` script fails.
///
/// [PKGBUILD]: https://man.archlinux.org/man/PKGBUILD.5
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
pub fn print_source_info(
    pkgbuild_path: PathBuf,
    output_format: OutputFormat,
    pretty: bool,
) -> Result<(), Error> {
    let source_info = source_info_v1_from_pkgbuild(&pkgbuild_path)?;

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
