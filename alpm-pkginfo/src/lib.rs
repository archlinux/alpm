#![doc = include_str!("../README.md")]

mod pkginfo_v1;
pub use crate::pkginfo_v1::PkgInfoV1;

mod pkginfo_v2;
pub use crate::pkginfo_v2::PkgInfoV2;

pub mod cli;

mod error;
use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::File;
use std::io;
use std::io::IsTerminal;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use cli::CreateCommand;
use cli::OutputFormat;
use erased_serde::Serialize;

pub use crate::error::Error;

/// Create a file according to a PKGINFO schema
///
/// ## Errors
///
/// Returns an error if one of the provided arguments is not valid, if creating output directory or
/// file is not possible, or if the output file can not be written to.
pub fn create_file(command: CreateCommand) -> Result<(), Error> {
    let (data, output) = match command {
        CreateCommand::V1 { args } => (
            PkgInfoV1::new(
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
            PkgInfoV2::new(
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
/// Returns a serializable PkgInfo if the file is valid, otherwise an error is returned.
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
pub fn parse(file: Option<PathBuf>) -> Result<Box<dyn Serialize>, Error> {
    let contents = if let Some(file) = &file {
        read_to_string(file)
            .map_err(|e| Error::IoPathError(file.clone(), "reading file contents", e))?
    } else if !io::stdin().is_terminal() {
        let mut buffer = Vec::new();
        let mut stdin = io::stdin();
        stdin.read_to_end(&mut buffer).map_err(|e| {
            Error::IoPathError(PathBuf::from("/dev/stdin"), "reading from stdin", e)
        })?;

        String::from_utf8(buffer)?.to_string()
    } else {
        return Err(Error::NoInputFile);
    };

    match PkgInfoV2::from_str(&contents) {
        Ok(pkg_info) => Ok(Box::new(pkg_info)),
        Err(_) => Ok(Box::new(PkgInfoV1::from_str(&contents)?)),
    }
}

/// Validate a file according to a PKGINFO schema.
///
/// This is a thin wrapper around [`parse`] that only checks if the file is valid.
///
/// ## Errors
///
/// Returns an error if parsing `file` fails.
pub fn validate(file: Option<PathBuf>) -> Result<(), Error> {
    let _ = parse(file)?;
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
    output_format: OutputFormat,
    pretty: bool,
) -> Result<(), Error> {
    let pkg_info = parse(file)?;
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
