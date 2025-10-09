//! This module contains test specifically for leading/trailing whitespace handling in the parser.

use alpm_srcinfo::{SourceInfoV1, source_info::v1::package::Override};
use alpm_types::Name;
use testresult::TestResult;

/// Test the scenario where **no** trailing newline exists on a `pkgname` line.
#[test]
fn missing_newline_after_package() -> TestResult {
    let input = r#"pkgbase = example
    pkgver = 0.1.0
    pkgrel = 1
    arch = x86_64

pkgname = example"#;

    let source_info = SourceInfoV1::from_string(input)?;
    assert_eq!(source_info.base.name, Name::new("example")?);
    assert_eq!(source_info.packages[0].name, Name::new("example")?);
    Ok(())
}

/// Test the scenario where **no** trailing newline exists on any package property's line.
#[test]
fn missing_newline_after_depends() -> TestResult {
    let input = r#"pkgbase = example
    pkgver = 0.1.0
    pkgrel = 1
    arch = x86_64

pkgname = example
depends = some_dependency"#;

    let source_info = SourceInfoV1::from_string(input)?;
    assert_eq!(source_info.base.name, Name::new("example")?);
    assert_eq!(source_info.packages[0].name, Name::new("example")?);
    // Verify dependencies field is overridden
    assert!(matches!(
        source_info.packages[0].dependencies,
        Override::Yes { .. }
    ));
    Ok(())
}

/// Test the scenario where trailing newline **with** trailing whitespaces exists at the end of the
/// file.
#[test]
fn trailing_whitespace_after_newline() -> TestResult {
    let input = r#"pkgbase = example
    pkgver = 0.1.0
    pkgrel = 1
    arch = x86_64

pkgname = example
   	  "#;

    let source_info = SourceInfoV1::from_string(input)?;
    assert_eq!(source_info.base.name, Name::new("example")?);
    assert_eq!(source_info.packages[0].name, Name::new("example")?);
    Ok(())
}
