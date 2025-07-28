//! Happy-path tests for the PKGBUILD to SRCINFO conversion.

use std::path::PathBuf;

use alpm_pkgbuild::bridge::{BridgeOutput, run_bridge_script};
use alpm_srcinfo::SourceInfoV1;
use insta::assert_snapshot;
use rstest::rstest;
use testresult::TestResult;

/// Get some valid PKGBUILD files and make sure the generated SRCINFO output is correct.
///
/// This tests the whole pipeline by also inspecting and snapshotting the intermediate output
/// from the `alpm-pkgbuild-bridge`
#[rstest]
fn correct_files(#[files("tests/pkgbuild_correct/*.pkgbuild")] case: PathBuf) -> TestResult {
    let test_name = case.file_stem().unwrap().to_str().unwrap().to_string();

    // Run the bridge script on the input file.
    let raw_bridge_output = run_bridge_script(&case)?;

    // Make sure the generated bridge output matches the expected values.
    insta::with_settings!({
        description => format!("{test_name} PKGBUILD -> SRCINFO generation."),
        snapshot_path => "pkgbuild_correct_bridge_output",
        prepend_module_to_snapshot => false,
    }, {
        assert_snapshot!(format!("{test_name}_bridge"), raw_bridge_output);
    });

    // Take the raw bridge script output and parse it.
    let output = BridgeOutput::from_script_output(&raw_bridge_output)?;
    // Then convert it into a SourceInfo struct.
    let source_info: SourceInfoV1 = output.try_into()?;

    // Now create actual .SRCINFO file format output.
    let srcinfo_output = source_info.as_srcinfo();

    // Compare the generated source_info json with the expected snapshot.
    // Remove the usual module prefix by explicitly setting the snapshot path.
    // This is necessary, as we're manually sorting snapshots by test scenario.
    insta::with_settings!({
        description => format!("{test_name} PKGBUILD -> SRCINFO generation."),
        snapshot_path => "pkgbuild_correct_snapshots",
        prepend_module_to_snapshot => false,
    }, {
        assert_snapshot!(format!("{test_name}_srcinfo"), srcinfo_output  );
    });

    Ok(())
}
