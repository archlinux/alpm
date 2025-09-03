//! Integration tests for the alpm-lint CLI.
//!
//! End-to-end test the CLI and make sure that all commands (and their options) actually work.

use std::{fs::File, io::Write};

use alpm_types::Architecture;
use assert_cmd::Command;
use tempfile::{TempDir, tempdir};
use testresult::TestResult;

mod fixtures;

use fixtures::default_source_info_v1;

/// Creates a `.SRCINFO` file with duplicate architectures in a temporary directory.
/// Returns the temporary directory handle.
fn setup_faulty_srcinfo() -> TestResult<TempDir> {
    // Create a temporary directory and write a faulty .SRCINFO file
    let tempdir = tempdir()?;
    let srcinfo_path = tempdir.path().join(".SRCINFO");

    let mut source_info = default_source_info_v1()?;
    source_info.base.architectures = vec![Architecture::X86_64, Architecture::X86_64];
    let srcinfo_content = source_info.as_srcinfo();

    let mut file = File::create(&srcinfo_path)?;
    file.write_all(srcinfo_content.as_bytes())?;

    Ok(tempdir)
}

/// Creates a valid `.SRCINFO` file in a temporary directory.
/// Returns the temporary directory handle.
fn setup_valid_srcinfo() -> TestResult<TempDir> {
    let tempdir = tempdir()?;
    let srcinfo_path = tempdir.path().join(".SRCINFO");

    let source_info = default_source_info_v1()?;
    let srcinfo_content = source_info.as_srcinfo();
    let mut file = File::create(&srcinfo_path)?;
    file.write_all(srcinfo_content.as_bytes())?;

    Ok(tempdir)
}

mod check {
    use super::*;

    /// Test the check command with a faulty .SRCINFO file that has duplicate architectures.
    ///
    /// This should trigger the duplicate_architecture lint rule and exit with code 1.
    #[test]
    fn check() -> TestResult {
        // Creates a temporary directory and writes a faulty .SRCINFO file.
        let tempdir = setup_faulty_srcinfo()?;

        // Run the check command on the faulty .SRCINFO file
        let mut cmd = Command::cargo_bin("alpm-lint")?;
        cmd.args(vec![
            "check",
            &tempdir.path().join(".SRCINFO").to_string_lossy(),
        ]);
        cmd.assert().failure();

        Ok(())
    }

    /// Test the check command with a valid .SRCINFO file.
    ///
    /// This should find no lints and exit with code 0.
    #[test]
    fn check_valid_srcinfo() -> TestResult {
        // Create a temporary directory and write a faulty .SRCINFO file.
        let tempdir = setup_valid_srcinfo()?;

        // Run the check command on the valid .SRCINFO file
        let mut cmd = Command::cargo_bin("alpm-lint")?;
        cmd.args(vec![
            "check",
            &tempdir.path().join(".SRCINFO").to_string_lossy(),
        ]);
        cmd.assert().success();

        Ok(())
    }

    /// Test the check command with JSON output format.
    #[test]
    fn check_json_output() -> TestResult {
        // Create a temporary directory and write a faulty .SRCINFO file.
        let tempdir = setup_faulty_srcinfo()?;

        // Run the check command with JSON output format
        let mut cmd = Command::cargo_bin("alpm-lint")?;
        cmd.args(vec![
            "check",
            "--format",
            "json",
            &tempdir.path().join(".SRCINFO").to_string_lossy(),
        ]);

        let output = cmd.assert().failure().get_output().clone();
        let output_str = String::from_utf8_lossy(&output.stdout);

        // The output should contain valid JSON
        let _: serde_json::Value = serde_json::from_str(&output_str)?;

        Ok(())
    }

    /// Test the check command with TOML output format.
    #[test]
    fn check_toml_output() -> TestResult {
        // Create a temporary directory and write a faulty .SRCINFO file.
        let tempdir = setup_faulty_srcinfo()?;

        // Run the check command with TOML output format
        let mut cmd = Command::cargo_bin("alpm-lint")?;
        cmd.args(vec![
            "check",
            "--format",
            "toml",
            &tempdir.path().join(".SRCINFO").to_string_lossy(),
        ]);

        let output = cmd.assert().failure().get_output().clone();
        let output_str = String::from_utf8_lossy(&output.stdout);

        // The output should contain valid TOML
        let _: toml::Value = toml::from_str(&output_str)?;

        Ok(())
    }

    /// Test the check command with pretty output.
    #[test]
    fn check_pretty_output() -> TestResult {
        // Create a temporary directory and write a faulty .SRCINFO file.
        let tempdir = setup_faulty_srcinfo()?;

        // Run the check command with pretty-printed JSON output
        let mut cmd = Command::cargo_bin("alpm-lint")?;
        cmd.args(vec![
            "check",
            "--format",
            "json",
            "--pretty",
            &tempdir.path().join(".SRCINFO").to_string_lossy(),
        ]);
        cmd.assert().failure();

        Ok(())
    }
}

mod rules {
    use super::*;

    /// Test the rules command with JSON output.
    #[test]
    fn json_output() -> TestResult {
        let mut cmd = Command::cargo_bin("alpm-lint")?;
        cmd.args(vec!["rules", "--format", "json"]);

        let output = cmd.assert().success().get_output().clone();
        let output_str = String::from_utf8_lossy(&output.stdout);

        // The output should contain valid JSON
        let _: serde_json::Value = serde_json::from_str(&output_str)?;

        Ok(())
    }

    /// Test the rules command with JSON output.
    #[test]
    fn toml_output() -> TestResult {
        let mut cmd = Command::cargo_bin("alpm-lint")?;
        cmd.args(vec!["rules", "--format", "toml"]);

        let output = cmd.assert().success().get_output().clone();
        let output_str = String::from_utf8_lossy(&output.stdout);

        // The output should contain valid TOML
        let _: toml::Value = toml::from_str(&output_str)?;

        Ok(())
    }

    /// Test the rules command with output to a file.
    #[test]
    fn file_output() -> TestResult {
        let tempdir = tempdir()?;
        let output_path = tempdir.path().join("rules.json");

        let mut cmd = Command::cargo_bin("alpm-lint")?;
        cmd.args(vec![
            "rules",
            "--format",
            "json",
            "--output",
            &output_path.to_string_lossy(),
        ]);
        cmd.assert().success();

        // Make sure the file was created and contains valid JSON.
        let content = std::fs::read_to_string(&output_path)?;
        let _: serde_json::Value = serde_json::from_str(&content)?;

        Ok(())
    }
}

mod options {
    use std::fs::read_to_string;

    use super::*;

    /// Test the options command with JSON output.
    #[test]
    fn json_output() -> TestResult {
        let mut cmd = Command::cargo_bin("alpm-lint")?;
        cmd.args(vec!["options", "--format", "json"]);

        let output = cmd.assert().success().get_output().clone();
        let output_str = String::from_utf8_lossy(&output.stdout);

        // The output should contain valid JSON
        let _: serde_json::Value = serde_json::from_str(&output_str)?;

        Ok(())
    }

    /// Test the options command with TOML output.
    #[test]
    fn toml_output() -> TestResult {
        let mut cmd = Command::cargo_bin("alpm-lint")?;
        cmd.args(vec!["options", "--format", "toml"]);

        // Make sure the command was successful and get the output
        let output = cmd.assert().success().get_output().clone();
        let output_str = String::from_utf8_lossy(&output.stdout);

        // The output should contain valid TOML
        let _: toml::Value = toml::from_str(&output_str)?;

        Ok(())
    }

    /// Test the options command with output to a file.
    #[test]
    fn file_output() -> TestResult {
        let tempdir = tempdir()?;
        let output_path = tempdir.path().join("options.json");

        let mut cmd = Command::cargo_bin("alpm-lint")?;
        cmd.args(vec![
            "options",
            "--format",
            "json",
            "--output",
            &output_path.to_string_lossy(),
        ]);
        cmd.assert().success();

        // Make sure the file was created and contains valid JSON.
        let content = read_to_string(&output_path)?;
        let _: serde_json::Value = serde_json::from_str(&content)?;

        Ok(())
    }
}

mod meta {
    use super::*;

    /// Test the meta command with JSON output.
    #[test]
    fn json_output() -> TestResult {
        let mut cmd = Command::cargo_bin("alpm-lint")?;
        cmd.args(vec!["meta", "--format", "json"]);

        let output = cmd.assert().success().get_output().clone();
        let output_str = String::from_utf8_lossy(&output.stdout);

        // The output should contain valid JSON
        let _: serde_json::Value = serde_json::from_str(&output_str)?;

        Ok(())
    }

    /// Test the meta command with TOML output.
    #[test]
    fn toml_output() -> TestResult {
        let mut cmd = Command::cargo_bin("alpm-lint")?;
        cmd.args(vec!["meta", "--format", "toml"]);

        let output = cmd.assert().success().get_output().clone();
        let output_str = String::from_utf8_lossy(&output.stdout);

        // The output should contain valid TOML
        let _: toml::Value = toml::from_str(&output_str)?;

        Ok(())
    }

    /// Test the meta command with output to a file.
    #[test]
    fn file_output() -> TestResult {
        let tempdir = tempdir()?;
        let output_path = tempdir.path().join("meta.json");

        let mut cmd = Command::cargo_bin("alpm-lint")?;
        cmd.args(vec![
            "meta",
            "--format",
            "json",
            "--output",
            &output_path.to_string_lossy(),
        ]);
        cmd.assert().success();

        // Make sure the file was created and contains valid JSON.
        let content = std::fs::read_to_string(&output_path)?;
        let _: serde_json::Value = serde_json::from_str(&content)?;

        Ok(())
    }
}
