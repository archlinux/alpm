//! Error test cases for the BridgeOutput to SRCINFO conversion.

use std::path::PathBuf;

use alpm_pkgbuild::bridge::{BridgeOutput, run_bridge_script};
use alpm_srcinfo::{SourceInfoV1, pkgbuild_bridge::error::BridgeError};
use insta::assert_snapshot;
use rstest::rstest;
use testresult::TestResult;

/// Make sure the correct errors are thrown on invalid PKGBUILD files that generate faulty
/// BridgeOutput.
///
/// This test does snapshot testing of the formatted errors for each invalid PKGBUILD.
#[rstest]
fn invalid_files(#[files("tests/pkgbuild_invalid/*.pkgbuild")] case: PathBuf) -> TestResult {
    let test_name = case.file_stem().unwrap().to_str().unwrap().to_string();

    // Run the bridge script on the input file.
    let raw_bridge_output = run_bridge_script(&case)?;

    // Take the raw bridge script output and parse it.
    let output = BridgeOutput::from_script_output(&raw_bridge_output)?;
    // Then convert it into a SourceInfo struct.
    let result: Result<SourceInfoV1, BridgeError> = output.try_into();

    let Err(err) = result else {
        panic!("PKGBUILD to SRCINFO conversion worked, although it should fail.");
    };

    // Compare the generated source_info json with the expected snapshot.
    // Remove the usual module prefix by explicitly setting the snapshot path.
    // This is necessary, as we're manually sorting snapshots by test scenario.
    insta::with_settings!({
        description => format!("{test_name} PKGBUILD -> SRCINFO generation."),
        snapshot_path => "pkgbuild_invalid_snapshots",
        prepend_module_to_snapshot => false,
    }, {
        assert_snapshot!(format!("{test_name}_srcinfo"), err.to_string());
    });

    Ok(())
}
