//! Integration tests to check lints.

// NOTE: With rstest > 0.25.0 this can be removed!
#![allow(missing_docs)]

use std::{fs::read_to_string, path::PathBuf};

use alpm_srcinfo::{Error, SourceInfoV1};
use insta::assert_snapshot;
use rstest::rstest;
use testresult::TestResult;

/// .SRCINFO files are processed in two steps:
///
/// - Raw parsing step with winnow into intermediate representation (IR)
/// - Bring IR into proper struct representation and apply lints
///
/// This test tests linting issues and error messages during the second step.
#[rstest]
pub fn ensure_lints(#[files("tests/lints/*")] case: PathBuf) -> TestResult {
    // Read the input file and parse it.

    let input = read_to_string(&case)?;
    let source_info_result = SourceInfoV1::from_string(input.as_str());

    // Make sure there're no parse errors
    let source_info_result = match source_info_result {
        Ok(result) => result,
        Err(err) => {
            return Err(format!(
                "The parser errored even though it should've succeeded the parsing step:\n{err}"
            )
            .into());
        }
    };

    // Ensure that there're lint errors
    let Some(lint_errors) = source_info_result.errors() else {
        return Err(
            "The parser didn't produce any lint errors, even though there should be some".into(),
        );
    };

    let error_msg = Error::SourceInfoErrors(lint_errors.clone()).to_string();

    let name = case.file_stem().unwrap().to_str().unwrap();

    // Run the tests with the input being displayed as the description.
    // This makes reviewing this whole stuff a lot easier.
    // Also remove the usual module prefix by explicitly setting the snapshot path.
    // This isn't necessary, as we're already manually sorting snapshots by test scenario.
    let input_clone = input.clone();
    insta::with_settings!({
        description => input_clone,
        snapshot_path => "lint_snapshots",
        prepend_module_to_snapshot => false,
    }, {
        assert_snapshot!(name, error_msg);
    });

    Ok(())
}
