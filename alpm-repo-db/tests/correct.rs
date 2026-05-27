//! Integration tests to ensure correct [alpm-repo-files] and [alpm-repo-desc] files roundtrip
//! through their parsed representation, JSON serialisation and string serialisation.
//!
//! [alpm-repo-files]: https://alpm.archlinux.page/specifications/alpm-repo-files.5.html
//! [alpm-repo-desc]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html

use std::{fs::read_to_string, path::PathBuf, str::FromStr};

use alpm_repo_db::{desc::RepoDescFile, files::RepoFiles};
use insta::assert_snapshot;
use rstest::rstest;
use testresult::TestResult;

/// Each `.files` file in `tests/correct/files` is expected to parse successfully, serialize to json
/// and roundtrip to the same string.
///
/// The test works as follows:
/// 1. Parse the input as [`RepoFiles`] and serialize it as JSON.
/// 2. Compare the JSON against a snapshot in `tests/correct_snapshots/files`.
/// 3. Render the parsed [`RepoFiles`] back to a string and ensure that the output is identical to
///    the original input.
#[rstest]
fn correct_files(#[files("tests/correct/files/*.files")] case: PathBuf) -> TestResult {
    let input = read_to_string(&case)?;
    let name = case
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown_case");

    let files = RepoFiles::from_str(&input)?;

    let json = serde_json::to_string_pretty(&files)?;
    insta::with_settings!({
        description => format!("{name} RepoFiles JSON representation."),
        snapshot_path => "correct_snapshots/files",
        prepend_module_to_snapshot => false,
    }, {
        assert_snapshot!(name, json);
    });

    assert_eq!(files.to_string(), input);

    Ok(())
}

/// Each `.desc` file in `tests/correct/desc/{v1,v2}` is expected to parse successfully, serialize
/// to json and roundtrip to the same string.
///
/// The test works as follows:
/// 1. Parse the input as [`RepoDescFile`] and serialize it as JSON.
/// 2. Compare the JSON against a snapshot in `tests/correct_snapshots/desc/{v1,v2}`.
/// 3. Render the parsed [`RepoDescFile`] back to a string and ensure that the output is identical
///    to the original input.
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

    let desc = RepoDescFile::from_str(&input)?;

    let json = serde_json::to_string_pretty(&desc)?;
    insta::with_settings!({
        description => format!("{version} {name} RepoDescFile JSON representation."),
        snapshot_path => format!("correct_snapshots/desc/{version}"),
        prepend_module_to_snapshot => false,
    }, {
        assert_snapshot!(name, json);
    });

    assert_eq!(desc.to_string(), input);

    Ok(())
}
