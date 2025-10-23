//! Commandline functions, that're called by the `alpm-buildinfo` executable.

use std::{
    fs::{File, create_dir_all},
    io::{self, IsTerminal, Write},
    str::FromStr,
};

use alpm_buildinfo::{
    BuildInfo,
    BuildInfoV1,
    BuildInfoV2,
    cli::{CreateCommand, OutputFormat, ValidateArgs},
};
use alpm_common::MetadataFile;
use alpm_types::Sha256Checksum;
use fluent_i18n::t;
use thiserror::Error;

/// A high-level error wrapper around [`alpm_buildinfo::Error`] to add CLI error cases.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// ALPM type error.
    #[error("{msg}", msg = t!("error-alpm-type", { "source" => .0.to_string() }))]
    AlpmType(#[from] alpm_types::Error),

    /// No input file given.
    #[error("{msg}", msg = t!("error-no-input-file"))]
    NoInputFile,

    /// JSON error.
    #[error("{msg}", msg = t!("error-json", { "source" => .0.to_string() }))]
    Json(#[from] serde_json::Error),

    /// An [alpm_buildinfo::Error]
    #[error(transparent)]
    BuildInfo(#[from] alpm_buildinfo::Error),
}

/// Create a file according to a BUILDINFO schema
pub fn create_file(command: CreateCommand) -> Result<(), Error> {
    let (data, output) = match command {
        CreateCommand::V1 { args } => (
            BuildInfoV1 {
                builddate: args.builddate,
                builddir: args.builddir,
                buildenv: args.buildenv,
                installed: args.installed,
                options: args.options,
                packager: args.packager,
                pkgarch: args.pkgarch,
                pkgbase: args.pkgbase,
                pkgbuild_sha256sum: Sha256Checksum::from_str(&args.pkgbuild_sha256sum)?,
                pkgname: args.pkgname,
                pkgver: args.pkgver,
            }
            .to_string(),
            args.output,
        ),
        CreateCommand::V2 {
            args,
            startdir,
            buildtool,
            buildtoolver,
        } => (
            BuildInfoV2 {
                builddate: args.builddate,
                builddir: args.builddir,
                startdir,
                buildtool,
                buildtoolver,
                buildenv: args.buildenv,
                installed: args.installed,
                options: args.options,
                packager: args.packager,
                pkgarch: args.pkgarch,
                pkgbase: args.pkgbase,
                pkgbuild_sha256sum: Sha256Checksum::from_str(&args.pkgbuild_sha256sum)?,
                pkgname: args.pkgname,
                pkgver: args.pkgver,
            }
            .to_string(),
            args.output,
        ),
    };

    // create any parent directories if necessary
    if let Some(output_dir) = output.0.parent() {
        create_dir_all(output_dir).map_err(|source| alpm_buildinfo::Error::IoPathError {
            path: output_dir.to_path_buf(),
            context: t!("error-io-create-output-dir"),
            source,
        })?;
    }

    let mut out = File::create(&output.0).map_err(|source| alpm_buildinfo::Error::IoPathError {
        path: output.0.clone(),
        context: t!("error-io-create-output-file"),
        source,
    })?;

    let _ = out
        .write(data.as_bytes())
        .map_err(|source| alpm_buildinfo::Error::IoPathError {
            path: output.0,
            context: t!("error-io-write-output-file"),
            source,
        })?;

    Ok(())
}

/// Parses a file according to a BUILDINFO schema.
///
/// Returns a serializable BuildInfo if the file is valid, otherwise an error is returned.
///
/// NOTE: If a command is piped to this process, the input is read from stdin.
/// See [`IsTerminal`] for more information about how terminal detection works.
///
/// [`IsTerminal`]: https://doc.rust-lang.org/stable/std/io/trait.IsTerminal.html
pub fn parse(args: ValidateArgs) -> Result<BuildInfo, Error> {
    let build_info = if let Some(file) = &args.file {
        BuildInfo::from_file_with_schema(file, args.schema)?
    } else if !io::stdin().is_terminal() {
        BuildInfo::from_stdin_with_schema(args.schema)?
    } else {
        Err(Error::NoInputFile)?
    };

    Ok(build_info)
}

/// Validate a file according to a BUILDINFO schema.
///
/// This is a thin wrapper around [`parse`] that only checks if the file is valid.
pub fn validate(args: ValidateArgs) -> Result<(), Error> {
    let _ = parse(args)?;
    Ok(())
}

/// Formats a file according to a BUILDINFO schema.
///
/// Validates and prints the parsed file in the specified output format to stdout.
///
/// The output will be pretty-printed if the `pretty` flag is set to `true` and if the format
/// supports it.
pub fn format(args: ValidateArgs, output_format: OutputFormat, pretty: bool) -> Result<(), Error> {
    let build_info = parse(args)?;
    match output_format {
        OutputFormat::Json => {
            let json = if pretty {
                serde_json::to_string_pretty(&build_info)?
            } else {
                serde_json::to_string(&build_info)?
            };
            println!("{json}");
        }
    }
    Ok(())
}
