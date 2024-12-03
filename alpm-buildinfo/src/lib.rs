#![doc = include_str!("../README.md")]

mod buildinfo_v1;
pub use crate::buildinfo_v1::BuildInfoV1;

mod buildinfo_v2;
pub use crate::buildinfo_v2::BuildInfoV2;

pub mod cli;

mod error;
pub mod schema;
use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::File;
use std::io;
use std::io::IsTerminal;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use alpm_types::SchemaVersion;
use alpm_types::Sha256Checksum;
use cli::CreateCommand;
use cli::OutputFormat;
use cli::ValidateArgs;
use erased_serde::Serialize;
use schema::Schema;

pub use crate::error::Error;

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
pub fn parse(args: ValidateArgs) -> Result<Box<dyn Serialize>, Error> {
    let contents = if let Some(file) = &args.file {
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

    // Determine the schema that should be used to validate the file.
    // If no explicit schema version is provided, the version will be deduced from the contents of
    // the file itself. If the file does not contain a version, an error will be returned.
    let schema = if let Some(schema) = args.schema {
        schema.clone()
    } else {
        Schema::from_contents(&contents)?
    };

    match schema {
        Schema::V1(_) => Ok(Box::new(BuildInfoV1::from_str(&contents)?)),
        Schema::V2(_) => Ok(Box::new(BuildInfoV2::from_str(&contents)?)),
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
pub fn format(args: ValidateArgs, output_format: OutputFormat) -> Result<(), Error> {
    let build_info = parse(args)?;
    match output_format {
        OutputFormat::Json => {
            let json = serde_json::to_string(&build_info)?;
            println!("{json}");
        }
    }
    Ok(())
}
