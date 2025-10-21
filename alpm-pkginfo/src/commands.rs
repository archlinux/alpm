//! Command-line functions, that are called by the `alpm-pkginfo` executable.

use std::{
    fs::{File, create_dir_all},
    io::{self, IsTerminal, Write},
    path::PathBuf,
};

use alpm_common::MetadataFile;
use alpm_pkginfo::{
    PackageInfo,
    PackageInfoSchema,
    PackageInfoV1,
    PackageInfoV2,
    cli::{CreateCommand, OutputFormat},
};
use fluent_i18n::t;
use thiserror::Error;

/// A high-level error wrapper around [`alpm_soname::Error`] to add CLI error cases.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// No input file given.
    #[error("{msg}", msg = t!("error-no-input-file"))]
    NoInputFile,

    /// JSON error.
    #[error("{msg}", msg = t!("error-json", { "source" => .0.to_string() }))]
    Json(#[from] serde_json::Error),

    /// An [alpm_pkginfo::Error]
    #[error(transparent)]
    PkgInfo(#[from] alpm_pkginfo::Error),

    /// An [alpm_types::Error]
    #[error(transparent)]
    AlpmTypes(#[from] alpm_types::Error),
}

/// Create a file according to a PKGINFO schema
///
/// ## Errors
///
/// Returns an error if one of the provided arguments is not valid, if creating output directory or
/// file is not possible, or if the output file can not be written to.
pub fn create_file(command: CreateCommand) -> Result<(), Error> {
    let (data, output) = match command {
        CreateCommand::V1 { args } => (
            PackageInfoV1 {
                pkgname: args.pkgname,
                pkgbase: args.pkgbase,
                pkgver: args.pkgver,
                pkgdesc: args.pkgdesc,
                url: args.url,
                builddate: args.builddate,
                packager: args.packager,
                size: args.size,
                arch: args.arch,
                license: args.license,
                replaces: args.replaces,
                group: args.group,
                conflict: args.conflict,
                provides: args.provides,
                backup: args.backup,
                depend: args.depend,
                optdepend: args.optdepend,
                makedepend: args.makedepend,
                checkdepend: args.checkdepend,
            }
            .to_string(),
            args.output,
        ),
        CreateCommand::V2 { args, xdata } => (
            PackageInfoV2 {
                pkgname: args.pkgname,
                pkgbase: args.pkgbase,
                pkgver: args.pkgver,
                pkgdesc: args.pkgdesc,
                url: args.url,
                builddate: args.builddate,
                packager: args.packager,
                size: args.size,
                arch: args.arch,
                license: args.license,
                replaces: args.replaces,
                group: args.group,
                conflict: args.conflict,
                provides: args.provides,
                backup: args.backup,
                depend: args.depend,
                optdepend: args.optdepend,
                makedepend: args.makedepend,
                checkdepend: args.checkdepend,
                xdata: xdata.try_into()?,
            }
            .to_string(),
            args.output,
        ),
    };

    // create any parent directories if necessary
    if let Some(output_dir) = output.0.parent() {
        create_dir_all(output_dir).map_err(|source| alpm_pkginfo::Error::IoPathError {
            path: output_dir.to_path_buf(),
            context: t!("error-io-create-output-dir"),
            source,
        })?;
    }

    let mut out = File::create(&output.0).map_err(|source| alpm_pkginfo::Error::IoPathError {
        path: output.0.clone(),
        context: t!("error-io-create-output-file"),
        source,
    })?;

    let _ = out
        .write(data.as_bytes())
        .map_err(|source| alpm_pkginfo::Error::IoPathError {
            path: output.0,
            context: t!("error-io-write-output-file"),
            source,
        })?;

    Ok(())
}

/// Parses a file according to a PKGINFO schema.
///
/// Returns a serializable PackageInfo if the file is valid, otherwise an error is returned.
///
/// NOTE: If a command is piped to this process, the input is read from stdin.
/// See [`IsTerminal`] for more information about how terminal detection works.
///
/// [`IsTerminal`]: https://doc.rust-lang.org/stable/std/io/trait.IsTerminal.html
///
/// ## Errors
///
/// Returns an error if the file can not be read, if the file can not be parsed, or if the file is
/// not valid according to the schema.
pub fn parse(
    file: Option<PathBuf>,
    schema: Option<PackageInfoSchema>,
) -> Result<PackageInfo, Error> {
    let package_info = if let Some(file) = file {
        PackageInfo::from_file_with_schema(file, schema)?
    } else if !io::stdin().is_terminal() {
        PackageInfo::from_stdin_with_schema(schema)?
    } else {
        Err(Error::NoInputFile)?
    };

    Ok(package_info)
}

/// Validate a file according to a PKGINFO schema.
///
/// This is a thin wrapper around [`parse`] that only checks if the file is valid.
///
/// ## Errors
///
/// Returns an error if parsing `file` fails.
pub fn validate(file: Option<PathBuf>, schema: Option<PackageInfoSchema>) -> Result<(), Error> {
    let _ = parse(file, schema)?;
    Ok(())
}

/// Formats a file according to a PKGINFO schema.
///
/// Validates and prints the parsed file in the specified output format to stdout.
///
/// The output will be pretty-printed if the `pretty` flag is set to `true` and if the format
/// supports it.
///
/// ## Errors
///
/// Returns an error if parsing of `file` fails or if the output format can not be created.
pub fn format(
    file: Option<PathBuf>,
    schema: Option<PackageInfoSchema>,
    output_format: OutputFormat,
    pretty: bool,
) -> Result<(), Error> {
    let pkg_info = parse(file, schema)?;
    match output_format {
        OutputFormat::Json => {
            println!(
                "{}",
                if pretty {
                    serde_json::to_string_pretty(&pkg_info)?
                } else {
                    serde_json::to_string(&pkg_info)?
                }
            );
        }
    }
    Ok(())
}
