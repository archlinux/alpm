//! Functions called from the binary.
use std::{
    fs::File,
    io::{self, BufReader, IsTerminal, Read},
    path::PathBuf,
};

use winnow::Parser;

use crate::{error::Error, parser::SourceInfoContent};

/// Validates a SRCINFO file from a path or stdin.
///
/// Wraps the [`parse`] function and allows to ensure that no errors occurred during parsing.
pub fn validate(file: Option<&PathBuf>) -> Result<(), Error> {
    parse(file)?;

    Ok(())
}

/// Parses and interprets a SRCINFO file from a path or stdin.
///
/// ## Note
///
/// If a command is piped to this process, the input is read from stdin.
/// See [`IsTerminal`] for more information about how terminal detection works.
///
/// [`IsTerminal`]: https://doc.rust-lang.org/stable/std/io/trait.IsTerminal.html
///
/// # Errors
///
/// Returns an error if the input can not be parsed and validated, or if the output can not be
/// formatted in the selected output format.
pub fn parse(file: Option<&PathBuf>) -> Result<(), Error> {
    // The buffer that'll contain the raw file.
    let mut buffer = Vec::new();

    // Read the file into the buffer, either from a given file or stdin.
    if let Some(path) = file {
        let file =
            File::open(path).map_err(|err| Error::IoPath(path.clone(), "opening file", err))?;
        let mut buf_reader = BufReader::new(file);
        buf_reader
            .read_to_end(&mut buffer)
            .map_err(|err| Error::IoPath(path.clone(), "reading file", err))?;
    } else if !io::stdin().is_terminal() {
        let mut buffer = Vec::new();
        let mut stdin = io::stdin();
        stdin
            .read_to_end(&mut buffer)
            .map_err(|err| Error::Io("reading from stdin", err))?;
    } else {
        return Err(Error::NoInputFile);
    };

    let contents = String::from_utf8(buffer)?.to_string();

    // Parse the given srcinfo file.
    SourceInfoContent::parser
        .parse(&contents)
        .map_err(|err| Error::ParseError(format!("{err}")))?;

    Ok(())
}
