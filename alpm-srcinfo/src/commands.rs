//! Functions called from the binary.
use std::{
    io::{self, IsTerminal},
    path::PathBuf,
};

use alpm_common::MetadataFile;
use alpm_types::Architecture;

use crate::{
    SourceInfo,
    SourceInfoSchema,
    cli::{PackagesOutputFormat, SourceInfoOutputFormat},
    error::Error,
    source_info::v1::merged::MergedPackage,
};

/// Validates a SRCINFO file from a path or stdin.
///
/// Wraps the [`parse`] function and allows to ensure that no errors occurred during parsing.
pub fn validate(file: Option<&PathBuf>, schema: Option<SourceInfoSchema>) -> Result<(), Error> {
    let _result = parse(file, schema)?;

    Ok(())
}

/// Parses a SRCINFO file from a path or stdin and outputs it in the specified format on stdout.
///
/// # Errors
///
/// Returns an error if the input can not be parsed and validated, or if the output can not be
/// formatted in the selected output format.
pub fn format_source_info(
    file: Option<&PathBuf>,
    schema: Option<SourceInfoSchema>,
    output_format: SourceInfoOutputFormat,
    pretty: bool,
) -> Result<(), Error> {
    let srcinfo = parse(file, schema)?;
    let SourceInfo::V1(source_info) = srcinfo;

    match output_format {
        SourceInfoOutputFormat::Json => {
            let json = if pretty {
                serde_json::to_string_pretty(&source_info)?
            } else {
                serde_json::to_string(&source_info)?
            };
            println!("{json}");
        }
        SourceInfoOutputFormat::Srcinfo => {
            println!("{}", source_info.as_srcinfo())
        }
    }

    Ok(())
}

/// Parses a SRCINFO file from a path or stdin and outputs all info grouped by packages for a given
/// architecture in the specified format on stdout.
///
/// # Errors
///
/// Returns an error if the input can not be parsed and validated, or if the output can not be
/// formatted in the selected output format.
pub fn format_packages(
    file: Option<&PathBuf>,
    schema: Option<SourceInfoSchema>,
    output_format: PackagesOutputFormat,
    architecture: Architecture,
    pretty: bool,
) -> Result<(), Error> {
    let srcinfo = parse(file, schema)?;
    let SourceInfo::V1(source_info) = srcinfo;

    let packages: Vec<MergedPackage> = source_info
        .packages_for_architecture(architecture)
        .collect();

    match output_format {
        PackagesOutputFormat::Json => {
            let json = if pretty {
                serde_json::to_string_pretty(&packages)?
            } else {
                serde_json::to_string(&packages)?
            };
            println!("{json}");
        }
    }

    Ok(())
}

/// Parses and interprets a SRCINFO file from a path or stdin.
///
/// ## Note
///
/// If a command is piped to this process, the input is read from stdin.
/// See [`IsTerminal`] for more information about how terminal detection works.
///
/// [`IsTerminal`]: https://doc.rust-lang.org/stable/std/io/trait.IsTerminal.html
///
/// # Errors
///
/// Returns an error if the input can not be parsed and validated, or if the output can not be
/// formatted in the selected output format.
///
/// Furthermore, returns an error array with potentially un/-recoverable (linting-)errors, which
/// needs to be explicitly handled by the caller.
pub fn parse(
    file: Option<&PathBuf>,
    schema: Option<SourceInfoSchema>,
) -> Result<SourceInfo, Error> {
    if let Some(file) = file {
        SourceInfo::from_file_with_schema(file, schema)
    } else if !io::stdin().is_terminal() {
        SourceInfo::from_stdin_with_schema(schema)
    } else {
        Err(Error::NoInputFile)
    }
}
