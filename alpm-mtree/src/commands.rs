use std::{
    io::{self, IsTerminal},
    path::PathBuf,
};

use alpm_common::MetadataFile;

use crate::{Error, Mtree, MtreeSchema, cli::OutputFormat};

/// A small wrapper around the parsing of an MTREE file that simply ensures that there were no
/// errors.
///
/// For all possible errors, check the [parse] function.
pub fn validate(file: Option<&PathBuf>, schema: Option<MtreeSchema>) -> Result<(), Error> {
    parse(file, schema)?;

    Ok(())
}

/// Parse a given file and output it in the specified format to stdout.
///
/// # Errors
///
/// Returns an error if the input can not be parsed and validated, or if the output can not be
/// formatted in the selected output format.
pub fn format(
    file: Option<&PathBuf>,
    schema: Option<MtreeSchema>,
    format: OutputFormat,
    pretty: bool,
) -> Result<(), Error> {
    let files = parse(file, schema)?;

    match format {
        OutputFormat::Json => {
            let json = if pretty {
                serde_json::to_string_pretty(&files)?
            } else {
                serde_json::to_string(&files)?
            };
            println!("{json}");
        }
    }

    Ok(())
}

/// Parse and interpret an MTREE file.
///
/// 1. Reads the contents of a file or stdin.
/// 2. Check whether the input is gzip compressed as that's how it's delivered inside of packages.
/// 3. Parse the input
///
/// NOTE: If a command is piped to this process, the input is read from stdin.
/// See [`IsTerminal`] for more information about how terminal detection works.
///
/// [`IsTerminal`]: https://doc.rust-lang.org/stable/std/io/trait.IsTerminal.html
///
/// # Errors
///
/// - [Error::NoInputFile] if a file is given and doesn't exist.
/// - [Error::IoPath] if a given file cannot be opened or read.
/// - [Error::Io] if the file is streamed via StdIn and an error occurs.
/// - [Error::InvalidGzip] if the file is gzip compressed, but the archive is malformed.
/// - [Error::InvalidUTF8] if the given file contains invalid UTF-8.
/// - [Error::ParseError] if a malformed MTREE file is encountered.
/// - [Error::InterpreterError] if expected properties for a given type aren't set.
pub fn parse(file: Option<&PathBuf>, schema: Option<MtreeSchema>) -> Result<Mtree, Error> {
    if let Some(file) = file {
        Mtree::from_file_with_schema(file, schema)
    } else if !io::stdin().is_terminal() {
        Mtree::from_stdin_with_schema(schema)
    } else {
        Err(Error::NoInputFile)
    }
}
