use std::{fs::read_to_string, path::PathBuf};

use alpm_mtree::parser::mtree;
use insta::assert_snapshot;
use rstest::rstest;
use testresult::TestResult;
use winnow::Parser;

/// Mtree files are processed in two steps:
///
/// 1. Parsing the file. Make sure that the syntax is correct and return the syntax representation.
/// 2. Interpret the syntax. MTree files are stateful via the `/set` and `/unset` commands.
///
/// This test covers errors in the parsing step.
///
/// Test that faulty mtree files in the `parse_error_inputs` folder result in the expected error
/// messages. The error messages are stored in the `parse_error_snapshots` directory under the
/// respective name of the input file.
#[rstest]
pub fn ensure_errors_v1(#[files("tests/parse_error_inputs/*")] case: PathBuf) -> TestResult {
    // Read the input file and parse it.
    let input = read_to_string(&case)?;
    let result = mtree.parse(input.as_str());

    let Err(error) = result else {
        return Err(format!(
            "The parser succeeded even though it should've failed for input:\n{input}"
        )
        .into());
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
        assert_snapshot!(name, error);
    });

    Ok(())
}
