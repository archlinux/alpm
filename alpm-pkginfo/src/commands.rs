use std::{
    fs::{File, create_dir_all},
    io::{self, IsTerminal, Write},
    path::PathBuf,
};

use alpm_common::MetadataFile;

use crate::{
    Error,
    PackageInfo,
    PackageInfoSchema,
    PackageInfoV1,
    PackageInfoV2,
    cli::{CreateCommand, OutputFormat},
};

/// Create a file according to a PKGINFO schema
///
/// ## Errors
///
/// Returns an error if one of the provided arguments is not valid, if creating output directory or
/// file is not possible, or if the output file can not be written to.
pub fn create_file(command: CreateCommand) -> Result<(), Error> {
    let (data, output) = match command {
        CreateCommand::V1 { args } => (
            PackageInfoV1::new(
                args.pkgname,
                args.pkgbase,
                args.pkgver,
                args.pkgdesc,
                args.url,
                args.builddate,
                args.packager,
                args.size,
                args.arch,
                args.license,
                args.replaces,
                args.group,
                args.conflict,
                args.provides,
                args.backup,
                args.depend,
                args.optdepend,
                args.makedepend,
                args.checkdepend,
            )
            .to_string(),
            args.output,
        ),
        CreateCommand::V2 { args, xdata } => (
            PackageInfoV2::new(
                args.pkgname,
                args.pkgbase,
                args.pkgver,
                args.pkgdesc,
                args.url,
                args.builddate,
                args.packager,
                args.size,
                args.arch,
                args.license,
                args.replaces,
                args.group,
                args.conflict,
                args.provides,
                args.backup,
                args.depend,
                args.optdepend,
                args.makedepend,
                args.checkdepend,
                xdata,
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
    if let Some(file) = file {
        PackageInfo::from_file_with_schema(file, schema)
    } else if !io::stdin().is_terminal() {
        PackageInfo::from_stdin_with_schema(schema)
    } else {
        return Err(Error::NoInputFile);
    }
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
