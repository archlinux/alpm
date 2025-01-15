use std::{
    io::{self, IsTerminal, Read},
    path::PathBuf,
};

use crate::{Error, SourceInfo, SourceInfoResult};

/// Validates a SRCINFO file from a path or stdin.
///
/// Wraps the [`parse`] function and allows to ensure that no errors occurred during parsing.
pub fn validate(file: Option<&PathBuf>) -> Result<(), Error> {
    let result = parse(file)?;
    result.source_info()?;

    Ok(())
}

/// Checks a SRCINFO file from a path or stdin strictly.
///
/// # Errors
///
/// Returns an error if any linter warnings, deprecation warnings, unrecoverable logic
/// of parsing errors are encountered while parsing the SRCINFO data.
pub fn check(file: Option<&PathBuf>) -> Result<(), Error> {
    let result = parse(file)?;
    result.lint()?;

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
///
/// Furthermore, returns an error array with potentially un/-recoverable (linting-)errors, which
/// needs to be explicitly handled by the caller.
pub fn parse(file: Option<&PathBuf>) -> Result<SourceInfoResult, Error> {
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
