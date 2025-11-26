//! Integration tests for the `alpm-repo-files` CLI.
#![cfg(feature = "cli")]

use std::{
    fs::{File, create_dir_all, read_to_string},
    io::Write,
};

use assert_cmd::cargo_bin_cmd;
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
const ALPM_REPO_FILES_MISSING_HEADER: &str = "";
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
/// See [`ALPM_REPO_FILES_WITH_ENTRIES`].
#[fixture]
fn dir_with_entries() -> TestResult<TempDir> {
    let temp_dir = tempdir()?;
    create_dir_all(temp_dir.path().join("usr/bin/"))?;
    File::create(temp_dir.path().join("usr/bin/foo"))?;
    Ok(temp_dir)
}

/// Integration tests for `alpm-repo-files create`.
mod create {
    use super::*;

    /// Ensures that `alpm-repo-files create` successfully creates alpm-repo-files data from a dir
    /// with files.
    #[rstest]
    fn succeeds_with_default_entries_in_dir(dir_with_entries: TestResult<TempDir>) -> TestResult {
        let temp_dir = dir_with_entries?;
        let path = temp_dir.path();
        let temp_file = NamedTempFile::new()?;

        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
        cmd.args(vec!["create", path.to_string_lossy().as_ref()]);

        // Make sure the command was successful and get the output.
        let output = cmd.assert().success();
        let output = String::from_utf8_lossy(&output.get_output().stdout);

        assert_eq!(output, ALPM_REPO_FILES_WITH_ENTRIES,);

        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
        cmd.args(vec![
            "create",
            "--output",
            temp_file.path().to_string_lossy().as_ref(),
            path.to_string_lossy().as_ref(),
        ]);

        // Make sure the command was successful and get the output.
        cmd.assert().success();
        let output = read_to_string(temp_file.path())?;

        assert_eq!(output, ALPM_REPO_FILES_WITH_ENTRIES,);

        Ok(())
    }

    /// Ensures that `alpm-repo-files create` successfully creates alpm-repo-files data from an
    /// empty dir.
    #[rstest]
    fn succeeds_with_empty_dir() -> TestResult {
        let temp_dir = tempdir()?;
        let path = temp_dir.path();
        let temp_file = NamedTempFile::new()?;

        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
        cmd.args(vec!["create", path.to_string_lossy().as_ref()]);

        // Make sure the command was successful and get the output.
        let output = cmd.assert().success();
        let output = String::from_utf8_lossy(&output.get_output().stdout);

        assert_eq!(output, ALPM_REPO_FILES_EMPTY,);

        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
        cmd.args(vec![
            "create",
            "--output",
            temp_file.path().to_string_lossy().as_ref(),
            path.to_string_lossy().as_ref(),
        ]);

        // Make sure the command was successful and get the output.
        cmd.assert().success();
        let output = read_to_string(temp_file.path())?;

        assert_eq!(output, ALPM_REPO_FILES_EMPTY,);

        Ok(())
    }

    #[rstest]
    fn fails_on_regular_file() -> TestResult {
        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
        cmd.args(vec!["create", path.to_string_lossy().as_ref()]);

        // Make sure the command was unsuccessful.
        cmd.assert().failure();

        Ok(())
    }
}

/// Integration tests for `alpm-repo-files format`.
mod format {
    use super::*;

    /// Ensures that `alpm-repo-files format` creates JSON output from valid alpm-repo-files data on
    /// stdin.
    ///
    /// Checks output on stdout and output file.
    #[rstest]
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
        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
        cmd.args(vec!["format", "--format", "json"]);
        cmd.write_stdin(input);
        // Make sure the command was successful and get the output.
        let output = cmd.assert().success();
        let output = String::from_utf8_lossy(&output.get_output().stdout);
        assert_eq!(output, expected_output);

        // Write pretty JSON to stdout.
        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
        cmd.args(vec!["format", "--format", "json", "--pretty"]);
        cmd.write_stdin(input);
        // Make sure the command was successful and get the output.
        let output = cmd.assert().success();
        let output = String::from_utf8_lossy(&output.get_output().stdout);
        assert_eq!(output, expected_output_pretty);

        // Prepare output file.
        let output_file = NamedTempFile::new()?;

        // Write JSON to output file.
        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
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
        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
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

    /// Ensures that `alpm-repo-files format` creates JSON output from a valid alpm-repo-files file.
    ///
    /// Checks output on stdout and output file.
    #[rstest]
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
        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
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
        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
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
        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
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
        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
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

    /// Ensures that `alpm-repo-files format` creates alpm-repo-files output from valid
    /// alpm-repo-files data on stdin.
    ///
    /// Checks output on stdout and output file.
    #[rstest]
    #[case::alpm_repo_files_empty_to_alpm_repo_files(ALPM_REPO_FILES_EMPTY, ALPM_REPO_FILES_EMPTY)]
    #[case::alpm_repo_files_with_entries_to_alpm_repo_files(
        ALPM_REPO_FILES_WITH_ENTRIES,
        ALPM_REPO_FILES_WITH_ENTRIES
    )]
    fn succeeds_to_output_v1_with_input_from_stdin(
        #[case] input: &str,
        #[case] expected_output: &str,
    ) -> TestResult {
        // Write text to stdout.
        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
        cmd.args(vec!["format", "--format", "v1"]);
        cmd.write_stdin(input);
        // Make sure the command was successful and get the output.
        let output = cmd.assert().success();
        let output = String::from_utf8_lossy(&output.get_output().stdout);
        assert_eq!(output, expected_output);

        // Prepare output file.
        let output_file = NamedTempFile::new()?;

        // Write text to output file.
        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
        cmd.args(vec![
            "format",
            "--format",
            "v1",
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

    /// Ensures that `alpm-repo-files format` creates alpm-repo-files output from a valid
    /// alpm-repo-files file.
    ///
    /// Checks output on stdout and output file.
    #[rstest]
    #[case::alpm_repo_files_empty_to_alpm_repo_files(ALPM_REPO_FILES_EMPTY, ALPM_REPO_FILES_EMPTY)]
    #[case::alpm_repo_files_with_entries_to_alpm_repo_files(
        ALPM_REPO_FILES_WITH_ENTRIES,
        ALPM_REPO_FILES_WITH_ENTRIES
    )]
    fn succeeds_to_output_v1_with_input_from_file(
        #[case] input: &str,
        #[case] expected_output: &str,
    ) -> TestResult {
        // Prepare input file.
        let mut input_file = NamedTempFile::new()?;
        input_file.write_all(input.as_bytes())?;

        // Write text to stdout.
        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
        cmd.args(vec![
            "format",
            "--format",
            "v1",
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
        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
        cmd.args(vec![
            "format",
            "--format",
            "v1",
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

/// Integration tests for `alpm-repo-files validate`.
mod validate {
    use super::*;

    /// Ensures that `alpm-repo-files validate` successfully validates valid `alpm-repo-files` data
    /// on stdin.
    #[rstest]
    #[case(ALPM_REPO_FILES_EMPTY)]
    #[case(ALPM_REPO_FILES_WITH_ENTRIES)]
    fn succeeds_with_input_on_stdin(#[case] input: &str) -> TestResult {
        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
        cmd.args(vec!["validate"]);
        cmd.write_stdin(input);

        cmd.assert().success();

        Ok(())
    }

    /// Ensures that `alpm-repo-files validate` successfully validates valid `alpm-repo-files` data
    /// in a file.
    #[rstest]
    #[case(ALPM_REPO_FILES_EMPTY)]
    #[case(ALPM_REPO_FILES_WITH_ENTRIES)]
    fn succeeds_with_input_from_file(#[case] input: &str) -> TestResult {
        let mut temp_path = NamedTempFile::new()?;
        temp_path.write_all(input.as_bytes())?;

        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
        cmd.args(vec![
            "validate".into(),
            "--input-file".into(),
            temp_path.path().to_string_lossy().to_string(),
        ]);

        cmd.assert().success();

        Ok(())
    }

    /// Ensures that `alpm-repo-files validate` fails on invalid `alpm-repo-files` data on stdin.
    #[rstest]
    #[case(ALPM_FILES_WRONG_HEADER)]
    #[case(ALPM_FILES_INTERMITTENT_EMPTY_LINE)]
    #[case(ALPM_REPO_FILES_MISSING_HEADER)]
    fn fails_with_input_on_stdin(#[case] input: &str) -> TestResult {
        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
        cmd.args(vec!["validate"]);
        cmd.write_stdin(input);

        cmd.assert().failure();

        Ok(())
    }

    /// Ensures that `alpm-repo-files validate` fails on invalid `alpm-repo-files` data in a file.
    #[rstest]
    #[case(ALPM_FILES_WRONG_HEADER)]
    #[case(ALPM_FILES_INTERMITTENT_EMPTY_LINE)]
    #[case(ALPM_REPO_FILES_MISSING_HEADER)]
    fn fails_with_input_from_file(#[case] input: &str) -> TestResult {
        let mut temp_path = NamedTempFile::new()?;
        temp_path.write_all(input.as_bytes())?;

        let mut cmd = cargo_bin_cmd!("alpm-repo-files");
        cmd.args(vec![
            "validate".into(),
            "--input-file".into(),
            temp_path.path().to_string_lossy().to_string(),
        ]);

        cmd.assert().failure();

        Ok(())
    }
}
