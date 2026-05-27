//! Integration tests to ensure correct [alpm-db-files] and [alpm-db-desc] files roundtrip
//! through their parsed representation, JSON serialisation and string serialisation.
//!
//! [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
//! [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html

use std::{fs::read_to_string, path::PathBuf, str::FromStr};

use alpm_db::{desc::DbDescFile, files::DbFiles};
use insta::assert_snapshot;
use rstest::rstest;
use testresult::TestResult;

/// Each `*.files` file in `tests/correct/files` is expected to parse successfully, serialize to
/// json and roundtrip to the same string.
///
/// The test works as follows:
/// 1. Parse the input as [`DbFiles`] and serialize it as JSON.
/// 2. Compared the JSON against a snapshot in `tests/correct_snapshots/files`.
/// 3. Render the parsed [`DbFiles`] back to a string and ensure that the output is identical to the
///    original input.
#[rstest]
fn correct_files(#[files("tests/correct/files/*.files")] case: PathBuf) -> TestResult {
    let input = read_to_string(&case)?;
    let name = case
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown_case");

    let files = DbFiles::from_str(&input)?;

    let json = serde_json::to_string_pretty(&files)?;
    insta::with_settings!({
        description => format!("{name} DbFiles JSON representation."),
        snapshot_path => "correct_snapshots/files",
        prepend_module_to_snapshot => false,
    }, {
        assert_snapshot!(name, json);
    });

    assert_eq!(files.to_string(), input);

    Ok(())
}

/// Each `*.desc` file in `tests/correct/desc/{v1,v2}` is expected to parse successfully, serialize
/// to json and roundtrip to the same string.
///
/// The test works as follows:
/// 1. Parse the input as [`DbDescFile`] and serialize it as JSON.
/// 2. Compare the JSON against a snapshot in `tests/correct_snapshots/desc/{v1,v2}`.
/// 3. Render the parsed [`DbDescFile`] back to a string and ensure that the output is identical to
///    the original input.
#[rstest]
fn correct_desc(#[files("tests/correct/desc/*/*.desc")] case: PathBuf) -> TestResult {
    let input = read_to_string(&case)?;
    let name = case
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown_case");
    let version = case
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("unknown_version");

    let desc = DbDescFile::from_str(&input)?;

    let json = serde_json::to_string_pretty(&desc)?;
    insta::with_settings!({
        description => format!("{version} {name} DbDescFile JSON representation."),
        snapshot_path => format!("correct_snapshots/desc/{version}"),
        prepend_module_to_snapshot => false,
    }, {
        assert_snapshot!(name, json);
    });

    assert_eq!(desc.to_string(), input);

    Ok(())
}
