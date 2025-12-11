//! Interpreter error integration tests for the `alpm-mtree` CLI.

#![cfg(feature = "cli")]

use std::{fs::read_to_string, path::PathBuf};

use alpm_types::{SchemaVersion, semver_version::Version};
use insta::assert_snapshot;
use rstest::rstest;
use testresult::TestResult;

/// Mtree files are processed in two steps:
///
/// 1. Parsing the file. Make sure that the syntax is correct and return the syntax representation.
/// 2. Interpret the syntax. MTree files are stateful via the `/set` and `/unset` commands.
///
/// This test covers errors in the interpretation step.
///
/// Test that mtree files with missing or superfluous properties result in the expected error
/// messages. The input files stored in the `interpreter_error_input` directory.
/// The snapshot files are stored in the `interpreter_error_snapshots` directory under the
/// respective filenames.
#[rstest]
fn ensure_errors_v1(#[files("tests/interpreter_error_inputs/*")] case: PathBuf) -> TestResult {
    // Read the input file and parse it.

    use alpm_common::MetadataFile;
    use alpm_mtree::Mtree;
    let input = read_to_string(&case)?;
    let result = Mtree::from_file_with_schema(
        &case,
        Some(alpm_mtree::MtreeSchema::V1(SchemaVersion::new(
            Version::new(1, 0, 0),
        ))),
    );

    let Err(error) = result else {
        panic!("The interpreter succeeded even though it should've failed for input:\n{input}");
    };

    let name = case.file_stem().unwrap().to_str().unwrap();

    // Run the tests with the input being displayed as the description.
    // This makes reviewing this whole stuff a lot easier.
    // Also remove the usual module prefix, as we're already manually sorting snapshots by test
    // scenario.
    let input_clone = input.clone();
    insta::with_settings!({
        description => input_clone,
        snapshot_path => "interpreter_error_snapshots",
        prepend_module_to_snapshot => false,
    }, {
        assert_snapshot!(name, error);
    });

    Ok(())
}
