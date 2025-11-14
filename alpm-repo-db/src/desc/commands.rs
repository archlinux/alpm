//! Commands for creating, validating, parsing, and formatting [alpm-repo-desc] files.
//!
//! [alpm-repo-desc]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html

use std::{
    fs::{File, create_dir_all},
    io::{IsTerminal, Write, stdin},
};

use alpm_common::MetadataFile;

use crate::{
    Error,
    desc::{
        RepoDescFile,
        RepoDescFileV1,
        RepoDescFileV2,
        cli::{CreateCommand, OutputFormat, ValidateArgs},
    },
};

/// Creates an [alpm-repo-desc] file according to the specified version command.
///
/// [alpm-repo-desc]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html
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
        CreateCommand::V1 { args, md5sum, pgpsig } => {
            let v1 = RepoDescFileV1 {
                file_name: args.filename,
                name: args.name,
                base: args.base,
                version: args.version,
                description: args.description.or_else(PackageDescription::new("")?),
                groups: args.groups,
                compressed_size: args.csize,
                installed_size: args.isize,
                md5_checksum: md5sum,
                sha256_checksum: args.sha256sum,
                pgp_signature: pgpsig,
                url: args.url,
                license: args.license,
                arch: args.arch,
                build_date: args.builddate,
                packager: args.packager,
                replaces: args.replaces,
                conflicts: args.conflicts,
                provides: args.provides,
                dependencies: args.depends,
                optional_dependencies: args.optdepends,
                make_dependencies: args.makedepends,
                check_dependencies: args.checkdepends,
            };
            (v1.to_string(), args.output)
        }
        CreateCommand::V2 { args, pgpsig } => {
            let v2 = RepoDescFileV2 {
                file_name: args.filename,
                name: args.name,
                base: args.base,
                version: args.version,
                description: args.description.or_else(PackageDescription::new("")?),
                groups: args.groups,
                compressed_size: args.csize,
                installed_size: args.isize,
                sha256_checksum: args.sha256sum,
                pgp_signature: pgpsig,
                url: args.url,
                license: args.license,
                arch: args.arch,
                build_date: args.builddate,
                packager: args.packager,
                replaces: args.replaces,
                conflicts: args.conflicts,
                provides: args.provides,
                dependencies: args.depends,
                optional_dependencies: args.optdepends,
                make_dependencies: args.makedepends,
                check_dependencies: args.checkdepends,
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

/// Parses an [alpm-repo-desc] file and returns it as a [`RepoDescFile`].
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
/// [alpm-repo-desc]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html
pub fn parse(args: ValidateArgs) -> Result<RepoDescFile, Error> {
    if let Some(file) = &args.file {
        RepoDescFile::from_file_with_schema(file, args.schema)
    } else if !stdin().is_terminal() {
        RepoDescFile::from_stdin_with_schema(args.schema)
    } else {
        Err(Error::NoInputFile)
    }
}

/// Validate a package repository desc file by parsing it.
pub fn validate(args: ValidateArgs) -> Result<(), Error> {
    let _ = parse(args)?;
    Ok(())
}

/// Formats and optionally pretty prints an [alpm-repo-desc] file.
///
/// # Errors
///
/// Returns an error if
///
/// - the [alpm-repo-desc] file cannot be parsed,
/// - or the data cannot be formatted in the chosen output format.
///
/// [alpm-repo-desc]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html
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
