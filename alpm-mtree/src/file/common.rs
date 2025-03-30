//! Common functionality for creating [ALPM-MTREE] files.
//!
//! [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use alpm_common::{MetadataFile, relative_files};
use alpm_types::{MetadataFileName, SchemaVersion, semver_version::Version};
use flate2::{Compression, GzBuilder};
use log::debug;
use which::which;

use crate::{CreationError, Error, Mtree, MtreeSchema};

/// The [bsdtar] options for different versions of [ALPM-MTREE].
///
/// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
/// [bsdtar]: https://man.archlinux.org/man/bsdtar.1
#[derive(Clone, Copy, Debug, strum::Display, strum::IntoStaticStr)]
pub enum BsdtarOptions {
    /// The [bsdtar] options for [ALPM-MTREEv1].
    ///
    /// [ALPM-MTREEv1]: https://alpm.archlinux.page/specifications/ALPM-MTREEv1.5.html
    /// [bsdtar]: https://man.archlinux.org/man/bsdtar.1
    #[strum(to_string = "!all,use-set,type,uid,gid,mode,time,size,md5,sha256,link")]
    MtreeV1,

    /// The [bsdtar] options for [ALPM-MTREEv2].
    ///
    /// [ALPM-MTREEv2]: https://alpm.archlinux.page/specifications/ALPM-MTREEv2.5.html
    /// [bsdtar]: https://man.archlinux.org/man/bsdtar.1
    #[strum(to_string = "!all,use-set,type,uid,gid,mode,time,size,sha256,link")]
    MtreeV2,
}

impl From<BsdtarOptions> for MtreeSchema {
    /// Creates an [`MtreeSchema`] from a [`BsdtarOptions`]
    fn from(value: BsdtarOptions) -> Self {
        match value {
            BsdtarOptions::MtreeV1 => MtreeSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))),
            BsdtarOptions::MtreeV2 => MtreeSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))),
        }
    }
}

/// Runs [bsdtar] in `path` with dedicated `options` and return its stdout.
///
/// Creates [ALPM-MTREE] data based on the string slice `stdin` which contains all sorted paths
/// below `path` and is passed to [bsdtar] on stdin.
///
/// # Errors
///
/// Returns an error if
///
/// - the [bsdtar] command can not be found,
/// - the [bsdtar] command can not be spawned in the background,
/// - the [bsdtar] command's stdin can not be attached to,
/// - the [bsdtar] command's stdin can not be written to,
/// - calling the [bsdtar] command is not possible,
/// - or [bsdtar] returned a non-zero status code.
///
/// [bsdtar]: https://man.archlinux.org/man/bsdtar.1
fn run_bsdtar(
    path: impl AsRef<Path>,
    options: BsdtarOptions,
    stdin: &str,
) -> Result<Vec<u8>, Error> {
    let command = "bsdtar";
    let bsdtar_command =
        which(command).map_err(|source| CreationError::CommandNotFound { command, source })?;

    let mut command = Command::new(bsdtar_command);
    command
        .current_dir(path)
        .env("LANG", "C")
        .args([
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
            options.into(),
        ])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped());
    let mut command_child = command
        .spawn()
        .map_err(|source| CreationError::CommandBackground {
            command: format!("{command:?}"),
            source,
        })?;

    // Write to stdin.
    command_child
        .stdin
        .take()
        .ok_or(CreationError::CommandAttachToStdin {
            command: format!("{command:?}"),
        })?
        .write_all(stdin.as_bytes())
        .map_err(|source| CreationError::CommandWriteToStdin {
            command: "bsdtar".to_string(),
            source,
        })?;

    let command_output =
        command_child
            .wait_with_output()
            .map_err(|source| CreationError::CommandExec {
                command: format!("{command:?}"),
                source,
            })?;
    if !command_output.status.success() {
        return Err(CreationError::CommandNonZero {
            command: format!("{command:?}"),
            exit_status: command_output.status,
            stderr: String::from_utf8_lossy(&command_output.stderr).into_owned(),
        }
        .into());
    }

    debug!(
        "bsdtar output:\n{}",
        String::from_utf8_lossy(&command_output.stdout)
    );

    Ok(command_output.stdout)
}

/// Creates an [ALPM-MTREE] file in a directory.
///
/// Validates the `mtree_data` based on `schema` and then creates the [ALPM-MTREE] file in `path`
/// using `mtree_data`.
///
/// # Errors
///
/// Returns an error if
///
/// - the `mtree_data` is not valid according to `schema`,
/// - creating the [ALPM-MTREE] file in `path` fails,
/// - or gzip compressing the [ALPM-MTREE] file fails.
///
/// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
fn create_mtree_file_in_dir(
    path: impl AsRef<Path>,
    mtree_data: &[u8],
    schema: MtreeSchema,
) -> Result<PathBuf, Error> {
    let path = path.as_ref();
    let mtree_file = path.join(MetadataFileName::Mtree.as_ref());
    debug!("Write ALPM-MTREE data to file: {mtree_file:?}");

    // Ensure that the data is correct.
    let _ = Mtree::from_reader_with_schema(mtree_data, Some(schema))?;

    // Create the target file
    let mtree = File::create(mtree_file.as_path())
        .map_err(|source| Error::IoPath(mtree_file.clone(), "creating the file", source))?;

    let mut gz = GzBuilder::new()
        // Add "Unix" as operating system to the file header.
        .operating_system(3)
        .write(mtree, Compression::best());
    gz.write_all(mtree_data).map_err(|source| {
        Error::IoPath(
            mtree_file.clone(),
            "writing data to gzip compressed file",
            source,
        )
    })?;
    gz.finish().map_err(|source| {
        Error::IoPath(mtree_file.clone(), "finishing gzip compressed file", source)
    })?;

    Ok(mtree_file)
}

/// Creates an [ALPM-MTREE] file from a package input directory.
///
/// Collects all files in `path` relative to it in a newline-delimited string.
/// Calls the [bsdtar] command, using options specific to a version of [ALPM-MTREE] to create
/// an [ALPM-MTREE] file in `path`.
/// Returns the path to the [ALPM-MTREE] file.
///
/// # Errors
///
/// Returns an error if
///
/// - calling [`relative_files`] on `path` fails,
/// - the [bsdtar] command can not be spawned in the background,
/// - the [bsdtar] command's stdin can not be attached to,
/// - the [bsdtar] command's stdin can not be written to,
/// - calling the [bsdtar] command is not possible,
/// - [bsdtar] returned a non-zero status code,
/// - creating the [ALPM-MTREE] file fails,
/// - or gzip compressing the [ALPM-MTREE] file fails.
///
/// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
/// [bsdtar]: https://man.archlinux.org/man/bsdtar.1
pub fn create_mtree_file_from_input_dir(
    path: impl AsRef<Path>,
    bsdtar_options: BsdtarOptions,
) -> Result<PathBuf, Error> {
    let path = path.as_ref();
    debug!("Create ALPM-MTREE file from input dir {path:?} with bsdtar options {bsdtar_options}");

    // Collect all files and directories in newline-delimited String.
    let collected_files: Vec<PathBuf> =
        relative_files(path, &[]).map_err(CreationError::AlpmCommon)?;
    let all_files = collected_files.iter().fold(String::new(), |mut acc, file| {
        acc.push_str(&format!("{}\n", file.to_string_lossy()));
        acc
    });
    debug!("Collected files:\n{all_files}");

    // Run bsdtar and collect the output.
    let bsdtar_output = run_bsdtar(path, bsdtar_options, &all_files)?;

    // Get the schema for the bsdtar options.
    let schema: MtreeSchema = bsdtar_options.into();

    create_mtree_file_in_dir(path, &bsdtar_output, schema)
}
