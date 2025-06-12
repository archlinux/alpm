//! Integration tests for the `alpm-mtree create` command.
#![cfg(feature = "creation")]

use std::{
    fs::{File, create_dir_all},
    io::Write,
    os::unix::fs::symlink,
    path::Path,
};

use alpm_common::MetadataFile;
use alpm_mtree::{
    Mtree,
    MtreeSchema,
    create_mtree_v1_from_input_dir,
    create_mtree_v2_from_input_dir,
};
use alpm_types::SchemaVersion;
use log::debug;
use rstest::rstest;
use simplelog::{Config, TermLogger};
use tempfile::TempDir;
use testresult::TestResult;

/// Creates test files and directories below `path`.
fn create_test_files(path: impl AsRef<Path>) -> TestResult {
    let path = path.as_ref();
    // Create dummy directory structure
    create_dir_all(path.join("foo/bar/baz"))?;
    // Create dummy text file
    let mut output = File::create(path.join("foo/beh.txt"))?;
    write!(output, "test")?;
    // Create relative symlink to actual text file
    symlink("../../beh.txt", path.join("foo/bar/baz/beh.txt"))?;
    Ok(())
}

/// Initializes a global logger once.
fn init_logger() -> TestResult {
    if TermLogger::init(
        log::LevelFilter::Info,
        Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .is_err()
    {
        debug!("Not initializing another logger, as one is initialized already.");
    }

    Ok(())
}

/// Creates an .MTREE file (as ALPM-MTREEv1) and validates it.
#[rstest]
fn create_mtreev1_from_input() -> TestResult {
    init_logger()?;

    let test_dir = TempDir::new()?;

    create_test_files(test_dir.as_ref())?;

    let mtree_file = create_mtree_v1_from_input_dir(test_dir.as_ref())?;

    Mtree::from_file_with_schema(
        mtree_file,
        Some(MtreeSchema::V1(SchemaVersion::new(
            alpm_types::semver_version::Version::new(1, 0, 0),
        ))),
    )?;

    Ok(())
}

/// Creates an .MTREE file (as ALPM-MTREEv2) and validates it.
#[rstest]
fn create_mtreev2_from_input() -> TestResult {
    init_logger()?;

    let test_dir = TempDir::new()?;

    create_test_files(test_dir.as_ref())?;

    // Create .MTREE file (as ALPM-MTREEv2).
    let mtree_file = create_mtree_v2_from_input_dir(test_dir.as_ref())?;

    Mtree::from_file_with_schema(
        mtree_file,
        Some(MtreeSchema::V2(SchemaVersion::new(
            alpm_types::semver_version::Version::new(2, 0, 0),
        ))),
    )?;

    Ok(())
}
