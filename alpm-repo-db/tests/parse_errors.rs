//! Integration tests to ensure that the [`RepoFiles`] parser produces meaningful error messages.

use std::{fs::read_to_string, path::PathBuf};

use alpm_common::MetadataFile;
use alpm_repo_db::files::RepoFiles;
use alpm_types::{SchemaVersion, semver_version::Version};
use insta::{assert_snapshot, with_settings};
use rstest::rstest;
use testresult::TestResult;

/// Each `.files` file in `tests/parse_errors/files` is expected to fail parsing.
///
/// Snapshots are saved in `tests/parse_errors/files/snapshots` with the error message
/// and the file's contents as description.
#[rstest]
fn ensure_parse_errors(#[files("tests/parse_errors/files/*.files")] file: PathBuf) -> TestResult {
    let input = read_to_string(&file)?;
    let result = RepoFiles::from_str_with_schema(
        &input,
        Some(alpm_repo_db::files::RepoFilesSchema::V1(
            SchemaVersion::new(Version::new(1, 0, 0)),
        )),
    );

    match result {
        Ok(files) => {
            return Err(
                format!("Should have failed but successfully created RepoFiles:\n{files}").into(),
            );
        }
        Err(error) => {
            let name = file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("UNKNOWN FILE");

            with_settings!({
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
