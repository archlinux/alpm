//! Functions called from the binary.
use std::{
    io::{self, IsTerminal},
    path::{Path, PathBuf},
};

use alpm_common::MetadataFile;
use alpm_srcinfo::{
    SourceInfo,
    SourceInfoSchema,
    SourceInfoV1,
    cli::{PackagesOutputFormat, SourceInfoOutputFormat},
    source_info::v1::merged::MergedPackage,
};
use alpm_types::Architecture;
use fluent_i18n::t;
use thiserror::Error;

/// A high-level error wrapper around [`alpm_srcinfo::Error`] to add CLI error cases.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// JSON error while creating JSON formatted output.
    #[error("{msg}", msg = t!("error-json", { "error" => .0.to_string() }))]
    Json(#[from] serde_json::Error),

    /// No input file given.
    #[error("{msg}", msg = t!("error-no-input-file"))]
    NoInputFile,

    /// An [alpm_srcinfo::Error]
    #[error(transparent)]
    Srcinfo(#[from] alpm_srcinfo::Error),
}

/// Take a [PKGBUILD], create [SRCINFO] data from it and print it.
///
/// # Errors
///
/// Returns an error if
///
/// - running the [`alpm-pkgbuild-bridge`] script fails,
/// - or parsing the output of the ``alpm-pkgbuild-bridge`] script fails.
///
/// [PKGBUILD]: https://man.archlinux.org/man/PKGBUILD.5
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
/// [`alpm-pkgbuild-bridge`]: https://gitlab.archlinux.org/archlinux/alpm/alpm-pkgbuild-bridge
pub fn create(
    pkgbuild_path: &Path,
    output_format: SourceInfoOutputFormat,
    pretty: bool,
) -> Result<(), Error> {
    let source_info = SourceInfoV1::from_pkgbuild(pkgbuild_path)?;

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
            print!("{}", source_info.as_srcinfo())
        }
    }

    Ok(())
}

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
    let source_info = if let Some(file) = file {
        SourceInfo::from_file_with_schema(file, schema)?
    } else if !io::stdin().is_terminal() {
        SourceInfo::from_stdin_with_schema(schema)?
    } else {
        Err(Error::NoInputFile)?
    };

    Ok(source_info)
}
