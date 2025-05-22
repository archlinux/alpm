use std::{fs::read_to_string, path::PathBuf};

use alpm_srcinfo::{SourceInfoV1, source_info::v1::writer::source_info_as_srcinfo};
use pretty_assertions::assert_eq;
use rstest::rstest;
use testresult::TestResult;

/// Get some correct SRCINFO files, parse it and make sure that the generated .SRCINFO file then
/// equals the originaly SRCINFO file.
#[rstest]
pub fn correct_files(#[files("tests/correct/*.srcinfo")] case: PathBuf) -> TestResult {
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
    let source_info = match source_info_result.lint() {
        Ok(source_info) => source_info,
        Err(err) => {
            return Err(
                format!("The parser produce (lint) errors that weren't expected:\n {err}").into(),
            );
        }
    };

    let output = source_info_as_srcinfo(&source_info);

    assert_eq!(
        input, output,
        "Input and generated SRCINFO output differ for file {case:?}"
    );

    Ok(())
}
