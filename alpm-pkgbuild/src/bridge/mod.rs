pub mod error;
pub mod parser;
pub mod source_info;

use std::{
    io::ErrorKind,
    path::Path,
    process::{Command, Stdio},
};

use log::debug;

use crate::error::Error;

/// Run the `pkgbuild-bridge.sh` script, which exposes all relevant information of a PKGBUILD
/// in a custom format.
///
/// Returns the output of that script as a String.
///
/// ```
/// use std::{fs::File, io::Write};
///
/// use alpm_pkgbuild::bridge::run_bridge_script;
/// use tempfile::tempdir;
///
/// const TEST_FILE: &str = include_str!("../../tests/unit_test_files/normal.pkgbuild");
///
/// # use testresult::TestResult;
/// # fn main() -> TestResult {
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
/// - The path could not be found.
/// - The path doesn't has a filename.
/// - The path doesn't point to a file.
/// - The bridge script failed for some reason.
pub fn run_bridge_script(pkgbuild_path: &Path) -> Result<String, Error> {
    // Make sure the PKGBUILD path exists.
    if !pkgbuild_path.exists() {
        let err = std::io::Error::new(ErrorKind::NotFound, "No such file or directory.");
        return Err(Error::IoPath(
            pkgbuild_path.to_path_buf(),
            "checking for PKGBUILD",
            err,
        ));
    }

    // Make sure the PKGBUILD path contains a filename.
    let Some(filename) = pkgbuild_path.file_name() else {
        return Err(Error::InvalidFile(
            pkgbuild_path.to_owned(),
            "No filename provided in path",
        ));
    };

    // Make sure the PKGBUILD path actually shows to a file.
    let metadata = pkgbuild_path
        .metadata()
        .map_err(|err| Error::IoPath(pkgbuild_path.to_owned(), "getting metadata of file", err))?;
    if !metadata.file_type().is_file() {
        return Err(Error::InvalidFile(
            pkgbuild_path.to_owned(),
            "Path doesn't point to a file.",
        ));
    };

    let mut command = Command::new("alpm-pkgbuild-bridge.sh");
    // Change the CWD to the directory that contains the PKGBUILD
    if let Some(parent) = pkgbuild_path.parent() {
        // `parent` returns an empty path for relative paths with a single component.
        if parent != Path::new("") {
            command.current_dir(parent);
        }
    }

    let args = vec![filename.to_string_lossy().to_string()];
    command.args(&args);

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    debug!("Spawning command 'bash {}'", args.join(" "));
    let child = command
        .spawn()
        .map_err(|err| Error::ScriptError("spawn", args.clone(), err))?;

    debug!("Waiting for bash to finish");
    let output = child
        .wait_with_output()
        .map_err(|err| Error::ScriptError("finish", args.clone(), err))?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Error::ScriptExecutionError {
            arguments: args,
            stdout,
            stderr,
        });
    }

    String::from_utf8(output.stdout).map_err(Error::from)
}
