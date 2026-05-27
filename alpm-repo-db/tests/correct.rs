//! Integration tests to ensure correct [alpm-repo-files] files roundtrip through
//! their parsed representation, JSON serialisation and string serialisation.
//!
//! [alpm-repo-files]: https://alpm.archlinux.page/specifications/alpm-repo-files.5.html

use std::{fs::read_to_string, path::PathBuf, str::FromStr};

use alpm_repo_db::files::RepoFiles;
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
