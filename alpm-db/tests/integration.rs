//! Integration tests for `alpm-db-desc`.

use std::{fs::File, io::Write, str::FromStr, thread};

use alpm_db::desc::{DbDescFile, DbDescFileV1, DbDescFileV2, DbDescSchema};
use alpm_types::{SchemaVersion, semver_version::Version};
use assert_cmd::Command;
use insta::assert_snapshot;
use rstest::rstest;
use tempfile::tempdir;
use testresult::TestResult;

/// A string slice representing valid [alpm-db-descv1] data.
///
/// [alpm-db-descv1]: https://alpm.archlinux.page/specifications/alpm-db-descv1.5.html
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

/// A string slice representing valid [alpm-db-descv2] data.
///
/// [alpm-db-descv2]: https://alpm.archlinux.page/specifications/alpm-db-descv2.5.html>
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

/// The name of the binary to test.
const BINARY_NAME: &str = "alpm-db-desc";

/// Convenience fixture helper
fn schema_fixture(schema: &DbDescSchema) -> (&'static str, &'static str) {
    match schema {
        DbDescSchema::V1(_) => ("v1", VALID_DESC_V1),
        DbDescSchema::V2(_) => ("v2", VALID_DESC_V2),
    }
}

mod validate {
    use super::*;
    /// Autodetect schema: v1
    #[test]
    fn v1_stdin() -> TestResult {
        let mut cmd = Command::cargo_bin(BINARY_NAME)?;
        cmd.arg("validate");
        cmd.write_stdin(VALID_DESC_V1);
        cmd.assert().success();
        Ok(())
    }

    /// Autodetect schema: v2
    #[test]
    fn v2_stdin() -> TestResult {
        let mut cmd = Command::cargo_bin(BINARY_NAME)?;
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

        let mut cmd = Command::cargo_bin(BINARY_NAME)?;
        cmd.args(["validate".into(), path.to_string_lossy().to_string()]);
        cmd.assert().success();
        Ok(())
    }

    /// Invalid input: unknown section
    #[test]
    fn invalid_unknown_section() -> TestResult {
        let mut cmd = Command::cargo_bin(BINARY_NAME)?;
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
    #[case::v1(DbDescSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))))]
    #[case::v2(DbDescSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))))]
    fn create_via_cli(#[case] schema: DbDescSchema) -> TestResult {
        let tmp = tempdir()?;
        let out = tmp.path().join("desc");

        let (version_flag, _data) = super::schema_fixture(&schema);

        // Common arguments shared between v1 and v2
        let mut args = vec![
            "create".to_string(),
            version_flag.to_string(),
            "--name".into(),
            "foo".into(),
            "--version".into(),
            "1.0.0-1".into(),
            "--base".into(),
            "foo".into(),
            "--description".into(),
            "An example package".into(),
            "--url".into(),
            "https://example.org".into(),
            "--arch".into(),
            "x86_64".into(),
            "--builddate".into(),
            "1733737242".into(),
            "--installdate".into(),
            "1733737243".into(),
            "--packager".into(),
            "Foobar McFooface <foobar@mcfooface.org>".into(),
            "--size".into(),
            "123".into(),
            "--groups".into(),
            "utils".into(),
            "--groups".into(),
            "cli".into(),
            "--reason".into(),
            "1".into(),
            "--license".into(),
            "MIT".into(),
            "--license".into(),
            "Apache-2.0".into(),
            "--validation".into(),
            "pgp".into(),
            "--replaces".into(),
            "pkg-old".into(),
            "--depends".into(),
            "glibc".into(),
            "--optdepends".into(),
            "optpkg".into(),
            "--conflicts".into(),
            "foo-old".into(),
            "--provides".into(),
            "foo-virtual".into(),
        ];

        // Add v2-only field
        if matches!(schema, DbDescSchema::V2(_)) {
            args.extend(["--xdata".into(), "pkgtype=pkg".into()]);
        }

        args.push(out.to_string_lossy().into());

        // Run the command
        let mut cmd = Command::cargo_bin(BINARY_NAME)?;
        cmd.args(args);
        cmd.assert().success();

        // Read and snapshot result
        let s = fs::read_to_string(&out)?;
        let test_name = thread::current()
            .name()
            .map(|n| n.replace("::", "__"))
            .unwrap_or_else(|| "unknown_test".to_string());

        assert_snapshot!(test_name, s);

        // Verify schema-specific conditions
        match schema {
            DbDescSchema::V1(_) => {
                let parsed = DbDescFileV1::from_str(&s)?;
                assert_eq!(parsed.name.to_string(), "foo");
            }
            DbDescSchema::V2(_) => {
                let parsed = DbDescFileV2::from_str(&s)?;
                assert_eq!(parsed.name.to_string(), "foo");
                assert!(
                    s.contains("%XDATA%"),
                    "v2 output must contain XDATA section"
                );
            }
        }

        Ok(())
    }

    /// Ensures that the `%SIZE%` section is omitted when its value is zero.
    #[rstest]
    #[case::v1(DbDescSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))))]
    #[case::v2(DbDescSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))))]
    fn size_zero_is_omitted(#[case] schema: DbDescSchema) -> TestResult {
        let tmp = tempdir()?;
        let out = tmp.path().join("desc");

        let (version_flag, _data) = super::schema_fixture(&schema);

        // Minimal required fields
        let mut args = vec![
            "create".to_string(),
            version_flag.to_string(),
            "--name".into(),
            "foo".into(),
            "--version".into(),
            "1.0.0-1".into(),
            "--base".into(),
            "foo".into(),
            "--arch".into(),
            "x86_64".into(),
            "--builddate".into(),
            "1733737242".into(),
            "--installdate".into(),
            "1733737243".into(),
            "--packager".into(),
            "Foobar <foo@bar>".into(),
            "--size".into(),
            "0".into(),
            out.to_string_lossy().into(),
        ];

        // Add v2-only field
        if matches!(schema, DbDescSchema::V2(_)) {
            args.extend(["--xdata".into(), "pkgtype=pkg".into()]);
        }

        // Run the command
        let mut cmd = Command::cargo_bin(BINARY_NAME)?;
        cmd.args(args);
        cmd.assert().success();

        let written = std::fs::read_to_string(&out)?;

        assert!(
            !written.contains("%SIZE%"),
            "SIZE section should be omitted when size = 0"
        );
        assert!(
            written.contains("%NAME%"),
            "Sanity check: file should contain valid sections"
        );

        Ok(())
    }
}

mod create_env {
    use super::*;

    /// Create DESC files (v1 and v2) via environment variables instead of CLI args.
    #[rstest]
    #[case::v1(DbDescSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))))]
    #[case::v2(DbDescSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))))]
    fn create_via_env(#[case] schema: DbDescSchema) -> TestResult {
        let tmp = tempdir()?;
        let out = tmp.path().join("desc");

        let (version_flag, data) = super::schema_fixture(&schema);
        let parsed = DbDescFile::from_str(data)?;

        // Get a concrete reference to the inner struct for ergonomic access
        let (inner_v1, inner_v2) = match &parsed {
            DbDescFile::V1(v1) => (Some(v1), None),
            DbDescFile::V2(v2) => (None, Some(v2)),
        };

        let mut cmd = Command::cargo_bin(BINARY_NAME)?;
        cmd.args(["create", version_flag]);

        // Set environment variables based on the parsed data
        let inner = if let Some(v1) = inner_v1 {
            v1.clone()
        } else if let Some(v2) = inner_v2 {
            DbDescFileV1::from(v2.clone())
        } else {
            return Err("no valid v1 or v2 data found".into());
        };
        cmd.env("ALPM_DB_DESC_NAME", inner.name.to_string())
            .env("ALPM_DB_DESC_VERSION", inner.version.to_string())
            .env("ALPM_DB_DESC_BASE", inner.base.to_string())
            .env(
                "ALPM_DB_DESC_DESC",
                inner.description.as_ref().map_or("", |d| d.as_ref()),
            )
            .env(
                "ALPM_DB_DESC_URL",
                inner.url.as_ref().map_or("", |u| u.as_ref()),
            )
            .env("ALPM_DB_DESC_ARCH", inner.arch.to_string())
            .env("ALPM_DB_DESC_BUILDDATE", inner.builddate.to_string())
            .env("ALPM_DB_DESC_INSTALLDATE", inner.installdate.to_string())
            .env("ALPM_DB_DESC_PACKAGER", inner.packager.to_string())
            .env("ALPM_DB_DESC_SIZE", inner.size.to_string())
            .env(
                "ALPM_DB_DESC_REASON",
                inner.reason.map_or(String::new(), |r| r.to_string()),
            )
            .arg(out.to_string_lossy().as_ref());

        // Macro to shorten repetitive env setup
        macro_rules! env_join {
            ($key:literal, $getter:expr) => {
                cmd.env(
                    $key,
                    $getter
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(" "),
                );
            };
        }

        env_join!("ALPM_DB_DESC_GROUPS", inner.groups);
        env_join!("ALPM_DB_DESC_LICENSE", inner.license);
        env_join!("ALPM_DB_DESC_VALIDATION", inner.validation);
        env_join!("ALPM_DB_DESC_REPLACES", inner.replaces);
        env_join!("ALPM_DB_DESC_DEPENDS", inner.depends);
        env_join!("ALPM_DB_DESC_OPTDEPENDS", inner.optdepends);
        env_join!("ALPM_DB_DESC_CONFLICTS", inner.conflicts);
        env_join!("ALPM_DB_DESC_PROVIDES", inner.provides);

        if let Some(v2) = inner_v2 {
            env_join!("ALPM_DB_DESC_XDATA", v2.xdata());
        }

        cmd.assert().success();

        let written = std::fs::read_to_string(&out)?;
        let reparsed = match schema {
            DbDescSchema::V1(_) => DbDescFileV1::from_str(&written)?.to_string(),
            DbDescSchema::V2(_) => DbDescFileV2::from_str(&written)?.to_string(),
        };

        let test_name = thread::current()
            .name()
            .map(|n| n.replace("::", "__"))
            .unwrap_or_else(|| "unknown_test".to_string());

        assert_snapshot!(test_name, reparsed);

        Ok(())
    }
}

mod format {
    use rstest::rstest;

    use super::*;

    /// Format as JSON (pretty and compact) from stdin for both schemas
    #[rstest]
    #[case(DbDescSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))))]
    #[case(DbDescSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))))]
    fn json_compact(#[case] schema: DbDescSchema) -> TestResult {
        let (_, data) = super::schema_fixture(&schema);

        let mut cmd = Command::cargo_bin(BINARY_NAME)?;
        cmd.args(["format", "--output-format", "json"]);
        cmd.write_stdin(data);
        let output = cmd.assert().success().get_output().clone();

        match schema {
            DbDescSchema::V1(_) => {
                let parsed: DbDescFileV1 = serde_json::from_slice(&output.stdout)?;
                assert_eq!(parsed.name.to_string(), "foo");
            }
            DbDescSchema::V2(_) => {
                let parsed: DbDescFileV2 = serde_json::from_slice(&output.stdout)?;
                assert_eq!(parsed.name.to_string(), "foo");
                assert!(!parsed.xdata().is_empty());
            }
        }
        Ok(())
    }

    #[rstest]
    #[case(DbDescSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))))]
    #[case(DbDescSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))))]
    fn json_pretty(#[case] schema: DbDescSchema) -> TestResult {
        let (_, data) = super::schema_fixture(&schema);

        let mut cmd = Command::cargo_bin(BINARY_NAME)?;
        cmd.args(["format", "--output-format", "json", "--pretty"]);
        cmd.write_stdin(data);

        let output = cmd.assert().success().get_output().clone();
        let json = String::from_utf8_lossy(&output.stdout);

        let test_name = thread::current()
            .name()
            .map(|n| n.replace("::", "__"))
            .unwrap_or_else(|| "unknown_test".to_string());

        assert_snapshot!(test_name, json.to_string());
        Ok(())
    }
}

mod display {
    use super::*;

    /// Ensure `Display` output can be parsed again and is semantically identical.
    #[rstest]
    #[case(DbDescSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))))]
    #[case(DbDescSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))))]
    fn display_round_trip(#[case] schema: DbDescSchema) -> TestResult {
        let (_, data) = super::schema_fixture(&schema);

        // Parse into enum
        let file = DbDescFile::from_str(data)?;
        let printed = file.to_string();

        // Re-parse and compare semantically
        let reparsed = DbDescFile::from_str(&printed)?;
        match (file, reparsed) {
            (DbDescFile::V1(a), DbDescFile::V1(b)) => assert_eq!(a, b),
            (DbDescFile::V2(a), DbDescFile::V2(b)) => assert_eq!(a, b),
            _ => panic!("schema changed after round-trip"),
        }
        Ok(())
    }
}
