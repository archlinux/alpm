use std::{fs::read_to_string, path::PathBuf};

use alpm_mtree::parse;
use insta::assert_snapshot;
use rstest::rstest;
use testresult::TestResult;

/// Happy path testing for mtree.
///
/// Take some input, parse it and compare the serialized JSON output with the snapshot.
#[rstest]
pub fn ensure_correct_syntax(
    #[files("tests/correct_syntax_inputs/*")] case: PathBuf,
) -> TestResult {
    // Read the input file and parse it.

    let input = read_to_string(&case)?;
    let result = parse(Some(&case));

    // Make sure the parsing succeeded.
    let files = match result {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("The parser failed even though it should've succeeded for input:\n{input}");
            return Err(format!("{error}").into());
        }
    };

    let name = case.file_stem().unwrap().to_str().unwrap();

    let pretty_json = serde_json::to_string_pretty(&files)?;

    // Run the tests with the input being displayed as the description.
    // This makes reviewing this whole stuff a lot easier.
    // Also remove the usual module prefix by explicitly setting the snapshot path.
    // This isn't necessary, as we're already manually sorting snapshots by test scenario.
    let input_clone = input.clone();
    insta::with_settings!({
        description => input_clone,
        snapshot_path => "correct_syntax_snapshots",
        prepend_module_to_snapshot => false,
    }, {
        assert_snapshot!(name, pretty_json);
    });

    Ok(())
}
