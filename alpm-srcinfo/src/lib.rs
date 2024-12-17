#![doc = include_str!("../README.md")]

use std::{
    fs::File,
    io::{self, BufReader, IsTerminal, Read},
    path::PathBuf,
};

use error::Error;
use winnow::Parser;

/// Commandline argument handling. This is most likely not interesting for you.
pub mod cli;
/// All error types that're exposed by this crate.
pub mod error;
/// This module contains the first parsing pass a SRCINFO file.
/// It returns a rather raw line-based, but already typed representation of the file content.
/// This isn't yet useful for consumers and needs to be further processed.
pub mod parser;

/// A small wrapper around the parsing of a Srcinfo file that simply ensures that there were no
/// errors.
pub fn validate(file: Option<&PathBuf>) -> Result<(), Error> {
    parse(file)?;

    Ok(())
}

/// Parse and interpret a Srcinfo file.
///
/// NOTE: If a command is piped to this process, the input is read from stdin.
/// See [`IsTerminal`] for more information about how terminal detection works.
///
/// [`IsTerminal`]: https://doc.rust-lang.org/stable/std/io/trait.IsTerminal.html
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
    parser::srcinfo
        .parse(&contents)
        .map_err(|err| Error::ParseError(format!("{err}")))?;

    Ok(())
}
