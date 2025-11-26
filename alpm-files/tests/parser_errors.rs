//! Integration tests to ensure that the [`Files`] parser produces meaningful error messages.

use std::{fs::read_to_string, path::PathBuf};

use alpm_common::MetadataFile;
use alpm_files::files::{Files, FilesStyle, FilesStyleToString};
use alpm_types::{SchemaVersion, semver_version::Version};
use insta::{assert_snapshot, with_settings};
use rstest::rstest;
use testresult::TestResult;

/// Each `.files` file in `tests/fixtures/parser_errors/` is expected to fail parsing.
///
/// Snapshots are also saved in `tests/fixutres/parser_errors/` with the error message
/// and the file's contents as description.
#[rstest]
fn ensure_parse_errors(
    #[files("tests/fixtures/parser_errors/*.files")] file: PathBuf,
) -> TestResult {
    let input = read_to_string(&file)?;
    let result = Files::from_str_with_schema(
        &input,
        Some(alpm_files::files::FilesSchema::V1(SchemaVersion::new(
            Version::new(1, 0, 0),
        ))),
    );

    match result {
        Ok(files) => {
            panic!(
                "Should have failed but successfully created Files:\n{}",
                files.to_string(FilesStyle::Db)
            );
        }
        Err(error) => {
            let name = file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("UNKNOWN FILE");

            with_settings!({
                description => &input,
                snapshot_path => "fixtures/parser_errors",
                prepend_module_to_snapshot => false,
            }, {
                assert_snapshot!(name, error.to_string());
            })
        }
    }

    Ok(())
}
