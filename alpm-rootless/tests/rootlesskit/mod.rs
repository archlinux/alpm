//! Integration tests for [rootlesskit].
//!
//! [rootlesskit]: https://github.com/rootless-containers/rootlesskit

use std::{
    fs::read_dir,
    os::{linux::fs::MetadataExt, unix::fs::PermissionsExt},
    path::Path,
};

use alpm_rootless::{RootlessBackend, RootlesskitBackend, RootlesskitOptions, detect_virt};
use cargo_metadata::MetadataCommand;
use change_user_run::{create_users, run_command_as_user};
use log::{LevelFilter, debug};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use testresult::TestResult;

const RUN_ROOTLESSKIT: &str = "run-rootlesskit";

/// Initializes a global [`TermLogger`].
fn init_logger() {
    if TermLogger::init(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
    .is_err()
    {
        debug!("Not initializing another logger, as one is initialized already.");
    }
}

/// Recursively lists files, their permissions and ownership.
pub fn list_files_in_dir(path: impl AsRef<Path>) -> TestResult {
    let path = path.as_ref();
    let entries = read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let meta = entry.metadata()?;

        debug!(
            "{} {}/{} {entry:?}",
            meta.permissions().mode(),
            meta.st_uid(),
            meta.st_gid()
        );

        if meta.is_dir() {
            list_files_in_dir(entry.path())?;
        }
    }

    Ok(())
}

/// Ensures that on a Linux-based system, the [`FakerootBackend`] can be used to run a
/// command ([whoami]) as root.
///
/// [whoami]: https://man.archlinux.org/man/whoami.1
#[test]
#[cfg(target_os = "linux")]
fn rootlesskitbackend_run() -> TestResult {
    use std::{collections::HashMap, fs::copy, path::PathBuf};

    use alpm_rootless::SystemdDetectVirtOutput;

    init_logger();

    let virt = detect_virt()?;
    if virt.uses_namespaces() {
        panic!("Cannot run this test in an environment that requires kernel namespaces itself.")
    }

    let stdout = if virt == SystemdDetectVirtOutput::None {
        // Run the test as the current user.
        let backend = RootlesskitBackend::new(RootlesskitOptions::default());
        let output = backend.run(&["whoami"])?;
        if !output.status.success() {
            return Err(format!(
                "The test failed but should have succeeded:\nstderr: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        // Here we assume that we are root in a virtual machine!
        let metadata = MetadataCommand::new().exec()?;
        let target_dir = metadata.target_directory.into_std_path_buf();
        list_files_in_dir(target_dir.join("debug").join("examples"))?;

        let test_cmd = PathBuf::from("/usr/local/bin").join(RUN_ROOTLESSKIT);
        copy(
            target_dir
                .join("debug")
                .join("examples")
                .join(RUN_ROOTLESSKIT),
            test_cmd,
        )?;

        // We create a dedicated user to run the test as.
        let user = "testuser";
        create_users(&[user], None, None)?;

        let env_list = [
            "LLVM_PROFILE_FILE",
            "CARGO_LLVM_COV",
            "CARGO_LLVM_COV_SHOW_ENV",
            "CARGO_LLVM_COV_TARGET_DIR",
            "RUSTFLAGS",
            "RUSTDOCFLAGS",
        ];

        let output = run_command_as_user(
            RUN_ROOTLESSKIT,
            &["whoami"],
            None,
            &env_list,
            Some(HashMap::from([(
                "LLVM_PROFILE_FILE".to_string(),
                "/tmp/alpm-%p-%16m.profraw".to_string(),
            )])),
            user,
        )?;

        if !output.status.success() {
            return Err(format!("The test failed but should have succeeded:\n{}", output).into());
        }
        output.stdout
    };

    assert_eq!("root\n", stdout);

    Ok(())
}
