//! This test file contains basic tests to ensure that the dev-tools CLI behaves as expected.
use std::{fs::File, io::Write};

use assert_cmd::Command;
use testresult::TestResult;

const TEST_PKGBUILD: &str = include_str!("test_files/normal.pkgbuild");
const TEST_SRCINFO: &str = include_str!("test_files/normal.srcinfo");

mod srcinfo_compare {
    use tempfile::tempdir;

    use super::*;

    /// Run the `srcinfo compare` subcommand to compare the generated .SRCINFO file
    /// from a PKGBUILD with an existing .SRCINFO file.
    #[test]
    fn compare() -> TestResult {
        // Write the PKGBUILD to a temporary directory
        let mut tempdir = tempdir()?;
        let pkgbuild_path = tempdir.path().join("PKGBUILD");
        let mut file = File::create_new(&pkgbuild_path)?;
        file.write_all(TEST_PKGBUILD.as_bytes())?;

        // Write the .SRCINFO to a temporary directory
        let srcinfo_path = tempdir.path().join(".SRCINFO");
        let mut file = File::create_new(&srcinfo_path)?;
        file.write_all(TEST_SRCINFO.as_bytes())?;

        // Call the bridge on the that PKGBUILD file.
        let mut cmd = Command::cargo_bin("dev-scripts")?;
        cmd.args(vec![
            "compare-srcinfo",
            "--pkgbuild",
            "PKGBUILD",
            "--srcinfo",
            ".SRCINFO",
        ]);

        cmd.current_dir(tempdir.path());

        // Make sure the command was successful, which implies that the file matched!.
        let assert = cmd.assert();
        // Disable the cleanup so that users may investigate the output files.
        if let Err(err) = assert.try_success() {
            println!(
                "Output files of this test can be found at: {}",
                tempdir.path().to_string_lossy()
            );
            tempdir.disable_cleanup(true);
            // Now we may error.
            return Err(err.into());
        }

        Ok(())
    }
}
