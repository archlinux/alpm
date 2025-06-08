//! Integration tests to test correct SRCINFO files.

// NOTE: With rstest > 0.25.0 this can be removed!
#![allow(missing_docs)]

use std::{fs::read_to_string, path::PathBuf};

use alpm_srcinfo::{MergedPackage, SourceInfoV1};
use alpm_types::Architecture;
use insta::assert_snapshot;
use rstest::rstest;
use testresult::TestResult;

/// Get some correct SRCINFO files and make sure the JSON output is created as expected.
///
/// This test also looks for specific keywords in the generated output, specifically:
/// - `unexpected` is used for any kind of value that shouldn't be included in the JSON output.
/// - `beefc0ffee` is used to mark hex values that shouldn't be included in the JSON.
///
/// The SRCINFO files are generated from `*.pkgbuild` files in the `tests/correct` folder
/// Each `*.pkgbuild` file contains an explanation of what it tests.
/// To regenerate the SRCINFO files run the following command in the `alpm-srcinfo` project root:
///
/// ```sh
/// ./tests/generate_srcinfo.bash tests/*/*.pkgbuild
/// ```
///
/// `makepkg` expects changelog and INSTALL files to be in the build directory when creating
/// the SRCINFO file. The script also takes care of creating those files.
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

    let source_info_json = serde_json::to_string_pretty(&source_info)?;

    let test_name = case.file_stem().unwrap().to_str().unwrap().to_string();
    // Compare the generated source_info json with the expected snapshot.
    // Remove the usual module prefix by explicitly setting the snapshot path.
    // This is necessary, as we're manually sorting snapshots by test scenario.
    insta::with_settings!({
        description => format!("{test_name} SourceInfo representation."),
        snapshot_path => "correct_snapshots",
        prepend_module_to_snapshot => false,
    }, {
        assert_snapshot!(format!("{test_name}_source_info"), source_info_json );
    });

    let packages = source_info
        .packages_for_architecture(Architecture::X86_64)
        .collect::<Vec<MergedPackage>>();

    let package_json = serde_json::to_string_pretty(&packages)?;

    if package_json.contains("unexpected") {
        return Err(format!(
            "Found 'unexpected' keyword in json output. {}:\n{package_json}",
            "This indicates that data was included that shouldn't be in there"
        )
        .into());
    }

    if package_json.contains("beefc0ffee") {
        return Err(format!(
            "Found 'beefc0ffee' keyword in json output. {}:\n{package_json}",
            "This indicates that an checksum was included that shouldn't be in there"
        )
        .into());
    }

    // Compare the generated merged json with the expected snapshot.
    // Remove the usual module prefix by explicitly setting the snapshot path.
    // This is necessary, as we're manually sorting snapshots by test scenario.
    insta::with_settings!({
        description => format!("{test_name} merged representation."),
        snapshot_path => "correct_snapshots",
        prepend_module_to_snapshot => false,
    }, {
        assert_snapshot!(format!("{test_name}_merged"), package_json);
    });

    Ok(())
}
