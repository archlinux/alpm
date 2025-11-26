//! Command line interface for interacting with [alpm-repo-files] files.
//!
//! [alpm-repo-files]: https://alpm.archlinux.page/specifications/alpm-repo-files.5.html

use std::{
    fs::{File, read_to_string},
    io::{IsTerminal, Read, Write, stdin, stdout},
    path::PathBuf,
    process::ExitCode,
    str::FromStr,
};

use alpm_repo_db::files::{
    RepoFiles,
    RepoFilesV1,
    cli::{Cli, Command, OutputFormat},
};
use clap::Parser;
use fluent_i18n::t;

// Initialize i18n support.
fluent_i18n::i18n!("locales");

#[derive(Debug, thiserror::Error)]
enum Error {
    /// An [`alpm_repo_db::files::Error`] occurred.
    #[error(transparent)]
    AlpmFiles(#[from] alpm_repo_db::files::Error),

    /// A JSON error occurred.
    #[error("{msg}", msg = t!("cli-error-json", { "context" => context, "source" => source.to_string() }))]
    Json {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "JSON error while ".
        /// See the fluent-i18n token "cli-error-json" for details.
        context: String,
        /// The source error.
        source: serde_json::Error,
    },

    /// The process stdin is a terminal.
    #[error("{msg}", msg = t!("cli-error-stdin-is-terminal"))]
    StdinIsTerminal,
}

/// Creates [`alpm-repo-files`] data from a directory.
///
/// Outputs data on [`stdout`] if no `output` is provided.
/// If `output` is provided, attempts to write data to that file.
/// The [alpm-repo-files] formatting matches the specification.
///
/// # Errors
///
/// Returns an error if
///
/// - no [`RepoFiles`] can be created from `input_dir`,
/// - `output` is provided, but cannot be opened for writing or be written to,
/// - `output` is not provided and [`stdout`] cannot be written to.
///
/// [alpm-repo-files]: https://alpm.archlinux.page/specifications/alpm-repo-files.5.html
fn create_from_dir(input_dir: PathBuf, output: Option<PathBuf>) -> Result<(), Error> {
    let files = RepoFiles::V1(RepoFilesV1::try_from(input_dir)?);

    if let Some(output) = output {
        let mut output_file =
            File::create(&output).map_err(|source| alpm_repo_db::files::Error::IoPath {
                path: output.to_path_buf(),
                context: t!("cli-error-io-path-opening-output-file-for-writing"),
                source,
            })?;
        write!(output_file, "{files}").map_err(|source| alpm_repo_db::files::Error::Io {
            context: t!("cli-error-io-writing-to-output-file"),
            source,
        })?;
    } else {
        stdout()
            .write(&files.to_string().into_bytes())
            .map_err(|source| alpm_repo_db::files::Error::Io {
                context: t!("cli-error-io-writing-to-stdout"),
                source,
            })?;
    }

    Ok(())
}

/// Formats [`alpm-repo-files`] data as another file format.
///
/// If no `input_file` is provided, data is read from [`stdin`].
/// If `output` is provided, attempts to write data to that file.
/// The output format depends on the chosen `format` and whether it is `pretty` printed (if
/// applicable).
///
/// # Errors
///
/// Returns an error if
///
/// - an `input_file` is provided, but cannot be read to string,
/// - an `input_file` is not provided and [`stdin`] is a terminal,
/// - [`stdin`] cannot be read to string,
/// - a [`RepoFiles`] cannot be created from the [`alpm-repo-files`] data,
/// - the chosen `format` is JSON and serializing the [`RepoFiles`] data fails,
/// - an `output` is provided, but cannot be created, or written to,
/// - or an `output` is not provided and [`stdout`] cannot be written to.
///
/// [alpm-repo-files]: https://alpm.archlinux.page/specifications/alpm-repo-files.5.html
fn format_output(
    input_file: Option<PathBuf>,
    output: Option<PathBuf>,
    format: OutputFormat,
    pretty: bool,
) -> Result<(), Error> {
    let files = if let Some(file) = input_file {
        RepoFiles::from_str(&read_to_string(&file).map_err(|source| {
            alpm_repo_db::files::Error::IoPath {
                path: file.to_path_buf(),
                context: t!("cli-reading-file-to-string"),
                source,
            }
        })?)?
    } else {
        if stdin().is_terminal() {
            return Err(Error::StdinIsTerminal);
        }

        let mut buf = String::new();
        stdin()
            .read_to_string(&mut buf)
            .map_err(|source| alpm_repo_db::files::Error::Io {
                context: t!("cli-reading-stdin-to-string"),
                source,
            })?;
        RepoFiles::from_str(&buf)?
    };

    let data = match format {
        OutputFormat::Json => {
            let mut output = if pretty {
                serde_json::to_string_pretty(&files).map_err(|source| Error::Json {
                    context: t!(
                        "cli-error-json-serializing-alpm-repo-files-data-as-pretty-printed-json-string"
                    ),
                    source,
                })?
            } else {
                serde_json::to_string(&files).map_err(|source| Error::Json {
                    context: t!("cli-error-json-serializing-alpm-repo-files-data-as-json-string"),
                    source,
                })?
            };
            output.push('\n');
            output
        }
        OutputFormat::V1 => files.to_string(),
    };

    if let Some(output) = output {
        let mut output_file =
            File::create(&output).map_err(|source| alpm_repo_db::files::Error::IoPath {
                path: output.to_path_buf(),
                context: t!("cli-opening-output-file-for-writing"),
                source,
            })?;
        write!(output_file, "{data}").map_err(|source| alpm_repo_db::files::Error::Io {
            context: t!("cli-writing-to-output-file"),
            source,
        })?;
    } else {
        write!(stdout(), "{data}").map_err(|source| alpm_repo_db::files::Error::Io {
            context: t!("cli-writing-to-stdout"),
            source,
        })?;
    }

    Ok(())
}

/// Validates [`alpm-repo-files`] data.
///
/// If no `input_file` is provided, data is read from [`stdin`].
///
/// # Errors
///
/// Returns an error if
///
/// - `input_file` is provided, but is not readable,
/// - no [`RepoFiles`] can be created from `input_file` or [`stdin`],
/// - or no `input_file` is provided and [`stdin`] is a terminal.
///
/// [alpm-repo-files]: https://alpm.archlinux.page/specifications/alpm-repo-files.5.html
fn validate_input(input_file: Option<PathBuf>) -> Result<(), Error> {
    if let Some(file) = input_file {
        RepoFiles::from_str(&read_to_string(&file).map_err(|source| {
            alpm_repo_db::files::Error::IoPath {
                path: file.to_path_buf(),
                context: t!("cli-error-io-path-reading-file-to-string"),
                source,
            }
        })?)?;
    } else {
        if stdin().is_terminal() {
            return Err(Error::StdinIsTerminal);
        }

        let mut buf = String::new();
        stdin()
            .read_to_string(&mut buf)
            .map_err(|source| alpm_repo_db::files::Error::Io {
                context: t!("cli-error-io-reading-stdin-to-string"),
                source,
            })?;
        RepoFiles::from_str(&buf)?;
    }

    Ok(())
}

/// Runs the `alpm-repo-files` executable.
///
/// Depending on [`Cli`], delegates to `alpm_repo_create`, `alpm_repo_format` or
/// `alpm_repo_validate`.
/// In case of success, exits with [`ExitCode::SUCCESS`].
///
/// If an error occurs, the error message is emitted on stderr and the executable exits with
/// [`ExitCode::FAILURE`].
fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Create { input_dir, output } => create_from_dir(input_dir, output),
        Command::Format {
            input_file,
            output,
            format,
            pretty,
        } => format_output(input_file, output, format, pretty),
        Command::Validate { input_file } => validate_input(input_file),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
