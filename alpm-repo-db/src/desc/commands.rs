//! Commands for creating, validating, parsing, and formatting [alpm-repo-desc] files.
//!
//! [alpm-repo-desc]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html

use std::{
    fs::{File, create_dir_all},
    io::{IsTerminal, Write, stdin},
};

use alpm_common::MetadataFile;
use fluent_i18n::t;

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
        CreateCommand::V1 {
            common,
            md5sum,
            pgpsig,
        } => {
            let v1 = RepoDescFileV1 {
                file_name: common.filename,
                name: common.name,
                base: common.base,
                version: common.version,
                description: common.description.unwrap_or_default(),
                groups: common.groups,
                compressed_size: common.csize,
                installed_size: common.isize,
                md5_checksum: md5sum,
                sha256_checksum: common.sha256sum,
                pgp_signature: pgpsig,
                url: common.url,
                license: common.license,
                arch: common.arch,
                build_date: common.builddate,
                packager: common.packager,
                replaces: common.replaces,
                conflicts: common.conflicts,
                provides: common.provides,
                dependencies: common.depends,
                optional_dependencies: common.optdepends,
                make_dependencies: common.makedepends,
                check_dependencies: common.checkdepends,
            };
            (v1.to_string(), common.output)
        }
        CreateCommand::V2 { common, pgpsig } => {
            let v2 = RepoDescFileV2 {
                file_name: common.filename,
                name: common.name,
                base: common.base,
                version: common.version,
                description: common.description.unwrap_or_default(),
                groups: common.groups,
                compressed_size: common.csize,
                installed_size: common.isize,
                sha256_checksum: common.sha256sum,
                pgp_signature: pgpsig,
                url: common.url,
                license: common.license,
                arch: common.arch,
                build_date: common.builddate,
                packager: common.packager,
                replaces: common.replaces,
                conflicts: common.conflicts,
                provides: common.provides,
                dependencies: common.depends,
                optional_dependencies: common.optdepends,
                make_dependencies: common.makedepends,
                check_dependencies: common.checkdepends,
            };
            (v2.to_string(), common.output)
        }
    };

    if let Some(output_path) = output {
        // Create parent directories if necessary
        if let Some(output_dir) = output_path.parent() {
            create_dir_all(output_dir).map_err(|source| Error::IoPath {
                path: output_dir.to_path_buf(),
                context: t!("error-io-create-output-dir"),
                source,
            })?;
        }

        let mut out = File::create(&output_path).map_err(|source| Error::IoPath {
            path: output_path.clone(),
            context: t!("error-io-create-output-file"),
            source,
        })?;

        out.write_all(data.as_bytes())
            .map_err(|source| Error::IoPath {
                path: output_path.clone(),
                context: t!("error-io-write-output-file"),
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
                    context: t!("error-json-serialize-pretty"),
                    source: e,
                })?
            } else {
                serde_json::to_string(&desc).map_err(|e| Error::Json {
                    context: t!("error-json-serialize"),
                    source: e,
                })?
            };
            println!("{json}");
        }
    }

    Ok(())
}
