//! Integration tests to ensure the desc parser produces meaningful errors.

use std::{fs::read_to_string, path::PathBuf, str::FromStr};

use alpm_db::desc::{DbDescFileV1, DbDescFileV2};
use insta::assert_snapshot;
use rstest::rstest;
use testresult::TestResult;

/// Each `.desc` file in `tests/parse_errors/` is expected to fail parsing.
///
/// Snapshots are saved in `parse_error_snapshots/` with the error message
/// and the file's contents as description.
#[rstest]
fn ensure_parse_errors(#[files("tests/parse_errors/*")] case: PathBuf) -> TestResult {
    let input = read_to_string(&case)?;
    let name = case
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown_case");

    // Try both V1 and V2 constructors, as syntax errors should fail on either.
    let res_v1 = DbDescFileV1::from_str(&input);
    let res_v2 = DbDescFileV2::from_str(&input);

    // ensure at least one parser fails
    if res_v1.is_ok() || res_v2.is_ok() {
        unreachable!("parser unexpectedly succeeded");
    }

    let error = res_v1
        .err()
        .map(|e| e.to_string())
        .or_else(|| res_v2.err().map(|e| e.to_string()))
        .unwrap_or_else(|| "unknown parser failure".to_string());

    let input_clone = input.clone();
    insta::with_settings!({
        description => input_clone,
        snapshot_path => "parse_errors/snapshots",
        prepend_module_to_snapshot => false,
    }, {
        assert_snapshot!(name, error);
    });

    Ok(())
}
