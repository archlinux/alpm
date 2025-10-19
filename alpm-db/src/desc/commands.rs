//! Commands for creating, validating, parsing, and formatting [alpm-db-desc] files.
//!
//! [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html

use std::{
    fs::{File, create_dir_all},
    io::{IsTerminal, Write, stdin},
};

use alpm_common::MetadataFile;

use crate::{
    Error,
    desc::{
        DbDescFile,
        DbDescFileV1,
        DbDescFileV2,
        cli::{CreateCommand, OutputFormat, ValidateArgs},
    },
};

/// Creates an [alpm-db-desc] file according to the specified version command.
///
/// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
///
/// # Notes
///
/// If no output file is specified, the data is printed to stdout.
///
/// # Errors
///
/// Returns an error if:
///
/// - the output file cannot be created,
/// - or the data cannot be written to the output file.
pub fn create_file(command: CreateCommand) -> Result<(), Error> {
    let (data, output) = match command {
        CreateCommand::V1 { args } => {
            let v1 = DbDescFileV1 {
                name: args.name,
                version: args.version,
                base: args.base,
                description: args.description,
                url: args.url,
                arch: args.arch,
                builddate: args.builddate,
                installdate: args.installdate,
                packager: args.packager,
                size: args.size,
                groups: args.groups,
                reason: args.reason,
                license: args.license,
                validation: args.validation,
                replaces: args.replaces,
                depends: args.depends,
                optdepends: args.optdepends,
                conflicts: args.conflicts,
                provides: args.provides,
            };
            (v1.to_string(), args.output)
        }
        CreateCommand::V2 { args, xdata } => {
            let v2 = DbDescFileV2 {
                name: args.name,
                version: args.version,
                base: args.base,
                description: args.description,
                url: args.url,
                arch: args.arch,
                builddate: args.builddate,
                installdate: args.installdate,
                packager: args.packager,
                size: args.size,
                groups: args.groups,
                reason: args.reason,
                license: args.license,
                validation: args.validation,
                replaces: args.replaces,
                depends: args.depends,
                optdepends: args.optdepends,
                conflicts: args.conflicts,
                provides: args.provides,
                xdata,
            };
            (v2.to_string(), args.output)
        }
    };

    if let Some(output_path) = output {
        // Create parent directories if necessary
        if let Some(output_dir) = output_path.parent() {
            create_dir_all(output_dir).map_err(|source| Error::IoPathError {
                path: output_dir.to_path_buf(),
                context: "creating output directory",
                source,
            })?;
        }

        let mut out = File::create(&output_path).map_err(|source| Error::IoPathError {
            path: output_path.clone(),
            context: "creating output file",
            source,
        })?;

        out.write_all(data.as_bytes())
            .map_err(|source| Error::IoPathError {
                path: output_path.clone(),
                context: "writing to output file",
                source,
            })?;
    } else {
        print!("{data}");
    }

    Ok(())
}

/// Parses an [alpm-db-desc] file and returns it as a [`DbDescFile`].
///
/// Data can be read from file or stdin.
///
/// # Errors
///
/// Returns an error if
///
/// - valid data cannot be read from a file,
/// - valid data cannot be read from stdin,
/// - or stdin is not a valid file descriptor/TTY.
///
/// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
pub fn parse(args: ValidateArgs) -> Result<DbDescFile, Error> {
    if let Some(file) = &args.file {
        DbDescFile::from_file_with_schema(file, args.schema)
    } else if !stdin().is_terminal() {
        DbDescFile::from_stdin_with_schema(args.schema)
    } else {
        Err(Error::NoInputFile)
    }
}

/// Validate a DB desc file by parsing it.
pub fn validate(args: ValidateArgs) -> Result<(), Error> {
    let _ = parse(args)?;
    Ok(())
}

/// Formats and optionally pretty prints an [alpm-db-desc] file.
///
/// # Errors
///
/// Returns an error if
///
/// - the [alpm-db-desc] file cannot be parsed,
/// - or the data cannot be formatted in the chosen output format.
///
/// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
pub fn format(args: ValidateArgs, output_format: OutputFormat, pretty: bool) -> Result<(), Error> {
    let desc = parse(args)?;
    match output_format {
        OutputFormat::Json => {
            let json = if pretty {
                serde_json::to_string_pretty(&desc).map_err(|e| Error::Json {
                    context: "serializing to pretty JSON",
                    source: e,
                })?
            } else {
                serde_json::to_string(&desc).map_err(|e| Error::Json {
                    context: "serializing to JSON",
                    source: e,
                })?
            };
            println!("{json}");
        }
    }

    Ok(())
}
