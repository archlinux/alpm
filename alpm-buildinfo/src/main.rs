use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::MAIN_SEPARATOR;
use std::process::exit;
use std::str::FromStr;

use alpm_buildinfo::cli::Cli;
use alpm_buildinfo::cli::Command;
use alpm_buildinfo::cli::CreateCommand;
use alpm_buildinfo::cli::ExportCommand;
use alpm_buildinfo::cli::Schema;
use alpm_buildinfo::BuildInfoV1;
use alpm_buildinfo::Error;

use alpm_types::digests::Sha256;
use alpm_types::Checksum;
use alpm_types::SchemaVersion;
use clap::CommandFactory;
use clap::Parser;

use clap_complete::generate_to;
use clap_complete::Shell;
use clap_mangen::Man;

/// Create a file according to a BUILDINFO schema
fn create_file(command: CreateCommand) -> Result<(), Error> {
    let (data, output) = match command {
        CreateCommand::V1 {
            builddate,
            builddir,
            buildenv,
            installed,
            options,
            packager,
            pkgarch,
            pkgbase,
            pkgbuild_sha256sum,
            pkgname,
            pkgver,
            output,
        } => (
            BuildInfoV1::new(
                builddate,
                builddir,
                buildenv,
                SchemaVersion::from_str("1")?,
                installed,
                options,
                packager,
                pkgarch,
                pkgbase,
                Checksum::<Sha256>::from_str(&pkgbuild_sha256sum).map_err(|_| {
                    Error::Default(format!(
                        "The provided SHA-256 checksum is not valid: {}",
                        &pkgbuild_sha256sum,
                    ))
                })?,
                pkgname,
                pkgver,
            )?
            .to_string(),
            output,
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

/// Validate a file according to a BUILDINFO schema
fn validate_file(file: &Path, schema: &Schema) -> Result<(), Error> {
    let contents =
        read_to_string(file).map_err(|_| Error::FailedReading(format!("{}", file.display())))?;

    match schema {
        Schema::V1 => BuildInfoV1::from_str(&contents)?,
        _ => unimplemented!("Unimplemented schema!"),
    };

    Ok(())
}

/// Render shell completion files to an output directory
fn render_shell_completions(output_dir: &Path) -> Result<(), Error> {
    create_dir_all(output_dir)
        .map_err(|_| Error::FailedDirCreation(format!("{}", output_dir.display())))?;

    let mut command = Cli::command();

    for shell in &[
        Shell::Bash,
        Shell::Elvish,
        Shell::Fish,
        Shell::PowerShell,
        Shell::Zsh,
    ] {
        generate_to(*shell, &mut command, env!("CARGO_BIN_NAME"), output_dir).map_err(|_| {
            Error::FailedFileCreation(format!(
                "{}{}{}",
                output_dir.display(),
                MAIN_SEPARATOR,
                shell
            ))
        })?;
    }
    Ok(())
}

/// Render man pages to an output directory
fn render_manpages(output_dir: &Path) -> Result<(), Error> {
    /// Render man pages for commands and subcommands recursively
    fn render_recursive(
        output_dir: &Path,
        command: &mut clap::Command,
        prefix: &str,
    ) -> Result<(), Error> {
        // prefix name with name of parent command if we are a subcommand
        // NOTE: this is not ideal yet, as we are getting e.g. `alpm-buildinfo-create-v1` instead of
        // `alpm-buildinfo create v1` in SYNOPSIS, however this is due to a clap_mangen limitation:
        // https://github.com/clap-rs/clap/discussions/3603
        let name = if !prefix.is_empty() {
            format!("{}-{}", prefix, command.get_name())
        } else {
            command.get_name().to_string()
        };

        let command = &mut command.clone().name(&name);

        let mut out = File::create(output_dir.join(format!("{name}.1"))).map_err(|_| {
            Error::FailedFileCreation(format!(
                "{}",
                output_dir.join(format!("{name}.1")).display()
            ))
        })?;
        Man::new(command.clone()).render(&mut out).map_err(|_| {
            Error::FailedFileCreation(format!(
                "{}",
                output_dir.join(format!("{name}.1")).display()
            ))
        })?;
        out.flush().map_err(|_| {
            Error::FailedFileCreation(format!(
                "{}",
                output_dir.join(format!("{name}.1")).display()
            ))
        })?;

        // get the current command's name to prefix any further subcommands
        let cmd_name = command.get_name().to_string();
        for subcommand in command.get_subcommands_mut() {
            // we do not want man pages for the help subcommands
            if !subcommand.get_name().contains("help") {
                render_recursive(output_dir, subcommand, &cmd_name)?;
            }
        }

        Ok(())
    }

    create_dir_all(output_dir)
        .map_err(|_| Error::FailedDirCreation(format!("{}", output_dir.display())))?;

    let mut command = Cli::command();
    command.build();
    render_recursive(output_dir, &mut command, "")?;

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
        Command::Validate { schema, file } => validate_file(&file, &schema).handle_exit_code(),
        Command::Export(command) => match command {
            ExportCommand::ShellCompletion { output } => {
                render_shell_completions(&output).handle_exit_code()
            }
            ExportCommand::Manpage { output } => render_manpages(&output).handle_exit_code(),
        },
    }
}
