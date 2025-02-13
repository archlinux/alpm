use std::{fs::read_to_string, path::PathBuf};

use alpm_srcinfo::{MergedPackage, SourceInfo};
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
    let source_info_result = SourceInfo::from_string(input.as_str());

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
            )
        }
    };

    let packages = source_info
        .packages_for_architecture(Architecture::X86_64)
        .collect::<Vec<MergedPackage>>();

    let json = serde_json::to_string_pretty(&packages)?;
    let name = case.file_stem().unwrap().to_str().unwrap().to_string();

    if json.contains("unexpected") {
        return Err(format!(
            "Found 'unexpected' keyword in json output. {}:\n{json}",
            "This indicates that data was included that shouldn't be in there"
        )
        .into());
    }

    if json.contains("beefc0ffee") {
        return Err(format!(
            "Found 'beefc0ffee' keyword in json output. {}:\n{json}",
            "This indicates that an checksum was included that shouldn't be in there"
        )
        .into());
    }

    // Compare the generated json with the expected snapshot.
    // Remove the usual module prefix by explicitly setting the snapshot path.
    // This is necessary, as we're manually sorting snapshots by test scenario.
    insta::with_settings!({
        snapshot_path => "correct_snapshots",
        prepend_module_to_snapshot => false,
    }, {
        assert_snapshot!(name, json);
    });

    Ok(())
}
