use std::process::Output;

use crate::Error;

/// Make sure a command finished successfully, otherwise throw an error.
pub fn ensure_success(output: &Output, message: String) -> Result<(), Error> {
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    if !output.status.success() {
        return Err(Error::CommandFailed {
            message,
            stdout,
            stderr,
        });
    }

    Ok(())
}
