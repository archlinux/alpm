//! Integration tests for the `alpm-files` CLI.
#![cfg(feature = "cli")]

use std::{
    fs::{File, create_dir_all, read_to_string},
    io::Write,
};

use alpm_files::FilesStyle;
use assert_cmd::Command;
use rstest::{fixture, rstest};
use tempfile::{NamedTempFile, TempDir, tempdir};
use testresult::TestResult;

const ALPM_FILES_WRONG_HEADER: &str = r#"%WRONG%
usr/
usr/bin/
usr/bin/foo
"#;
const ALPM_FILES_INTERMITTENT_EMPTY_LINE: &str = r#"%FILES%
usr/
usr/bin/

usr/bin/foo
"#;
const ALPM_DB_FILES_EMPTY: &str = "";
const ALPM_DB_FILES_WITH_ENTRIES: &str = r#"%FILES%
usr/
usr/bin/
usr/bin/foo

"#;
const ALPM_REPO_FILES_EMPTY: &str = "%FILES%\n";
const ALPM_REPO_FILES_WITH_ENTRIES: &str = r#"%FILES%
usr/
usr/bin/
usr/bin/foo
"#;
const ALPM_FILES_EMPTY_JSON: &str = "[]\n";
const ALPM_FILES_EMPTY_JSON_PRETTY: &str = "[]\n";
const ALPM_FILES_WITH_ENTRIES_JSON: &str = "[\"usr/\",\"usr/bin/\",\"usr/bin/foo\"]\n";
const ALPM_FILES_WITH_ENTRIES_JSON_PRETTY: &str = r#"[
  "usr/",
  "usr/bin/",
  "usr/bin/foo"
]
"#;

/// Creates a temporary directory containing the default entries.
///
/// See [`ALPM_REPO_FILES_WITH_ENTRIES`] and [`ALPM_DB_FILES_WITH_ENTRIES`].
#[fixture]
fn dir_with_entries() -> TestResult<TempDir> {
    let temp_dir = tempdir()?;
    create_dir_all(temp_dir.path().join("usr/bin/"))?;
    File::create(temp_dir.path().join("usr/bin/foo"))?;
    Ok(temp_dir)
}

/// Integration tests for `alpm-files create`.
mod create {
    use super::*;

    /// Ensures that `alpm-files create` successfully creates alpm-files data from a dir with files.
    #[rstest]
    #[case(FilesStyle::Db)]
    #[case(FilesStyle::Repo)]
    fn succeeds_with_default_entries_in_dir(
        #[case] style: FilesStyle,
        dir_with_entries: TestResult<TempDir>,
    ) -> TestResult {
        let temp_dir = dir_with_entries?;
        let path = temp_dir.path();
        let temp_file = NamedTempFile::new()?;

        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec![
            "create",
            "--style",
            &style.to_string(),
            path.to_string_lossy().as_ref(),
        ]);

        // Make sure the command was successful and get the output.
        let output = cmd.assert().success();
        let output = String::from_utf8_lossy(&output.get_output().stdout);

        assert_eq!(
            output,
            match style {
                FilesStyle::Db => ALPM_DB_FILES_WITH_ENTRIES,
                FilesStyle::Repo => ALPM_REPO_FILES_WITH_ENTRIES,
            }
        );

        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec![
            "create",
            "--style",
            &style.to_string(),
            "--output",
            temp_file.path().to_string_lossy().as_ref(),
            path.to_string_lossy().as_ref(),
        ]);

        // Make sure the command was successful and get the output.
        cmd.assert().success();
        let output = read_to_string(temp_file.path())?;

        assert_eq!(
            output,
            match style {
                FilesStyle::Db => ALPM_DB_FILES_WITH_ENTRIES,
                FilesStyle::Repo => ALPM_REPO_FILES_WITH_ENTRIES,
            }
        );

        Ok(())
    }

    /// Ensures that `alpm-files create` successfully creates alpm-files data from an empty dir.
    #[rstest]
    #[case(FilesStyle::Db)]
    #[case(FilesStyle::Repo)]
    fn succeeds_with_empty_dir(#[case] style: FilesStyle) -> TestResult {
        let temp_dir = tempdir()?;
        let path = temp_dir.path();
        let temp_file = NamedTempFile::new()?;

        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec![
            "create",
            "--style",
            &style.to_string(),
            path.to_string_lossy().as_ref(),
        ]);

        // Make sure the command was successful and get the output.
        let output = cmd.assert().success();
        let output = String::from_utf8_lossy(&output.get_output().stdout);

        assert_eq!(
            output,
            match style {
                FilesStyle::Db => ALPM_DB_FILES_EMPTY,
                FilesStyle::Repo => ALPM_REPO_FILES_EMPTY,
            }
        );

        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec![
            "create",
            "--style",
            &style.to_string(),
            "--output",
            temp_file.path().to_string_lossy().as_ref(),
            path.to_string_lossy().as_ref(),
        ]);

        // Make sure the command was successful and get the output.
        cmd.assert().success();
        let output = read_to_string(temp_file.path())?;

        assert_eq!(
            output,
            match style {
                FilesStyle::Db => ALPM_DB_FILES_EMPTY,
                FilesStyle::Repo => ALPM_REPO_FILES_EMPTY,
            }
        );

        Ok(())
    }

    #[rstest]
    #[case(FilesStyle::Db)]
    #[case(FilesStyle::Repo)]
    fn fails_on_regular_file(#[case] style: FilesStyle) -> TestResult {
        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec![
            "create",
            "--style",
            &style.to_string(),
            path.to_string_lossy().as_ref(),
        ]);

        // Make sure the command was unsuccessful.
        cmd.assert().failure();

        Ok(())
    }
}

/// Integration tests for `alpm-files format`.
mod format {
    use super::*;

    /// Ensures that `alpm-files format` creates JSON output from valid alpm-files data on stdin.
    ///
    /// Checks output on stdout and output file.
    #[rstest]
    #[case::alpm_db_files_empty(
        ALPM_DB_FILES_EMPTY,
        ALPM_FILES_EMPTY_JSON,
        ALPM_FILES_EMPTY_JSON_PRETTY
    )]
    #[case::alpm_db_files_with_entries(
        ALPM_DB_FILES_WITH_ENTRIES,
        ALPM_FILES_WITH_ENTRIES_JSON,
        ALPM_FILES_WITH_ENTRIES_JSON_PRETTY
    )]
    #[case::alpm_repo_files_empty(
        ALPM_REPO_FILES_EMPTY,
        ALPM_FILES_EMPTY_JSON,
        ALPM_FILES_EMPTY_JSON_PRETTY
    )]
    #[case::alpm_repo_files_with_entries(
        ALPM_REPO_FILES_WITH_ENTRIES,
        ALPM_FILES_WITH_ENTRIES_JSON,
        ALPM_FILES_WITH_ENTRIES_JSON_PRETTY
    )]
    fn succeeds_to_output_json_with_input_from_stdin(
        #[case] input: &str,
        #[case] expected_output: &str,
        #[case] expected_output_pretty: &str,
    ) -> TestResult {
        // Write JSON to stdout.
        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec!["format", "--format", "json"]);
        cmd.write_stdin(input);
        // Make sure the command was successful and get the output.
        let output = cmd.assert().success();
        let output = String::from_utf8_lossy(&output.get_output().stdout);
        assert_eq!(output, expected_output);

        // Write pretty JSON to stdout.
        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec!["format", "--format", "json", "--pretty"]);
        cmd.write_stdin(input);
        // Make sure the command was successful and get the output.
        let output = cmd.assert().success();
        let output = String::from_utf8_lossy(&output.get_output().stdout);
        assert_eq!(output, expected_output_pretty);

        // Prepare output file.
        let output_file = NamedTempFile::new()?;

        // Write JSON to output file.
        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec![
            "format",
            "--format",
            "json",
            "--output",
            &output_file.path().to_string_lossy().as_ref(),
        ]);
        cmd.write_stdin(input);
        // Make sure the command was successful and get the output.
        cmd.assert().success();
        let output = read_to_string(output_file.path())?;
        assert_eq!(output, expected_output);

        // Write pretty JSON to output file.
        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec![
            "format",
            "--format",
            "json",
            "--output",
            &output_file.path().to_string_lossy().as_ref(),
            "--pretty",
        ]);
        cmd.write_stdin(input);
        // Make sure the command was successful and get the output.
        cmd.assert().success();
        let output = read_to_string(output_file.path())?;
        assert_eq!(output, expected_output_pretty);

        Ok(())
    }

    /// Ensures that `alpm-files format` creates JSON output from a valid alpm-files file.
    ///
    /// Checks output on stdout and output file.
    #[rstest]
    #[case::alpm_db_files_empty(
        ALPM_DB_FILES_EMPTY,
        ALPM_FILES_EMPTY_JSON,
        ALPM_FILES_EMPTY_JSON_PRETTY
    )]
    #[case::alpm_db_files_with_entries(
        ALPM_DB_FILES_WITH_ENTRIES,
        ALPM_FILES_WITH_ENTRIES_JSON,
        ALPM_FILES_WITH_ENTRIES_JSON_PRETTY
    )]
    #[case::alpm_repo_files_empty(
        ALPM_REPO_FILES_EMPTY,
        ALPM_FILES_EMPTY_JSON,
        ALPM_FILES_EMPTY_JSON_PRETTY
    )]
    #[case::alpm_repo_files_with_entries(
        ALPM_REPO_FILES_WITH_ENTRIES,
        ALPM_FILES_WITH_ENTRIES_JSON,
        ALPM_FILES_WITH_ENTRIES_JSON_PRETTY
    )]
    fn succeeds_to_output_json_with_input_from_file(
        #[case] input: &str,
        #[case] expected_output: &str,
        #[case] expected_output_pretty: &str,
    ) -> TestResult {
        // Prepare input file.
        let mut input_file = NamedTempFile::new()?;
        input_file.write_all(input.as_bytes())?;

        // Write JSON to stdout.
        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec![
            "format",
            "--format",
            "json",
            "--input-file",
            input_file.path().to_string_lossy().as_ref(),
        ]);
        // Make sure the command was successful and get the output.
        let output = cmd.assert().success();
        let output = String::from_utf8_lossy(&output.get_output().stdout);
        assert_eq!(output, expected_output);

        // Write pretty JSON to stdout.
        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec![
            "format",
            "--format",
            "json",
            "--pretty",
            "--input-file",
            input_file.path().to_string_lossy().as_ref(),
        ]);
        cmd.write_stdin(input);
        // Make sure the command was successful and get the output.
        let output = cmd.assert().success();
        let output = String::from_utf8_lossy(&output.get_output().stdout);
        assert_eq!(output, expected_output_pretty);

        // Prepare output file.
        let output_file = NamedTempFile::new()?;

        // Write JSON to output file.
        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec![
            "format",
            "--format",
            "json",
            "--input-file",
            input_file.path().to_string_lossy().as_ref(),
            "--output",
            output_file.path().to_string_lossy().as_ref(),
        ]);
        cmd.write_stdin(input);
        // Make sure the command was successful and get the output.
        cmd.assert().success();
        let output = read_to_string(output_file.path())?;
        assert_eq!(output, expected_output);

        // Write pretty JSON to output file.
        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec![
            "format",
            "--format",
            "json",
            "--input-file",
            input_file.path().to_string_lossy().as_ref(),
            "--output",
            output_file.path().to_string_lossy().as_ref(),
            "--pretty",
        ]);
        cmd.write_stdin(input);
        // Make sure the command was successful and get the output.
        cmd.assert().success();
        let output = read_to_string(output_file.path())?;
        assert_eq!(output, expected_output_pretty);

        Ok(())
    }

    /// Ensures that `alpm-files format` creates alpm-files output from valid alpm-files data on
    /// stdin.
    ///
    /// Checks output on stdout and output file.
    #[rstest]
    #[case::alpm_db_files_empty_to_alpm_db_files(
        ALPM_DB_FILES_EMPTY,
        FilesStyle::Db,
        ALPM_DB_FILES_EMPTY
    )]
    #[case::alpm_db_files_empty_to_alpm_repo_files(
        ALPM_DB_FILES_EMPTY,
        FilesStyle::Repo,
        ALPM_REPO_FILES_EMPTY
    )]
    #[case::alpm_db_files_with_entries_to_alpm_db_files(
        ALPM_DB_FILES_WITH_ENTRIES,
        FilesStyle::Db,
        ALPM_DB_FILES_WITH_ENTRIES
    )]
    #[case::alpm_db_files_with_entries_to_alpm_repo_files(
        ALPM_DB_FILES_WITH_ENTRIES,
        FilesStyle::Repo,
        ALPM_REPO_FILES_WITH_ENTRIES
    )]
    #[case::alpm_repo_files_empty_to_alpm_repo_files(
        ALPM_REPO_FILES_EMPTY,
        FilesStyle::Repo,
        ALPM_REPO_FILES_EMPTY
    )]
    #[case::alpm_repo_files_empty_to_alpm_db_files(
        ALPM_REPO_FILES_EMPTY,
        FilesStyle::Db,
        ALPM_DB_FILES_EMPTY
    )]
    #[case::alpm_repo_files_with_entries_to_alpm_repo_files(
        ALPM_REPO_FILES_WITH_ENTRIES,
        FilesStyle::Repo,
        ALPM_REPO_FILES_WITH_ENTRIES
    )]
    #[case::alpm_repo_files_with_entries_to_alpm_db_files(
        ALPM_REPO_FILES_WITH_ENTRIES,
        FilesStyle::Db,
        ALPM_DB_FILES_WITH_ENTRIES
    )]
    fn succeeds_to_output_v1_with_input_from_stdin(
        #[case] input: &str,
        #[case] style: FilesStyle,
        #[case] expected_output: &str,
    ) -> TestResult {
        // Write text to stdout.
        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec![
            "format",
            "--format",
            "v1",
            "--style",
            &style.to_string(),
        ]);
        cmd.write_stdin(input);
        // Make sure the command was successful and get the output.
        let output = cmd.assert().success();
        let output = String::from_utf8_lossy(&output.get_output().stdout);
        assert_eq!(output, expected_output);

        // Prepare output file.
        let output_file = NamedTempFile::new()?;

        // Write text to output file.
        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec![
            "format",
            "--format",
            "v1",
            "--style",
            &style.to_string(),
            "--output",
            &output_file.path().to_string_lossy().as_ref(),
        ]);
        cmd.write_stdin(input);
        // Make sure the command was successful and get the output.
        cmd.assert().success();
        let output = read_to_string(output_file.path())?;
        assert_eq!(output, expected_output);

        Ok(())
    }

    /// Ensures that `alpm-files format` creates alpm-files output from a valid alpm-files file.
    ///
    /// Checks output on stdout and output file.
    #[rstest]
    #[case::alpm_db_files_empty_to_alpm_db_files(
        ALPM_DB_FILES_EMPTY,
        FilesStyle::Db,
        ALPM_DB_FILES_EMPTY
    )]
    #[case::alpm_db_files_empty_to_alpm_repo_files(
        ALPM_DB_FILES_EMPTY,
        FilesStyle::Repo,
        ALPM_REPO_FILES_EMPTY
    )]
    #[case::alpm_db_files_with_entries_to_alpm_db_files(
        ALPM_DB_FILES_WITH_ENTRIES,
        FilesStyle::Db,
        ALPM_DB_FILES_WITH_ENTRIES
    )]
    #[case::alpm_db_files_with_entries_to_alpm_repo_files(
        ALPM_DB_FILES_WITH_ENTRIES,
        FilesStyle::Repo,
        ALPM_REPO_FILES_WITH_ENTRIES
    )]
    #[case::alpm_repo_files_empty_to_alpm_repo_files(
        ALPM_REPO_FILES_EMPTY,
        FilesStyle::Repo,
        ALPM_REPO_FILES_EMPTY
    )]
    #[case::alpm_repo_files_empty_to_alpm_db_files(
        ALPM_REPO_FILES_EMPTY,
        FilesStyle::Db,
        ALPM_DB_FILES_EMPTY
    )]
    #[case::alpm_repo_files_with_entries_to_alpm_repo_files(
        ALPM_REPO_FILES_WITH_ENTRIES,
        FilesStyle::Repo,
        ALPM_REPO_FILES_WITH_ENTRIES
    )]
    #[case::alpm_repo_files_with_entries_to_alpm_db_files(
        ALPM_REPO_FILES_WITH_ENTRIES,
        FilesStyle::Db,
        ALPM_DB_FILES_WITH_ENTRIES
    )]
    fn succeeds_to_output_v1_with_input_from_file(
        #[case] input: &str,
        #[case] style: FilesStyle,
        #[case] expected_output: &str,
    ) -> TestResult {
        // Prepare input file.
        let mut input_file = NamedTempFile::new()?;
        input_file.write_all(input.as_bytes())?;

        // Write text to stdout.
        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec![
            "format",
            "--format",
            "v1",
            "--style",
            &style.to_string(),
            "--input-file",
            input_file.path().to_string_lossy().as_ref(),
        ]);
        // Make sure the command was successful and get the output.
        let output = cmd.assert().success();
        let output = String::from_utf8_lossy(&output.get_output().stdout);
        assert_eq!(output, expected_output);

        // Prepare output file.
        let output_file = NamedTempFile::new()?;

        // Write text to output file.
        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec![
            "format",
            "--format",
            "v1",
            "--style",
            &style.to_string(),
            "--input-file",
            input_file.path().to_string_lossy().as_ref(),
            "--output",
            &output_file.path().to_string_lossy().as_ref(),
        ]);
        cmd.write_stdin(input);
        // Make sure the command was successful and get the output.
        cmd.assert().success();
        let output = read_to_string(output_file.path())?;
        assert_eq!(output, expected_output);

        Ok(())
    }
}

/// Integration tests for `alpm-files validate`.
mod validate {
    use super::*;

    /// Ensures that `alpm-files validate` successfully validates valid `alpm-files` data on stdin.
    #[rstest]
    #[case(ALPM_DB_FILES_EMPTY)]
    #[case(ALPM_DB_FILES_WITH_ENTRIES)]
    #[case(ALPM_REPO_FILES_EMPTY)]
    #[case(ALPM_REPO_FILES_WITH_ENTRIES)]
    fn succeeds_with_input_on_stdin(#[case] input: &str) -> TestResult {
        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec!["validate"]);
        cmd.write_stdin(input);

        cmd.assert().success();

        Ok(())
    }

    /// Ensures that `alpm-files validate` successfully validates valid `alpm-files` data in a file.
    #[rstest]
    #[case(ALPM_DB_FILES_EMPTY)]
    #[case(ALPM_DB_FILES_WITH_ENTRIES)]
    #[case(ALPM_REPO_FILES_EMPTY)]
    #[case(ALPM_REPO_FILES_WITH_ENTRIES)]
    fn succeeds_with_input_from_file(#[case] input: &str) -> TestResult {
        let mut temp_path = NamedTempFile::new()?;
        temp_path.write_all(input.as_bytes())?;

        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec![
            "validate".into(),
            "--input-file".into(),
            temp_path.path().to_string_lossy().to_string(),
        ]);

        cmd.assert().success();

        Ok(())
    }

    /// Ensures that `alpm-files validate` fails on invalid `alpm-files` data on stdin.
    #[rstest]
    #[case(ALPM_FILES_WRONG_HEADER)]
    #[case(ALPM_FILES_INTERMITTENT_EMPTY_LINE)]
    fn fails_with_input_on_stdin(#[case] input: &str) -> TestResult {
        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec!["validate"]);
        cmd.write_stdin(input);

        cmd.assert().failure();

        Ok(())
    }

    /// Ensures that `alpm-files validate` fails on invalid `alpm-files` data in a file.
    #[rstest]
    #[case(ALPM_FILES_WRONG_HEADER)]
    #[case(ALPM_FILES_INTERMITTENT_EMPTY_LINE)]
    fn fails_with_input_from_file(#[case] input: &str) -> TestResult {
        let mut temp_path = NamedTempFile::new()?;
        temp_path.write_all(input.as_bytes())?;

        let mut cmd = Command::cargo_bin("alpm-files")?;
        cmd.args(vec![
            "validate".into(),
            "--input-file".into(),
            temp_path.path().to_string_lossy().to_string(),
        ]);

        cmd.assert().failure();

        Ok(())
    }
}
