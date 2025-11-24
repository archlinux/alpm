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
        let out = tmp.path().join("desc").to_string_lossy().to_string();

        let (version_flag, _data) = super::schema_fixture(&schema);

        // Common arguments shared between v1 and v2
        let mut args = vec![
            "create",
            version_flag,
            "--filename",
            "example-1.0.0-1-any.pkg.tar.zst",
            "--name",
            "foo",
            "--base",
            "foo",
            "--version",
            "1.0.0-1",
            "--description",
            "An example package",
            "--groups",
            "example-group",
            "--groups",
            "other-group",
            "--csize",
            "1818463",
            "--isize",
            "18184634",
            "--sha256sum",
            "b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c",
            "--pgpsig",
            "iHUEABYKAB0WIQRizHP4hOUpV7L92IObeih9mi7GCAUCaBZuVAAKCRCbeih9mi7GCIlMAP9ws/jU4f580ZRQlTQKvUiLbAZOdcB7mQQj83hD1Nc/GwD/WIHhO1/OQkpMERejUrLo3AgVmY3b4/uGhx9XufWEbgE=",
            "--url",
            "https://example.org/",
            "--license",
            "MIT",
            "--license",
            "Apache-2.0",
            "--arch",
            "x86_64",
            "--builddate",
            "1729181726",
            "--packager",
            "Foobar McFooface <foobar@mcfooface.org>",
            "--replaces",
            "other-pkg-replaced",
            "--conflicts",
            "other-pkg-conflicts",
            "--provides",
            "example-component",
            "--depends",
            "glibc",
            "--depends",
            "gcc-libs",
            "--optdepends",
            "bash: for a script",
            "--makedepends",
            "cmake",
            "--checkdepends",
            "bats",
        ];

        // Add v1-only field
        if matches!(schema, RepoDescSchema::V1(_)) {
            args.extend(["--md5sum", "d3b07384d113edec49eaa6238ad5ff00"]);
        }

        args.push(&out);

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
        let tmp = tempdir()?;
        let output_path = tmp.path().join("desc");

        let (version_flag, data) = schema_fixture(&schema);
        let parsed = RepoDescFile::from_str(data)?;

        // Get a concrete reference to the inner struct for ergonomic access
        let (inner_v1, inner_v2) = match &parsed {
            RepoDescFile::V1(v1) => (Some(v1), None),
            RepoDescFile::V2(v2) => (None, Some(v2)),
        };

        let mut cmd = cargo_bin_cmd!("alpm-repo-desc");
        cmd.args(["create", version_flag]);
        let mut cli_args = vec!["create".to_string(), version_flag.to_string()];

        // Set environment variables based on the parsed data
        let inner = if let Some(v1) = inner_v1 {
            RepoDescFileV2::from(v1.clone())
        } else if let Some(v2) = inner_v2 {
            v2.clone()
        } else {
            unreachable!("no valid v1 or v2 data found");
        };

        let mut envs = HashMap::new();

        // Insert all single-value parameters
        envs.insert("ALPM_REPO_DESC_FILENAME", inner.file_name.to_string());
        envs.insert("ALPM_REPO_DESC_NAME", inner.name.to_string());
        envs.insert("ALPM_REPO_DESC_BASE", inner.base.to_string());
        envs.insert("ALPM_REPO_DESC_VERSION", inner.version.to_string());
        envs.insert("ALPM_REPO_DESC_DESC", inner.description.to_string());
        envs.insert("ALPM_REPO_DESC_CSIZE", inner.compressed_size.to_string());
        envs.insert("ALPM_REPO_DESC_ISIZE", inner.installed_size.to_string());
        envs.insert(
            "ALPM_REPO_DESC_SHA256SUM",
            inner.sha256_checksum.to_string(),
        );
        envs.insert(
            "ALPM_REPO_DESC_URL",
            inner.url.map_or(String::new(), |s| s.to_string()),
        );
        envs.insert("ALPM_REPO_DESC_ARCH", inner.arch.to_string());
        envs.insert("ALPM_REPO_DESC_BUILDDATE", inner.build_date.to_string());
        envs.insert("ALPM_REPO_DESC_PACKAGER", inner.packager.to_string());

        // Helper macro to shorten env setup handling for lists.
        macro_rules! env_join_list {
            ($key:literal, $getter:expr, $delimiter:expr) => {{
                let value = $getter
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join($delimiter);
                envs.insert($key, value);
            }};
            ($key:literal, $getter:expr) => {{
                env_join_list!($key, $getter, " ");
            }};
        }

        // Insert all group parameters
        env_join_list!("ALPM_REPO_DESC_GROUPS", inner.groups);
        env_join_list!("ALPM_REPO_DESC_LICENSE", inner.license);
        env_join_list!("ALPM_REPO_DESC_REPLACES", inner.replaces);
        env_join_list!("ALPM_REPO_DESC_DEPENDS", inner.dependencies);
        env_join_list!(
            "ALPM_REPO_DESC_OPTDEPENDS",
            inner.optional_dependencies,
            ","
        );
        env_join_list!("ALPM_REPO_DESC_MAKEDEPENDS", inner.make_dependencies);
        env_join_list!("ALPM_REPO_DESC_CHECKDEPENDS", inner.check_dependencies);
        env_join_list!("ALPM_REPO_DESC_CONFLICTS", inner.conflicts);
        env_join_list!("ALPM_REPO_DESC_PROVIDES", inner.provides);

        if let Some(v1) = inner_v1 {
            envs.insert("ALPM_REPO_DESC_MD5SUM", v1.md5_checksum.to_string());
            envs.insert("ALPM_REPO_DESC_PGPSIG", v1.pgp_signature.to_string());
        }

        // Add all arguments to the command and create a debug `env_string`, which will be
        // displayed in the insta snapshot's description.
        let mut env_strings = Vec::new();
        for (key, value) in envs {
            env_strings.push(format!("{key}={value}"));
            cmd.env(key, value);
        }
        let env_string = env_strings.join(" ");

        // Push the output path as an actual string.
        let out_arg = output_path.to_string_lossy().to_string();
        cli_args.push(out_arg.clone());
        cmd.arg(out_arg);

        // Run the command and assert that it succeeds
        cmd.assert().success();

        let written = std::fs::read_to_string(&output_path)?;
        let reparsed = match schema {
            RepoDescSchema::V1(_) => RepoDescFileV1::from_str(&written)?.to_string(),
            RepoDescSchema::V2(_) => RepoDescFileV2::from_str(&written)?.to_string(),
        };

        let test_name = thread::current()
            .name()
            .map(|n| n.replace("::", "__"))
            .unwrap_or_else(|| "unknown_test".to_string());

        let sanitized_args = cli_args
            .iter()
            .enumerate()
            .map(|(idx, arg)| {
                if idx == cli_args.len() - 1 {
                    "desc".to_string()
                } else {
                    arg.to_string()
                }
            })
            .collect::<Vec<_>>();
        let description = format!(
            "Args: alpm-repo-desc {} | Env: {}",
            sanitized_args.join(" "),
            env_string
        );
        insta::with_settings!({ description => description }, {
            assert_snapshot!(test_name, reparsed);
        });

        Ok(())
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
        let (_, data) = schema_fixture(&schema);

        let mut cmd = cargo_bin_cmd!("alpm-repo-desc");
        cmd.args(["format", "--output-format", "json"]);
        cmd.write_stdin(data);
        let output = cmd.assert().success().get_output().clone();

        match schema {
            RepoDescSchema::V1(_) => {
                let parsed: RepoDescFileV1 = serde_json::from_slice(&output.stdout)?;
                assert_eq!(parsed.name.to_string(), "example");
            }
            RepoDescSchema::V2(_) => {
                let parsed: RepoDescFileV2 = serde_json::from_slice(&output.stdout)?;
                assert_eq!(parsed.name.to_string(), "example");
            }
        }
        Ok(())
    }

    #[rstest]
    #[case(RepoDescSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))))]
    #[case(RepoDescSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))))]
    fn json_pretty(#[case] schema: RepoDescSchema) -> TestResult {
        let (_, data) = schema_fixture(&schema);

        let mut cmd = cargo_bin_cmd!("alpm-repo-desc");
        let args = ["format", "--output-format", "json", "--pretty"];
        cmd.args(args);
        cmd.write_stdin(data);

        let output = cmd.assert().success().get_output().clone();
        let json = String::from_utf8_lossy(&output.stdout).to_string();

        let test_name = thread::current()
            .name()
            .map(|n| n.replace("::", "__"))
            .unwrap_or_else(|| "unknown_test".to_string());

        let description = format!("alpm-repo-desc {}", args.join(" "));
        insta::with_settings!({ description => description }, {
            assert_snapshot!(test_name, json);
        });
        Ok(())
    }
}

mod display {
    use super::*;

    /// Ensure `Display` output can be parsed again and is semantically identical.
    #[rstest]
    #[case(RepoDescSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))))]
    #[case(RepoDescSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))))]
    fn display_round_trip(#[case] schema: RepoDescSchema) -> TestResult {
        let (_, data) = schema_fixture(&schema);

        // Parse into enum
        let file = RepoDescFile::from_str(data)?;
        let printed = file.to_string();

        // Re-parse and compare semantically
        let reparsed = RepoDescFile::from_str(&printed)?;
        match (file, reparsed) {
            (RepoDescFile::V1(a), RepoDescFile::V1(b)) => assert_eq!(a, b),
            (RepoDescFile::V2(a), RepoDescFile::V2(b)) => assert_eq!(a, b),
            _ => panic!("schema changed after round-trip"),
        }
        Ok(())
    }
}
