//! This test file contains basic tests to ensure that the `alpm-db-desc` CLI behaves as expected.

use std::{fs::File, io::Write};

use alpm_db_desc::{DbDescFileV1, DbDescFileV2};
use assert_cmd::Command;
use tempfile::tempdir;
use testresult::TestResult;

/// A string slice representing valid DB DESC v1 data.
///
/// <https://alpm.archlinux.page/specifications/alpm-db-desc.5.html>
pub const VALID_DESC_V1: &str = r#"
%NAME%
foo

%VERSION%
1.0.0-1

%BASE%
foo

%DESC%
An example package

%URL%
https://example.org

%ARCH%
x86_64

%BUILDDATE%
1733737242

%INSTALLDATE%
1733737243

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%SIZE%
123

%GROUPS%
utils
cli

%REASON%
1

%LICENSE%
MIT
Apache-2.0

%VALIDATION%
pgp

%REPLACES%
pkg-old

%DEPENDS%
glibc

%OPTDEPENDS%
optpkg

%CONFLICTS%
foo-old

%PROVIDES%
foo-virtual
"#;

/// A string slice representing valid DB DESC v2 data (adds XDATA).
pub const VALID_DESC_V2: &str = r#"
%NAME%
foo

%VERSION%
1.0.0-1

%BASE%
foo

%DESC%
An example package

%URL%
https://example.org

%ARCH%
x86_64

%BUILDDATE%
1733737242

%INSTALLDATE%
1733737243

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%SIZE%
123

%GROUPS%
utils
cli

%REASON%
1

%LICENSE%
MIT
Apache-2.0

%VALIDATION%
pgp

%REPLACES%
pkg-old

%DEPENDS%
glibc

%OPTDEPENDS%
optpkg

%CONFLICTS%
foo-old

%PROVIDES%
foo-virtual

%XDATA%
pkgtype=pkg
"#;

mod create {
    use std::str::FromStr;

    use super::*;

    /// Create a desc file via CLI.
    #[test]
    fn create_v2() -> TestResult {
        let tmp = tempdir()?;
        let out = tmp.path().join(".DESC");

        let mut cmd = Command::cargo_bin("alpm-db-desc")?;
        cmd.args([
            "create",
            "v2",
            "--name",
            "foo",
            "--version",
            "1.0.0-1",
            "--base",
            "foo",
            "--arch",
            "x86_64",
            "--builddate",
            "1733737242",
            "--installdate",
            "1733737243",
            "--packager",
            "Foobar <foo@example.org>",
            "--size",
            "123",
            "--xdata",
            "pkgtype=pkg",
            out.to_string_lossy().as_ref(),
        ]);

        println!("Running command: {:?}", cmd);
        cmd.assert().success();

        let file_content = std::fs::read_to_string(out)?;
        let parsed = DbDescFileV2::from_str(&file_content)?;
        assert_eq!(parsed.name().to_string(), "foo");
        assert_eq!(parsed.arch().to_string(), "x86_64");

        Ok(())
    }
}

mod validate {
    use super::*;

    /// Validate a valid desc file input from stdin
    #[test]
    fn validate_stdin() -> TestResult {
        let mut cmd = Command::cargo_bin("alpm-db-desc")?;
        cmd.args(["validate"]);
        cmd.write_stdin(VALID_DESC_V1);
        cmd.assert().success();
        Ok(())
    }

    /// Validate a valid DESC v2 file from disk
    #[test]
    fn validate_file_v2() -> TestResult {
        let tmp = tempfile::tempdir()?;
        let path = tmp.path().join("test.desc");
        let mut f = File::create(&path)?;
        f.write_all(VALID_DESC_V2.as_bytes())?;

        let mut cmd = Command::cargo_bin("alpm-db-desc")?;
        cmd.args(["validate", path.to_str().unwrap()]);
        cmd.assert().success();
        Ok(())
    }

    /// Validate an invalid DESC input from stdin
    #[test]
    fn validate_invalid_stdin() -> TestResult {
        let mut cmd = Command::cargo_bin("alpm-db-desc")?;
        cmd.args(["validate"]);
        cmd.write_stdin(format!("{VALID_DESC_V1}\n%UNKNOWN%\nvalue"));
        cmd.assert().failure();
        Ok(())
    }
}

mod format {
    use rstest::rstest;

    use super::*;

    /// Format DESC files as JSON (both pretty and not)
    #[rstest]
    #[case::pretty(true)]
    #[case::not_pretty(false)]
    fn format_json(#[case] pretty: bool) -> TestResult {
        let mut cmd = Command::cargo_bin("alpm-db-desc")?;
        cmd.args(["format", "--output-format", "json"]);
        if pretty {
            cmd.arg("--pretty");
        }
        cmd.write_stdin(VALID_DESC_V1);
        let output = cmd.assert().success().get_output().clone();

        let parsed: DbDescFileV1 = serde_json::from_slice(&output.stdout)?;
        assert_eq!(parsed.name().to_string(), "foo");
        Ok(())
    }
}
