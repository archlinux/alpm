use std::{fs::read_to_string, path::PathBuf};

use alpm_srcinfo::SourceInfo;
use insta::assert_snapshot;
use rstest::rstest;
use testresult::TestResult;

/// .SRCINFO files are processed in two steps:
///
/// - Raw parsing step with winnow into intermediate representation (IR)
/// - Bring IR into proper struct representation and apply lints
///
/// This test tests parse errors during the first step.
#[rstest]
pub fn ensure_parse_errors(#[files("tests/parse_errors/*")] case: PathBuf) -> TestResult {
    // Read the input file and parse it.
    let input = read_to_string(&case)?;
    let result = SourceInfo::from_string(input.as_str());

    // Make sure there're no parse errors
    let Err(error) = result else {
        return Err("The parser succeeded even though it should've failed parsing.".into());
    };

    let name = case.file_stem().unwrap().to_str().unwrap();

    // Run the tests with the input being displayed as the description.
    // This makes reviewing this whole stuff a lot easier.
    // Also remove the usual module prefix by explicitly setting the snapshot path.
    // This isn't necessary, as we're already manually sorting snapshots by test scenario.
    let input_clone = input.clone();
    insta::with_settings!({
        description => input_clone,
        snapshot_path => "parse_error_snapshots",
        prepend_module_to_snapshot => false,
    }, {
        assert_snapshot!(name, format!("{error}"));
    });

    Ok(())
}
