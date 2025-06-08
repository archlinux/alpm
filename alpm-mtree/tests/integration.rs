//! This test file contains basic tests to ensure that the alpm-mtree CLI behaves as expected.

use std::{fs::File, io::Write};

use assert_cmd::Command;
use rstest::rstest;
use testresult::TestResult;

/// A string slice representing valid [ALPM-MTREEv2] data.
///
/// [ALPM-MTREEv2]: https://alpm.archlinux.page/specifications/ALPM-MTREEv2.5.html
pub const VALID_MTREE: &str = r#"
#mtree
/set mode=644 uid=0 gid=0 type=file
./some_file time=1700000000.0 size=1337 sha256digest=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
./some_link type=link link=some_file time=1700000000.0
./some_dir type=dir time=1700000000.0
"#;

/// Validate a valid MTREE file from stdin for both schema versions.
#[rstest]
#[case("1")]
#[case("2")]
fn validate_stdin(#[case] schema: &str) -> TestResult {
    let mut cmd = Command::cargo_bin("alpm-mtree")?;
    cmd.args(vec!["validate", "--schema", schema]);
    cmd.write_stdin(VALID_MTREE);

    cmd.assert().success();

    Ok(())
}

/// Validate a valid MTREE file for both schema versions.
#[rstest]
#[case("1")]
#[case("2")]
fn validate_file(#[case] schema: &str) -> TestResult {
    let tmp_dir = tempfile::tempdir()?;
    let file_path = tmp_dir.path().join("MTREE-TEST");
    let mut file = File::create(&file_path)?;
    file.write_all(VALID_MTREE.as_bytes())?;

    let mut cmd = Command::cargo_bin("alpm-mtree")?;
    cmd.args(vec!["validate", "--schema", schema]);
    cmd.arg(file_path.to_string_lossy().to_string());

    // Make sure the command was successful
    cmd.assert().success();

    Ok(())
}

/// Validate an invalid MTREE file from stdin for both schema versions.
#[rstest]
#[case("1")]
#[case("2")]
fn validate_wrong_stdin(#[case] schema: &str) -> TestResult {
    let mut cmd = Command::cargo_bin("alpm-mtree")?;
    cmd.args(vec!["validate", "--schema", schema]);
    cmd.write_stdin(format!(
        "{VALID_MTREE}\ngiberish doesnt_exist=1235 sha256digest=thisisatest"
    ));

    // Make sure the command failed
    cmd.assert().failure();

    Ok(())
}

/// Validate an invalid MTREE file for both schema versions.
#[rstest]
#[case("1")]
#[case("2")]
fn validate_wrong_file(#[case] schema: &str) -> TestResult {
    let tmp_dir = tempfile::tempdir()?;
    let file_path = tmp_dir.path().join("MTREE-TEST");
    let mut file = File::create(&file_path)?;
    file.write_all(VALID_MTREE.as_bytes())?;
    file.write_all(b"giberish doesnt_exist=1235 sha256digest=thisisatest")?;

    let mut cmd = Command::cargo_bin("alpm-mtree")?;
    cmd.args(vec!["validate", "--schema", schema]);
    cmd.arg(file_path.to_string_lossy().to_string());

    // Make sure the command failed
    cmd.assert().failure();

    Ok(())
}
