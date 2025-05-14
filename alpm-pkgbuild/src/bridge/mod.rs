//! The `PKGBUILD` to `.SRCINFO` bridge logic.

pub mod error;
pub(crate) mod parser;
pub mod source_info;

use std::{
    io::ErrorKind,
    path::Path,
    process::{Command, Stdio},
};

use log::debug;
use which::which;

use crate::error::Error;

const DEFAULT_SCRIPT_NAME: &str = "alpm-pkgbuild-bridge";

/// Runs the [`alpm-pkgbuild-bridge`] script, which exposes all relevant information of a
/// [`PKGBUILD`] in a custom format.
///
/// Returns the output of that script as a `String`.
///
/// # Examples
///
/// ```
/// use std::{fs::File, io::Write};
///
/// use alpm_pkgbuild::bridge::run_bridge_script;
/// use tempfile::tempdir;
///
/// const TEST_FILE: &str = include_str!("../../tests/test_files/normal.pkgbuild");
///
/// # fn main() -> testresult::TestResult {
/// // Create a temporary directory where we write the PKGBUILD to.
/// let temp_dir = tempdir()?;
/// let path = temp_dir.path().join("PKGBUILD");
/// let mut file = File::create(&path)?;
/// file.write_all(TEST_FILE.as_bytes());
///
/// // Call the bridge script on that path.
/// println!("{}", run_bridge_script(&path)?);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if
///
/// - `pkgbuild_path` does not exist,
/// - `pkgbuild_path` does not have a file name,
/// - `pkgbuild_path` is not a file,
/// - or running the `alpm-pkgbuild-bridge` script fails.
///
/// [`PKGBUILD`]: https://man.archlinux.org/man/PKGBUILD.5
/// [`alpm-pkgbuild-bridge`]: https://gitlab.archlinux.org/archlinux/alpm/alpm-pkgbuild-bridge
pub fn run_bridge_script(pkgbuild_path: &Path) -> Result<String, Error> {
    // Make sure the PKGBUILD path exists.
    if !pkgbuild_path.exists() {
        let source = std::io::Error::new(ErrorKind::NotFound, "No such file or directory.");
        return Err(Error::IoPath {
            path: pkgbuild_path.to_path_buf(),
            context: "checking for PKGBUILD",
            source,
        });
    }

    // Make sure the PKGBUILD path contains a filename.
    let Some(filename) = pkgbuild_path.file_name() else {
        return Err(Error::InvalidFile {
            path: pkgbuild_path.to_owned(),
            context: "No filename provided in path",
        });
    };

    // Make sure the PKGBUILD path actually points to a file.
    let metadata = pkgbuild_path.metadata().map_err(|source| Error::IoPath {
        path: pkgbuild_path.to_owned(),
        context: "getting metadata of file",
        source,
    })?;
    if !metadata.file_type().is_file() {
        return Err(Error::InvalidFile {
            path: pkgbuild_path.to_owned(),
            context: "Path doesn't point to a file.",
        });
    };

    let script_path = which(DEFAULT_SCRIPT_NAME).map_err(|source| Error::ScriptNotFound {
        script_name: DEFAULT_SCRIPT_NAME.to_string(),
        source,
    })?;

    let mut command = Command::new(script_path);
    // Change the CWD to the directory that contains the PKGBUILD
    if let Some(parent) = pkgbuild_path.parent() {
        // `parent` returns an empty path for relative paths with a single component.
        if parent != Path::new("") {
            command.current_dir(parent);
        }
    }

    let parameters = vec![filename.to_string_lossy().to_string()];
    command.args(&parameters);

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    debug!(
        "Spawning command '{DEFAULT_SCRIPT_NAME} {}'",
        parameters.join(" ")
    );
    let child = command.spawn().map_err(|source| Error::ScriptError {
        context: "spawn",
        parameters: parameters.clone(),
        source,
    })?;

    debug!("Waiting for '{DEFAULT_SCRIPT_NAME}' to finish");
    let output = child
        .wait_with_output()
        .map_err(|source| Error::ScriptError {
            context: "finish",
            parameters: parameters.clone(),
            source,
        })?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Error::ScriptExecutionError {
            parameters,
            stdout,
            stderr,
        });
    }

    String::from_utf8(output.stdout).map_err(Error::from)
}
