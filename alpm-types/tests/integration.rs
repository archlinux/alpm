use std::thread::current;

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

/// Tests that cases of broken package filenames can be recognized.
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
#[case::no_package_marker_name_with_dashes("example-pkg-1.0.0-1-x86_64.zst")]
#[case::invalid_compression("example-1.0.0-1-x86_64.pkg.tar.foo")]
#[case::invalid_dashes("example-pkg---x86_64.pkg.tar.zst")]
#[case::invalid_dashes("example---x86_64.pkg.tar.zst")]
#[case::no_dashes("examplepkg1.0.01x86_64.pkg.tar.zst")]
fn fail_to_parse_package_filename(#[case] s: &str) -> TestResult {
    init_logger()?;

    let Err(error) = PackageFileName::parser.parse(s) else {
        return Err(
            format!("The parser succeeded parsing {s} although it should have failed").into(),
        );
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

/// Tests that common and uncommon cases of package filenames can be recognized.
#[rstest]
#[case::name_with_dashes("example-pkg-1.0.0-1-x86_64.pkg.tar.zst")]
#[case::no_compression("example-pkg-1.0.0-1-x86_64.pkg.tar")]
#[case::version_as_name("1.0.0-1-1.0.0-1-x86_64.pkg.tar.zst")]
#[case::version_with_epoch("example-1:1.0.0-1-x86_64.pkg.tar.zst")]
#[case::version_with_pkgrel_sub_version("example-1.0.0-1.1-x86_64.pkg.tar.zst")]
fn succeed_to_parse_package_filename(#[case] s: &str) -> TestResult {
    init_logger()?;

    let value = match PackageFileName::parser.parse(s) {
        Err(error) => {
            return Err(format!(
                "The parser failed parsing {s} although it should have succeeded:\n{error}"
            )
            .into());
        }
        Ok(value) => value,
    };

    with_settings!({
                description => s.to_string(),
                snapshot_path => "parse_succeed_snapshots",
                prepend_module_to_snapshot => false,
            }, {
                assert_snapshot!(current()
                .name()
                .unwrap()
                .to_string()
                .replace("::", "__")
    , format!("{value}"));
            });

    Ok(())
}
