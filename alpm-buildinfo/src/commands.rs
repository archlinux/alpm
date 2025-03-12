use std::{
    fs::{File, create_dir_all},
    io::{self, IsTerminal, Write},
    str::FromStr,
};

use alpm_common::MetadataFile;
use alpm_types::{SchemaVersion, Sha256Checksum};

use crate::{
    BuildInfo,
    BuildInfoV1,
    BuildInfoV2,
    cli::{CreateCommand, OutputFormat, ValidateArgs},
    error::Error,
};

/// Create a file according to a BUILDINFO schema
pub fn create_file(command: CreateCommand) -> Result<(), Error> {
    let (data, output) = match command {
        CreateCommand::V1 { args } => (
            BuildInfoV1::new(
                args.builddate,
                args.builddir,
                args.buildenv,
                SchemaVersion::from_str("1")?,
                args.installed,
                args.options,
                args.packager,
                args.pkgarch,
                args.pkgbase,
                Sha256Checksum::from_str(&args.pkgbuild_sha256sum)?,
                args.pkgname,
                args.pkgver,
            )?
            .to_string(),
            args.output,
        ),
        CreateCommand::V2 {
            args,
            startdir,
            buildtool,
            buildtoolver,
        } => (
            BuildInfoV2::new(
                args.builddate,
                args.builddir,
                startdir,
                buildtool,
                buildtoolver,
                args.buildenv,
                SchemaVersion::from_str("2")?,
                args.installed,
                args.options,
                args.packager,
                args.pkgarch,
                args.pkgbase,
                Sha256Checksum::from_str(&args.pkgbuild_sha256sum)?,
                args.pkgname,
                args.pkgver,
            )?
            .to_string(),
            args.output,
        ),
    };

    // create any parent directories if necessary
    if let Some(output_dir) = output.0.parent() {
        create_dir_all(output_dir).map_err(|e| {
            Error::IoPathError(output_dir.to_path_buf(), "creating output directory", e)
        })?;
    }

    let mut out = File::create(&output.0)
        .map_err(|e| Error::IoPathError(output.0.clone(), "creating output file", e))?;

    let _ = out
        .write(data.as_bytes())
        .map_err(|e| Error::IoPathError(output.0, "writing to output file", e))?;

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
    if let Some(file) = &args.file {
        BuildInfo::from_file_with_schema(file, args.schema)
    } else if !io::stdin().is_terminal() {
        BuildInfo::from_stdin_with_schema(args.schema)
    } else {
        return Err(Error::NoInputFile);
    }
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
