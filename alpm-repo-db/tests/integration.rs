//! Integration tests for `alpm-repo-desc`.
//!
//! These tests are only executed when the `cli` feature flag is enabled.
#![cfg(feature = "cli")]

use std::{fs::File, io::Write, str::FromStr, thread};

use alpm_repo_db::desc::{RepoDescFile, RepoDescFileV1, RepoDescFileV2, RepoDescSchema};
use alpm_types::{SchemaVersion, semver_version::Version};
use assert_cmd::cargo::cargo_bin_cmd;
use insta::assert_snapshot;
use rstest::rstest;
use tempfile::tempdir;
use testresult::TestResult;

/// A string slice representing valid [alpm-repo-descv1] data.
///
/// [alpm-repo-descv1]: https://alpm.archlinux.page/specifications/alpm-repo-descv1.5.html
pub const VALID_DESC_V1: &str = r#"
%FILENAME%
example-1.0.0-1-any.pkg.tar.zst

%NAME%
example

%BASE%
example

%VERSION%
1.0.0-1

%DESC%
An example package

%GROUPS%
example-group
other-group

%CSIZE%
1818463

%ISIZE%
18184634

%MD5SUM%
d3b07384d113edec49eaa6238ad5ff00

%SHA256SUM%
b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c

%PGPSIG%
iHUEABYKAB0WIQRizHP4hOUpV7L92IObeih9mi7GCAUCaBZuVAAKCRCbeih9mi7GCIlMAP9ws/jU4f580ZRQlTQKvUiLbAZOdcB7mQQj83hD1Nc/GwD/WIHhO1/OQkpMERejUrLo3AgVmY3b4/uGhx9XufWEbgE=

%URL%
https://example.org/

%LICENSE%
MIT
Apache-2.0

%ARCH%
x86_64

%BUILDDATE%
1729181726

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%REPLACES%
other-pkg-replaced

%CONFLICTS%
other-pkg-conflicts

%PROVIDES%
example-component

%DEPENDS%
glibc
gcc-libs

%OPTDEPENDS%
bash: for a script

%MAKEDEPENDS%
cmake

%CHECKDEPENDS%
bats
"#;

/// A string slice representing valid [alpm-repo-descv2] data.
///
/// [alpm-repo-descv2]: https://alpm.archlinux.page/specifications/alpm-repo-descv2.5.html>
pub const VALID_DESC_V2: &str = r#"
%FILENAME%
example-1.0.0-1-any.pkg.tar.zst

%NAME%
example

%BASE%
example

%VERSION%
1.0.0-1

%DESC%
An example package

%GROUPS%
example-group
other-group

%CSIZE%
1818463

%ISIZE%
18184634

%SHA256SUM%
b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c

%URL%
https://example.org/

%LICENSE%
MIT
Apache-2.0

%ARCH%
x86_64

%BUILDDATE%
1729181726

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%REPLACES%
other-pkg-replaced

%CONFLICTS%
other-pkg-conflicts

%PROVIDES%
example-component

%DEPENDS%
glibc
gcc-libs

%OPTDEPENDS%
bash: for a script

%MAKEDEPENDS%
cmake

%CHECKDEPENDS%
bats
"#;

/// Convenience fixture helper
fn schema_fixture(schema: &RepoDescSchema) -> (&'static str, &'static str) {
    match schema {
        RepoDescSchema::V1(_) => ("v1", VALID_DESC_V1),
        RepoDescSchema::V2(_) => ("v2", VALID_DESC_V2),
    }
}

mod validate {

    use super::*;
    /// Autodetect schema: v1
    #[test]
    fn v1_stdin() -> TestResult {
        let mut cmd = cargo_bin_cmd!("alpm-repo-desc");
        cmd.arg("validate");
        cmd.write_stdin(VALID_DESC_V1);
        cmd.assert().success();
        Ok(())
    }

    /// Autodetect schema: v2
    #[test]
    fn v2_stdin() -> TestResult {
        let mut cmd = cargo_bin_cmd!("alpm-repo-desc");
        cmd.arg("validate");
        cmd.write_stdin(VALID_DESC_V2);
        cmd.assert().success();
        Ok(())
    }

    /// Validate from file (v2)
    #[test]
    fn v2_file() -> TestResult {
        let tmp = tempfile::tempdir()?;
        let path = tmp.path().join("desc");
        let mut f = File::create(&path)?;
        f.write_all(VALID_DESC_V2.as_bytes())?;

        let mut cmd = cargo_bin_cmd!("alpm-repo-desc");
        cmd.args(["validate".into(), path.to_string_lossy().to_string()]);
        cmd.assert().success();
        Ok(())
    }

    /// Invalid input: unknown section
    #[test]
    fn invalid_unknown_section() -> TestResult {
        let mut cmd = cargo_bin_cmd!("alpm-repo-desc");
        cmd.arg("validate");
        cmd.write_stdin(format!("{VALID_DESC_V1}\n%UNKNOWN%\nvalue"));
        cmd.assert().failure();
        Ok(())
    }
}

mod create_cli {
    use std::fs;

    use super::*;

    /// Create DESC files (v1 and v2) via CLI arguments and snapshot the result.
    #[rstest]
    #[case::v1(RepoDescSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))))]
    #[case::v2(RepoDescSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))))]
    fn create(#[case] schema: RepoDescSchema) -> TestResult {
        let tmp = tempdir()?;
        let out = tmp.path().join("desc");

        let (version_flag, _data) = super::schema_fixture(&schema);

        // Common arguments shared between v1 and v2
        let mut args = vec![
            "create".to_string(),
            version_flag.to_string(),
            "--filename".into(),
            "example-1.0.0-1-any.pkg.tar.zst".into(),
            "--name".into(),
            "foo".into(),
            "--base".into(),
            "foo".into(),
            "--version".into(),
            "1.0.0-1".into(),
            "--description".into(),
            "An example package".into(),
            "--groups".into(),
            "example-group".into(),
            "other-group".into(),
            "--csize".into(),
            "1818463".into(),
            "--isize".into(),
            "18184634".into(),
            "--sha256sum".into(),
            "b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c".into(),
            "--pgpsig".into(),
            "iHUEABYKAB0WIQRizHP4hOUpV7L92IObeih9mi7GCAUCaBZuVAAKCRCbeih9mi7GCIlMAP9ws/jU4f580ZRQlTQKvUiLbAZOdcB7mQQj83hD1Nc/GwD/WIHhO1/OQkpMERejUrLo3AgVmY3b4/uGhx9XufWEbgE=".into(),
            "--url".into(),
            "https://example.org/".into(),
            "--license".into(),
            "MIT".into(),
            "Apache-2.0".into(),
            "--arch".into(),
            "x86_64".into(),
            "--builddate".into(),
            "1729181726".into(),
            "--packager".into(),
            "Foobar McFooface <foobar@mcfooface.org>".into(),
            "--replaces".into(),
            "other-pkg-replaced".into(),
            "--conflicts".into(),
            "other-pkg-conflicts".into(),
            "--provides".into(),
            "example-component".into(),
            "--depends".into(),
            "glibc".into(),
            "gcc-libs".into(),
            "--optdepends".into(),
            "bash: for a script".into(),
            "--makedepends".into(),
            "cmake".into(),
            "--checkdepends".into(),
            "bats".into(),
        ];

        // Add v1-only field
        if matches!(schema, RepoDescSchema::V1(_)) {
            args.extend(["--md5sum".into(), "d3b07384d113edec49eaa6238ad5ff00".into()]);
        }

        args.push(out.to_string_lossy().into());

        // Run the command
        let mut cmd = cargo_bin_cmd!("alpm-repo-desc");
        cmd.args(&args);
        cmd.assert().success();

        // Read and snapshot result
        let s = fs::read_to_string(&out)?;
        let test_name = thread::current()
            .name()
            .map(|n| n.replace("::", "__"))
            .unwrap_or_else(|| "unknown_test".to_string());

        let sanitized_args = args
            .iter()
            .enumerate()
            .map(|(idx, arg)| {
                if idx == args.len() - 1 {
                    "desc".to_string()
                } else {
                    arg.to_string()
                }
            })
            .collect::<Vec<_>>();
        let description = format!("alpm-repo-desc {}", sanitized_args.join(" "));
        insta::with_settings!({ description => description }, {
            assert_snapshot!(test_name, s);
        });

        // Verify schema-specific conditions
        match schema {
            RepoDescSchema::V1(_) => {
                let parsed = RepoDescFileV1::from_str(&s)?;
                assert_eq!(parsed.name.to_string(), "foo");
                assert!(
                    s.contains("%MD5SUM%"),
                    "v1 output must contain MD5SUM section"
                );
                assert!(
                    s.contains("%PGPSIG%"),
                    "v1 output must contain PGPSIG section"
                );
            }
            RepoDescSchema::V2(_) => {
                let parsed = RepoDescFileV2::from_str(&s)?;
                assert_eq!(parsed.name.to_string(), "foo");
                assert!(
                    !s.contains("%MD5SUM%"),
                    "v2 output can't contain MD5SUM section"
                );
            }
        }

        Ok(())
    }
}

mod create_env {
    use std::collections::HashMap;

    use super::*;

    /// Create DESC files (v1 and v2) via environment variables instead of CLI args.
    #[rstest]
    #[case::v1(RepoDescSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))))]
    #[case::v2(RepoDescSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))))]
    fn create(#[case] schema: RepoDescSchema) -> TestResult {
        // TODO
    }
}

mod format {
    use rstest::rstest;

    use super::*;

    /// Format as JSON (pretty and compact) from stdin for both schemas
    #[rstest]
    #[case(RepoDescSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))))]
    #[case(RepoDescSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))))]
    fn json_compact(#[case] schema: RepoDescSchema) -> TestResult {
        // TODO
    }

    #[rstest]
    #[case(RepoDescSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))))]
    #[case(RepoDescSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))))]
    fn json_pretty(#[case] schema: RepoDescSchema) -> TestResult {
        // TODO
    }
}

mod display {
    use super::*;

    /// Ensure `Display` output can be parsed again and is semantically identical.
    #[rstest]
    #[case(RepoDescSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))))]
    #[case(RepoDescSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))))]
    fn display_round_trip(#[case] schema: RepoDescSchema) -> TestResult {
        // TODO
    }
}
