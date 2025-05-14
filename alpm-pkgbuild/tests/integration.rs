//! This test file contains basic tests to ensure that the alpm-pkgbuild CLI behaves as expected.
use std::{fs::File, io::Write};

use assert_cmd::Command;
use testresult::TestResult;

const TEST_PKGBUILD: &str = include_str!("test_files/normal.pkgbuild");

mod srcinfo_run_bridge {
    use tempfile::tempdir;

    use super::*;

    /// Execute the `run_bridge` subcommand, which is used to generate the intermediate format via
    /// the bridge shell script.
    #[test]
    fn run_bridge() -> TestResult {
        // Write the PKGBUILD to a temporary directory
        let tempdir = tempdir()?;
        let path = tempdir.path().join("PKGBUILD");
        let mut file = File::create_new(&path)?;
        file.write_all(TEST_PKGBUILD.as_bytes())?;

        // Call the bridge on the that PKGBUILD file.
        let mut cmd = Command::cargo_bin("alpm-pkgbuild")?;
        cmd.args(vec![
            "srcinfo".into(),
            "run-bridge".into(),
            path.to_string_lossy().to_string(),
        ]);

        // Make sure the command was successful and get the output.
        let output = cmd.assert().success();
        let output = String::from_utf8(output.get_output().stdout.to_vec())?;
        println!("Output:\n{output}");

        assert!(
            output.contains(r#"VAR GLOBAL ARRAY arch "x86_64" "aarch64""#),
            "Got unexpected output:\n{output}"
        );

        Ok(())
    }
}

mod srcinfo_format {
    use alpm_srcinfo::SourceInfoV1;
    use tempfile::tempdir;

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
        let mut cmd = Command::cargo_bin("alpm-pkgbuild")?;
        cmd.args(vec![
            "srcinfo".into(),
            "format".into(),
            path.to_string_lossy().to_string(),
        ]);

        // Make sure the command was successful and get the output.
        let output = cmd.assert().success();
        let output = String::from_utf8_lossy(&output.get_output().stdout);

        let srcinfo = SourceInfoV1::from_string(&output)?.source_info()?;

        assert_eq!(srcinfo.base.name.inner(), "example");

        Ok(())
    }
}
