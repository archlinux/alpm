//! This test file contains basic tests to ensure that the alpm-srcinfo CLI behaves as expected.
use std::{fs::File, io::Write};

use assert_cmd::Command;
use testresult::TestResult;

pub const VALID_SRCINFO: &str = r#"
pkgbase = example
    pkgver = 1.0.0
    epoch = 1
    pkgrel = 1
    pkgdesc = A project that does something
    url = https://example.org/
    arch = x86_64

pkgname = example

pkgname = example_2

pkgname = example_aarch64
    arch = aarch64
"#;

mod validate {
    use super::*;

    /// Validate a valid SRCINFO file input from stdin
    #[test]
    fn validate_stdin() -> TestResult {
        let mut cmd = Command::cargo_bin("alpm-srcinfo")?;
        cmd.args(vec!["validate"]);
        cmd.write_stdin(VALID_SRCINFO);

        // Make sure the command was successful
        cmd.assert().success();

        Ok(())
    }

    /// Validate a valid SRCINFO file
    #[test]
    fn validate_file() -> TestResult {
        let tmp_dir = tempfile::tempdir()?;
        let file_path = tmp_dir.path().join("SRCINFO-TEST");
        let mut file = File::create(&file_path)?;
        file.write_all(VALID_SRCINFO.as_bytes())?;

        let mut cmd = Command::cargo_bin("alpm-srcinfo")?;
        cmd.args(vec!["validate"]);
        cmd.arg(file_path.to_string_lossy().to_string());

        // Make sure the command was successful
        cmd.assert().success();

        Ok(())
    }

    /// Validate an invalid SRCINFO file input from stdin
    #[test]
    fn validate_wrong_stdin() -> TestResult {
        let mut cmd = Command::cargo_bin("alpm-srcinfo")?;
        cmd.args(vec!["validate"]);
        cmd.write_stdin(format!("{VALID_SRCINFO}\ngiberish_key=this is a test"));

        // Make sure the command failed
        cmd.assert().failure();

        Ok(())
    }

    /// Validate an invalid SRCINFO
    #[test]
    fn validate_wrong_file() -> TestResult {
        let tmp_dir = tempfile::tempdir()?;
        let file_path = tmp_dir.path().join("SRCINFO-TEST");
        let mut file = File::create(&file_path)?;
        file.write_all(VALID_SRCINFO.as_bytes())?;
        file.write_all(b"\ngiberish_key=this is a test")?;

        let mut cmd = Command::cargo_bin("alpm-srcinfo")?;
        cmd.args(vec!["validate"]);
        cmd.arg(file_path.to_string_lossy().to_string());

        // Make sure the command failed
        cmd.assert().failure();

        Ok(())
    }
}

mod format_packages {
    use alpm_srcinfo::MergedPackage;
    use alpm_types::Architecture;

    use super::*;

    // TODO: Write a test once we have a default value for the architecture.
    //       https://gitlab.archlinux.org/archlinux/alpm/alpm/-/issues/107

    /// Run a basic format-package test for the x86_64 architecture.
    #[test]
    fn format_package_x86_64() -> TestResult {
        let mut cmd = Command::cargo_bin("alpm-srcinfo")?;
        cmd.args(vec!["format-packages", "--architecture", "x86_64"]);
        cmd.write_stdin(VALID_SRCINFO);

        // Make sure the command was successful and get the output.
        let output = cmd.assert().success().get_output().clone();

        let merged_packages: Vec<MergedPackage> = serde_json::from_slice(&output.stdout)?;
        assert_eq!(merged_packages[0].name.to_string(), "example");
        assert_eq!(merged_packages[0].architecture, Architecture::X86_64);

        assert_eq!(merged_packages[1].name.to_string(), "example_2");
        assert_eq!(merged_packages[1].architecture, Architecture::X86_64);

        Ok(())
    }

    /// Run a basic format-package test and explicitly specify the aarch64 architecture.
    #[test]
    fn format_package_aarch64() -> TestResult {
        let mut cmd = Command::cargo_bin("alpm-srcinfo")?;
        cmd.args(vec!["format-packages", "--architecture", "aarch64"]);
        cmd.write_stdin(VALID_SRCINFO);

        // Make sure the command was successful and get the output.
        let output = cmd.assert().success().get_output().clone();

        let merged_packages: Vec<MergedPackage> = serde_json::from_slice(&output.stdout)?;
        assert_eq!(merged_packages[0].name.to_string(), "example_aarch64");
        assert_eq!(merged_packages[0].architecture, Architecture::Aarch64);

        Ok(())
    }
}
