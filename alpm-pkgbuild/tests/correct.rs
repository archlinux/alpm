use std::path::PathBuf;

use alpm_pkgbuild::bridge::parser::BridgeOutput;
use alpm_srcinfo::SourceInfoV1;
use insta::assert_snapshot;
use rstest::rstest;
use testresult::TestResult;

/// Get some valid PKGBUILD files and make sure the generated SRCINFO output is correct.
#[rstest]
pub fn correct_files(#[files("tests/correct/*.pkgbuild")] case: PathBuf) -> TestResult {
    // Read the input file and parse it.
    let output = BridgeOutput::from_file(&case)?;
    let source_info: SourceInfoV1 = output.try_into()?;

    let srcinfo_output = source_info.as_srcinfo();

    let test_name = case.file_stem().unwrap().to_str().unwrap().to_string();
    // Compare the generated source_info json with the expected snapshot.
    // Remove the usual module prefix by explicitly setting the snapshot path.
    // This is necessary, as we're manually sorting snapshots by test scenario.
    insta::with_settings!({
        description => format!("{test_name} PKGBUILD -> SRCINFO generation."),
        snapshot_path => "correct_snapshots",
        prepend_module_to_snapshot => false,
    }, {
        assert_snapshot!(format!("{test_name}_srcinfo"), srcinfo_output  );
    });

    Ok(())
}
