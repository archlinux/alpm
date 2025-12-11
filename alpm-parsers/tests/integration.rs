//! Integration tests for the alpm-parsers crate.

use std::thread;

use alpm_parsers::custom_ini::parser::ini_file;
use insta::assert_snapshot;
use rstest::rstest;
use testresult::TestResult;
use winnow::Parser;

#[rstest]
#[case::missing_key("= value")]
#[case::incomplete_delimiter_1("key= value")]
#[case::incomplete_delimiter_2("key =value")]
#[case::missing_delimiter_2("key value")]
fn write_ini(#[case] input: &str) -> TestResult {
    let Err(err) = ini_file.parse(input) else {
        panic!("Expected input to fail:\n{input}");
    };

    assert_snapshot!(
        thread::current()
            .name()
            .unwrap_or("?")
            .to_string()
            .replace("::", "__"),
        format!("{err}"),
    );

    Ok(())
}
