//! Integration tests to ensure the desc parser produces meaningful errors.

use std::{fs::read_to_string, path::PathBuf, str::FromStr};

use alpm_common::MetadataFile;
use alpm_db::{
    desc::{DbDescFileV1, DbDescFileV2},
    files::DbFiles,
};
use alpm_types::{SchemaVersion, semver_version::Version};
use insta::assert_snapshot;
use rstest::rstest;
use testresult::TestResult;

/// Each `.desc` file in `tests/parse_errors/desc` is expected to fail parsing.
///
/// Snapshots are saved in same directory with the error message
/// and the file's contents as description.
#[rstest]
fn ensure_desc_parse_errors(#[files("tests/parse_errors/desc/*")] case: PathBuf) -> TestResult {
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
        snapshot_path => "parse_errors/desc/snapshots",
        prepend_module_to_snapshot => false,
    }, {
        assert_snapshot!(name, error);
    });

    Ok(())
}

/// Each `.files` file in `tests/parse_errors/files` is expected to fail parsing.
///
/// Snapshots are saved in same directory with the error message
/// and the file's contents as description.
#[rstest]
fn ensure_files_parse_errors(
    #[files("tests/parse_errors/files/*.files")] file: PathBuf,
) -> TestResult {
    let input = read_to_string(&file)?;
    let result = DbFiles::from_str_with_schema(
        &input,
        Some(alpm_db::files::DbFilesSchema::V1(SchemaVersion::new(
            Version::new(1, 0, 0),
        ))),
    );

    match result {
        Ok(files) => panic!("Should have failed but created DbFiles:\n{files}"),
        Err(error) => {
            let name = file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("UNKNOWN FILE");

            insta::with_settings!({
                description => &input,
                snapshot_path => "parse_errors/files/snapshots",
                prepend_module_to_snapshot => false,
            }, {
                assert_snapshot!(name, error.to_string());
            })
        }
    }

    Ok(())
}
