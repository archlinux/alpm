#![doc = include_str!("../README.md")]

use std::{
    fs::File,
    io::{self, BufReader, IsTerminal, Read},
    path::PathBuf,
};

use cli::OutputFormat;
use error::Error;
use flate2::read::GzDecoder;
use mtree_v2::parse_mtree_v2;

pub mod cli;
pub mod error;
pub mod mtree_v2;
pub mod parser;

/// A small wrapper around the parsing of a Mtree file that simply ensures that there were no
/// errors.
pub fn validate(file: Option<&PathBuf>) -> Result<(), Error> {
    parse(file)?;

    Ok(())
}

/// Parse a given file and output it in the specified format to stdout.
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

/// These magic two bytes are located at the very front of gzip files.
/// This can be used to determine whether a file is a gzip file or not.
const GZIP_MAGIC_NUMBER: [u8; 2] = [0x1f, 0x8b];

/// Parse and interpret a Mtree file.
///
/// 1. Reads the contents of a file or stdin.
/// 2. Check whether the input is gzip compressed as that's how it's delivered inside of packages.
/// 3. Parse the input
///
/// NOTE: If a command is piped to this process, the input is read from stdin.
/// See [`IsTerminal`] for more information about how terminal detection works.
///
/// [`IsTerminal`]: https://doc.rust-lang.org/stable/std/io/trait.IsTerminal.html
pub fn parse(file: Option<&PathBuf>) -> Result<Vec<mtree_v2::Path>, Error> {
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

    // Check if the file starts with `0x1f8b`, which is the magic number that marks files
    // as gzip compressed. If that's the case, decompress the content first.
    let contents = if buffer.len() >= 2 && [buffer[0], buffer[1]] == GZIP_MAGIC_NUMBER {
        let mut decoder = GzDecoder::new(buffer.as_slice());

        let mut content = String::new();
        decoder.read_to_string(&mut content).unwrap();
        content
    } else {
        String::from_utf8(buffer)?.to_string()
    };

    // Parse the given mtree file.
    parse_mtree_v2(contents)
}
