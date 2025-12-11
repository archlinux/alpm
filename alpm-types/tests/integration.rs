//! Integration tests for `alpm-types`.

use std::{path::PathBuf, thread::current};

use alpm_types::PackageFileName;
use insta::{assert_snapshot, with_settings};
use log::{LevelFilter, debug};
use rstest::rstest;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use testresult::TestResult;
use winnow::Parser;

fn init_logger() -> TestResult {
    if TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .is_err()
    {
        debug!("Not initializing another logger, as one is initialized already.");
    }

    Ok(())
}

/// Tests that cases of broken package file names as string slice lead to error.
#[rstest]
#[case::no_name("-1.0.0-1-x86_64.pkg.tar.zst")]
#[case::no_name("1.0.0-1-x86_64.pkg.tar.zst")]
#[case::invalid_version("example-1-x86_64.pkg.tar.zst")]
#[case::no_version("example-x86_64.pkg.tar.zst")]
#[case::no_version_name_with_dashes("example-pkg-x86_64.pkg.tar.zst")]
#[case::invalid_architecture("example-pkg-1.0.0-1-x86-64.pkg.tar.zst")]
#[case::no_architecture("example-1.0.0-1.pkg.tar.zst")]
#[case::no_architecture_name_with_dashes("example-pkg-1.0.0-1.pkg.tar.zst")]
#[case::invalid_package_marker("example-1.0.0-1-x86_64.foo.zst")]
#[case::no_package_marker("example-1.0.0-1-x86_64.zst")]
#[case::no_tar_ending("example-1.0.0-1-x86_64.pkg.zst")]
#[case::no_package_marker_name_with_dashes("example-pkg-1.0.0-1-x86_64.zst")]
#[case::invalid_compression("example-1.0.0-1-x86_64.pkg.tar.foo")]
#[case::invalid_dashes("example-pkg---x86_64.pkg.tar.zst")]
#[case::invalid_dashes("example---x86_64.pkg.tar.zst")]
#[case::no_dashes("examplepkg1.0.01x86_64.pkg.tar.zst")]
fn fail_to_parse_package_filename(#[case] s: &str) -> TestResult {
    init_logger()?;

    let Err(error) = PackageFileName::parser.parse(s) else {
        panic!("The parser succeeded parsing {s} although it should have failed");
    };

    with_settings!({
                description => s.to_string(),
                snapshot_path => "parse_error_snapshots",
                prepend_module_to_snapshot => false,
            }, {
                assert_snapshot!(current()
                .name()
                .unwrap()
                .to_string()
                .replace("::", "__")
    , format!("{error}"));
            });

    Ok(())
}

/// Tests that cases of broken package file names as [`Path`] lead to error.
#[rstest]
#[case::no_file_name(PathBuf::from("./"))]
fn package_file_name_from_path_fails(#[case] path: PathBuf) -> TestResult {
    init_logger()?;

    let Err(error) = PackageFileName::try_from(path.as_path()) else {
        panic!(
            "Succeeded in creating a PackageFilename from {path:?} although it should have failed"
        );
    };

    with_settings!({
                description => format!("{path:?}"),
                snapshot_path => "from_path_snapshots",
                prepend_module_to_snapshot => false,
            }, {
                assert_snapshot!(current()
                .name()
                .unwrap()
                .to_string()
                .replace("::", "__")
    , format!("{error}"));
            });

    Ok(())
}
