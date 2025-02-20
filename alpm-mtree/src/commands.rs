use std::{
    fs::File,
    io::{self, BufReader, IsTerminal, Read},
    path::PathBuf,
};

use crate::{Error, cli::OutputFormat, mtree_v2::parse_raw_mtree_v2};

/// A small wrapper around the parsing of an MTREE file that simply ensures that there were no
/// errors.
///
/// For all possible errors, check the [parse] function.
pub fn validate(file: Option<&PathBuf>) -> Result<(), Error> {
    parse(file)?;

    Ok(())
}

/// Parse a given file and output it in the specified format to stdout.
///
/// # Errors
///
/// Returns an error if the input can not be parsed and validated, or if the output can not be
/// formatted in the selected output format.
pub fn format(file: Option<&PathBuf>, format: OutputFormat, pretty: bool) -> Result<(), Error> {
    let files = parse(file)?;

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
pub fn parse(file: Option<&PathBuf>) -> Result<Vec<crate::mtree_v2::Path>, Error> {
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

    // Parse the given mtree file.
    parse_raw_mtree_v2(buffer)
}
