//! Utils for the running of rootless backends.

use std::path::PathBuf;

use which::which;

use crate::Error;

/// Returns the path to a `command`.
///
/// Searches for an executable in `$PATH` of the current environment and returns the first one
/// found.
///
/// # Errors
///
/// Returns an error if no executable matches the provided `command`.
pub fn get_command(command: &str) -> Result<PathBuf, Error> {
    which(command).map_err(|source| Error::ExecutableNotFound {
        command: command.to_string(),
        source,
    })
}
