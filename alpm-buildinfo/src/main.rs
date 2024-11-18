use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::File;
use std::io;
use std::io::IsTerminal;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

use alpm_buildinfo::cli::Cli;
use alpm_buildinfo::cli::Command;
use alpm_buildinfo::cli::CreateCommand;
use alpm_buildinfo::schema::Schema;
use alpm_buildinfo::BuildInfoV1;
use alpm_buildinfo::BuildInfoV2;
use alpm_buildinfo::Error;
use alpm_types::digests::Sha256;
use alpm_types::Checksum;
use alpm_types::SchemaVersion;
use clap::Parser;

/// Create a file according to a BUILDINFO schema
fn create_file(command: CreateCommand) -> Result<(), Error> {
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
                Checksum::<Sha256>::from_str(&args.pkgbuild_sha256sum).map_err(|_| {
                    Error::Default(format!(
                        "The provided SHA-256 checksum is not valid: {}",
                        &args.pkgbuild_sha256sum,
                    ))
                })?,
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
                Checksum::<Sha256>::from_str(&args.pkgbuild_sha256sum).map_err(|_| {
                    Error::Default(format!(
                        "The provided SHA-256 checksum is not valid: {}",
                        &args.pkgbuild_sha256sum,
                    ))
                })?,
                args.pkgname,
                args.pkgver,
            )?
            .to_string(),
            args.output,
        ),
    };

    // create any parent directories if necessary
    if let Some(output_dir) = output.0.parent() {
        create_dir_all(output_dir)
            .map_err(|_| Error::FailedDirCreation(format!("{}", output_dir.display())))?;
    }

    let mut out = File::create(&output.0)
        .map_err(|_| Error::FailedFileCreation(format!("{}", output.0.display())))?;
    out.write(data.as_bytes())
        .map_err(|_| Error::FailedWriting(format!("{}", output.0.display())))?;

    Ok(())
}

/// Validate a file according to a BUILDINFO schema.
///
/// This function reads the contents of a file or stdin, and validates it according to a schema.
///
/// NOTE: If a command is piped to this process, the input is read from stdin.
/// See [`IsTerminal`] for more information about how terminal detection works.
///
/// [`IsTerminal`]: https://doc.rust-lang.org/stable/std/io/trait.IsTerminal.html
fn validate(file: &Option<PathBuf>, schema: &Option<Schema>) -> Result<(), Error> {
    let contents = if let Some(file) = file {
        read_to_string(file).map_err(|_| Error::FailedReadingFile(format!("{}", file.display())))?
    } else if !io::stdin().is_terminal() {
        let mut buffer = Vec::new();
        let mut stdin = io::stdin();
        stdin
            .read_to_end(&mut buffer)
            .map_err(|e| Error::FailedReadingStdin(e.to_string()))?;
        String::from_utf8(buffer)?.to_string()
    } else {
        return Err(Error::NoInputFile);
    };

    // Determine the schema that should be used to validate the file.
    // If no explicit schema version is provided, the version will be deducted from the contents of
    // the file itself. If the file does not contain a version, an error will be returned.
    let schema = if let Some(schema) = schema {
        schema.clone()
    } else {
        Schema::from_contents(&contents)?
    };

    match schema {
        Schema::V1(_) => {
            BuildInfoV1::from_str(&contents)?;
        }
        Schema::V2(_) => {
            BuildInfoV2::from_str(&contents)?;
        }
        _ => return Err(Error::UnsupportedSchemaVersion(schema.inner().clone())),
    };

    Ok(())
}

/// Implements helper for exit code handling
trait ExitResult {
    fn handle_exit_code(&self);
}

impl ExitResult for Result<(), Error> {
    /// Handle a [`Result`] by differing exit code and potentially printing to stderr
    ///
    /// If `self` contains `()`, exit with an exit code of 0.
    /// If `self` contains [`Error`], print it to stderr and exit with an exit code of 1.
    fn handle_exit_code(&self) {
        match self {
            Ok(_) => exit(0),
            Err(error) => {
                eprintln!("{}", error);
                exit(1)
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Create { command } => create_file(command).handle_exit_code(),
        Command::Validate { file, schema } => validate(&file, &schema).handle_exit_code(),
    }
}
