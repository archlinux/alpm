use std::process::Output;

use anyhow::{Result, bail};

/// Make sure a command finished successfully, otherwise throw an error.
pub fn ensure_success(output: &Output) -> Result<()> {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    if !output.status.success() {
        bail!("Failed to run command:\nstdout:\n{stdout}\nstderr:\n{stderr}");
    }

    Ok(())
}
