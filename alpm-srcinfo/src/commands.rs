use std::{
    io::{self, IsTerminal, Read},
    path::PathBuf,
};

use alpm_types::Architecture;

use crate::{
    cli::OutputFormat,
    error::{Error, SourceInfoErrors},
    merged::MergedPackage,
    source_info::SourceInfo,
};

/// Parse a SRCINFO file at a path or StdIn  and ensure that there were no critical parser or
/// logical errors.
/// Called when running the `validate` subcommand.
pub fn validate(file: Option<&PathBuf>) -> Result<(), Error> {
    let (_, errors) = parse(file)?;

    // Check if there're any unrecoverable errors.
    if let Some(errors) = errors {
        errors.check_unrecoverable_errors()?;
    };

    Ok(())
}

/// Parse a SRCINFO file at a path or StdIn and ensure and ensure that there were no errors, logical
/// errors and not even linter errors.
/// Called when running the `check` subcommand.
pub fn check(file: Option<&PathBuf>) -> Result<(), Error> {
    let (_, errors) = parse(file)?;

    if let Some(mut errors) = errors {
        errors.sort_errors();
        return Err(Error::SourceInfoErrors(errors));
    }

    Ok(())
}

/// Parse a SRCINFO file at a path or StdIn and output it in the specified format to stdout.
/// Called when running the `format` subcommand.
///
/// # Errors
///
/// Returns an error if the input can not be parsed and validated, or if the output can not be
/// formatted in the selected output format.
pub fn format(
    file: Option<&PathBuf>,
    output_format: OutputFormat,
    architecture: Architecture,
    pretty: bool,
) -> Result<(), Error> {
    let (source_info, _errors) = parse(file)?;

    let packages: Vec<MergedPackage> = source_info
        .packages_for_architecture(architecture)
        .collect();

    match output_format {
        OutputFormat::Json => {
            let json = if pretty {
                serde_json::to_string_pretty(&packages)?
            } else {
                serde_json::to_string(&packages)?
            };
            println!("{json}");
        }
    }

    Ok(())
}

/// Parse and interpret a Srcinfo file.
///
/// NOTE: If a command is piped to this process, the input is read from stdin.
/// See [`IsTerminal`] for more information about how terminal detection works.
///
/// [`IsTerminal`]: https://doc.rust-lang.org/stable/std/io/trait.IsTerminal.html
///
/// # Errors
///
/// Returns an error if the input can not be parsed and validated, or if the output can not be
/// formatted in the selected output format.
///
/// Furthermore, returns an error array with potentially un/-recoverable (linting-)errors, this
/// needs to be explicitly handled by the user.
pub fn parse(file: Option<&PathBuf>) -> Result<(SourceInfo, Option<SourceInfoErrors>), Error> {
    if let Some(path) = file {
        // Read directly from file.
        SourceInfo::from_file(path)
    } else if !io::stdin().is_terminal() {
        // Read from stdin into string.
        let mut buffer = Vec::new();
        let mut stdin = io::stdin();
        stdin
            .read_to_end(&mut buffer)
            .map_err(|err| Error::Io("reading from stdin", err))?;
        let content = String::from_utf8(buffer)?.to_string();

        // Convert into SourceInfo
        SourceInfo::from_string(&content)
    } else {
        Err(Error::NoInputFile)
    }
}
