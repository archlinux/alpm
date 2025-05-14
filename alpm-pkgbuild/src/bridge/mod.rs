pub mod error;
pub mod parser;
pub mod source_info;

use std::{
    io::{ErrorKind, Write},
    path::Path,
    process::{Command, Stdio},
};

use log::debug;

use crate::error::Error;

pub const PKGBUILD_SCRIPT: &str = include_str!("../../script/pkgbuild-bridge.sh");

pub fn run_bridge_script(pkgbuild_path: &Path) -> Result<String, Error> {
    if !pkgbuild_path.exists() {
        let err = std::io::Error::new(ErrorKind::NotFound, "No such file or directory.");
        return Err(Error::IoPath(
            pkgbuild_path.to_path_buf(),
            "checking for PKGBUILD",
            err,
        ));
    }

    let mut command = Command::new("bash");
    let args = vec![
        "--noprofile".into(),
        "--norc".into(),
        "-s".into(),
        "-".into(),
        pkgbuild_path.to_string_lossy().to_string(),
    ];
    command.args(&args);

    // Make sure the command is executed in the directory that contains the PKGBUILD.
    if let Some(parent) = pkgbuild_path.parent() {
        command.current_dir(parent);
    }

    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    debug!("Spawning command 'bash {}'", args.join(" "));
    let mut child = command
        .spawn()
        .map_err(|err| Error::ScriptError("spawn", args.clone(), err))?;

    debug!("Piping script into command");
    let mut stdin = child.stdin.take().unwrap();
    stdin
        .write_all(PKGBUILD_SCRIPT.as_bytes())
        .map_err(|err| Error::ScriptError("send script to stdin of", args.clone(), err))?;
    drop(stdin);

    debug!("Waiting for bash to finish");
    let output = child
        .wait_with_output()
        .map_err(|err| Error::ScriptError("finish", args, err))?;

    String::from_utf8(output.stdout).map_err(Error::from)
}
