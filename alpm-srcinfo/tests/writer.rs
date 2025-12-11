//! Integration tests for writing the SRCINFO file format.

use std::{fs::read_to_string, path::PathBuf};

use alpm_srcinfo::SourceInfoV1;
use pretty_assertions::assert_eq;
use rstest::rstest;
use testresult::TestResult;

/// Ensures that valid SRCINFO files can be created.
///
/// Reads valid SRCINFO files, parses and outputs them in SRCINFO file format.
/// Ensures that the generated data equals the input file.
#[rstest]
fn correct_writer(#[files("tests/correct/*.srcinfo")] case: PathBuf) -> TestResult {
    // Read the input file and parse it.
    let input = read_to_string(&case)?;
    let source_info_result = SourceInfoV1::from_string(&input);

    // Make sure there're no parse errors
    let source_info_result = match source_info_result {
        Ok(result) => result,
        Err(err) => {
            panic!(
                "The parser errored even though it should've succeeded the parsing step:\n{err}"
            );
        }
    };

    // Parse the source info
    let source_info = source_info_result;

    let output = source_info.as_srcinfo();

    // Compare the two files.
    assert_eq!(
        input, output,
        "Input and generated SRCINFO output differ for file {case:?}"
    );

    Ok(())
}
