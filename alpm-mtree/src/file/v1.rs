//! Mtree v1 file creation.

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use alpm_common::{MetadataFileName, relative_files};
use flate2::{Compression, GzBuilder};

/// The ALPM-MTREEv1 specific `bsdtar` options.
const BSDTAR_OPTIONS: &str = "!all,use-set,type,uid,gid,mode,time,size,md5,sha256,link";

/// An ALPM-MTREEv1 file.
pub struct MtreeFileV1(PathBuf);

impl MtreeFileV1 {
    pub fn to_path_buf(&self) -> PathBuf {
        self.0.clone()
    }
}

impl AsRef<Path> for MtreeFileV1 {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl TryFrom<PathBuf> for MtreeFileV1 {
    type Error = crate::Error;
    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        Self::try_from(value.as_path())
    }
}

impl TryFrom<&Path> for MtreeFileV1 {
    type Error = crate::Error;
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let collected_files: Vec<PathBuf> = relative_files(value, &[])?;

        let collected_files_string: Vec<String> = collected_files
            .iter()
            .map(|file| file.to_string_lossy().to_string())
            .collect();
        let all_files = collected_files_string.join("\n");

        let mut command = Command::new("bsdtar");
        command.current_dir(value).env("LANG", "C").args([
            "--create",
            "--exclude",
            MetadataFileName::Mtree.as_ref(),
            "--files-from",
            "-",
            "--file",
            "-",
            "--format=mtree",
            "--no-recursion",
            "--options",
            BSDTAR_OPTIONS,
        ]);
        let mut command_child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|source| crate::Error::CommandBackground {
                command: format!("{command:?}"),
                source,
            })?;

        let Some(mut stdin) = command_child.stdin.take() else {
            return Err(crate::Error::CommandAttachToStdin {
                command: format!("{command:?}"),
            })?;
        };
        let handle = std::thread::spawn(move || {
            stdin.write_all(all_files.as_bytes()).map_err(|source| {
                crate::Error::CommandWriteToStdin {
                    command: "bsdtar".to_string(),
                    source,
                }
            })
        });
        let _handle_result = handle.join().map_err(|source| crate::Error::Thread {
            context: format!("running command {command:?}: {source:?}"),
        })?;

        let command_output =
            command_child
                .wait_with_output()
                .map_err(|source| crate::Error::CommandExec {
                    command: format!("{command:?}"),
                    source,
                })?;

        if !command_output.status.success() {
            return Err(crate::Error::CommandNonZero {
                command: format!("{command:?}"),
                exit_status: command_output.status,
                stderr: String::from_utf8_lossy(&command_output.stderr).into_owned(),
            });
        }

        let mtree_file = value.join(MetadataFileName::Mtree.as_ref());
        let mtree = File::create(mtree_file.as_path()).map_err(|source| {
            crate::Error::IoPath(mtree_file.clone(), "creating the file", source)
        })?;
        let mut gz = GzBuilder::new()
            .filename(MetadataFileName::Mtree.as_ref())
            .write(mtree, Compression::best());
        gz.write_all(&command_output.stdout).map_err(|source| {
            crate::Error::IoPath(
                mtree_file.clone(),
                "writing data to gzip compressed file",
                source,
            )
        })?;
        gz.finish().map_err(|source| {
            crate::Error::IoPath(mtree_file.clone(), "finishing gzip compressed file", source)
        })?;

        Ok(Self(mtree_file))
    }
}
