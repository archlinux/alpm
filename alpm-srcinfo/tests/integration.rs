//! This test file contains basic tests to ensure that the alpm-srcinfo CLI behaves as expected.
//!
//! These tests are only executed when the `cli` feature flag is enabled.
#![cfg(feature = "cli")]

use std::{fs::File, io::Write};

use alpm_srcinfo::SourceInfoV1;
use assert_cmd::cargo::cargo_bin_cmd;
use tempfile::tempdir;
use testresult::TestResult;

const TEST_PKGBUILD: &str = include_str!("unit_test_files/normal.pkgbuild");

/// A string slice representing valid [SRCINFO] data.
///
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
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

mod create {
    use super::*;

    /// Run the `srcinfo format` subcommand to convert a PKGBUILD into a .SRCINFO file.
    #[test]
    fn format() -> TestResult {
        // Write the PKGBUILD to a temporary directory
        let tempdir = tempdir()?;
        let path = tempdir.path().join("PKGBUILD");
        let mut file = File::create_new(&path)?;
        file.write_all(TEST_PKGBUILD.as_bytes())?;

        // Generate the .SRCINFO file from the that PKGBUILD file.
        let mut cmd = cargo_bin_cmd!("alpm-srcinfo");
        cmd.args(vec!["create".into(), path.to_string_lossy().to_string()]);

        // Make sure the command was successful and get the output.
        let output = cmd.assert().success();
        let output = String::from_utf8_lossy(&output.get_output().stdout);

        let srcinfo = SourceInfoV1::from_string(&output)?;

        assert_eq!(srcinfo.base.name.inner(), "example");

        Ok(())
    }
}

mod validate {
    use super::*;

    /// Validate a valid SRCINFO file input from stdin
    #[test]
    fn validate_stdin() -> TestResult {
        let mut cmd = cargo_bin_cmd!("alpm-srcinfo");
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

        let mut cmd = cargo_bin_cmd!("alpm-srcinfo");
        cmd.args(vec!["validate"]);
        cmd.arg(file_path.to_string_lossy().to_string());

        // Make sure the command was successful
        cmd.assert().success();

        Ok(())
    }

    /// Validate an invalid SRCINFO file input from stdin
    #[test]
    fn validate_wrong_stdin() -> TestResult {
        let mut cmd = cargo_bin_cmd!("alpm-srcinfo");
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

        let mut cmd = cargo_bin_cmd!("alpm-srcinfo");
        cmd.args(vec!["validate"]);
        cmd.arg(file_path.to_string_lossy().to_string());

        // Make sure the command failed
        cmd.assert().failure();

        Ok(())
    }
}

mod format_packages {
    use alpm_srcinfo::MergedPackage;
    use alpm_types::SystemArchitecture;

    use super::*;

    // TODO: Write a test once we have a default value for the architecture.
    //       https://gitlab.archlinux.org/archlinux/alpm/alpm/-/issues/107

    /// Run a basic format-package test for the x86_64 architecture.
    #[test]
    fn format_package_x86_64() -> TestResult {
        let mut cmd = cargo_bin_cmd!("alpm-srcinfo");
        cmd.args(vec!["format-packages", "--architecture", "x86_64"]);
        cmd.write_stdin(VALID_SRCINFO);

        // Make sure the command was successful and get the output.
        let output = cmd.assert().success().get_output().clone();

        let merged_packages: Vec<MergedPackage> = serde_json::from_slice(&output.stdout)?;
        assert_eq!(merged_packages[0].name.to_string(), "example");
        assert_eq!(
            merged_packages[0].architecture,
            SystemArchitecture::X86_64.into()
        );

        assert_eq!(merged_packages[1].name.to_string(), "example_2");
        assert_eq!(
            merged_packages[1].architecture,
            SystemArchitecture::X86_64.into()
        );

        Ok(())
    }

    /// Run a basic format-package test and explicitly specify the aarch64 architecture.
    #[test]
    fn format_package_aarch64() -> TestResult {
        let mut cmd = cargo_bin_cmd!("alpm-srcinfo");
        cmd.args(vec!["format-packages", "--architecture", "aarch64"]);
        cmd.write_stdin(VALID_SRCINFO);

        // Make sure the command was successful and get the output.
        let output = cmd.assert().success().get_output().clone();

        let merged_packages: Vec<MergedPackage> = serde_json::from_slice(&output.stdout)?;
        assert_eq!(merged_packages[0].name.to_string(), "example_aarch64");
        assert_eq!(
            merged_packages[0].architecture,
            SystemArchitecture::Aarch64.into()
        );

        Ok(())
    }
}

mod format {
    use alpm_srcinfo::SourceInfoV1;
    use rstest::rstest;

    use super::*;

    /// Run a basic format-package test for the x86_64 architecture.
    #[rstest]
    #[case::pretty(true)]
    #[case::not_pretty(false)]
    fn format(#[case] pretty: bool) -> TestResult {
        let mut cmd = cargo_bin_cmd!("alpm-srcinfo");
        cmd.args(vec!["format", "--output-format", "json"]);
        if pretty {
            cmd.arg("--pretty");
        }
        cmd.write_stdin(VALID_SRCINFO);

        // Make sure the command was successful and get the output.
        let output = cmd.assert().success().get_output().clone();

        let source_info: SourceInfoV1 = serde_json::from_slice(&output.stdout)?;
        assert_eq!(source_info.base.name.to_string(), "example");
        assert_eq!(source_info.packages[1].name.to_string(), "example_2");

        Ok(())
    }
}
