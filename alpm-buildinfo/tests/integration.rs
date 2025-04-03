use std::{str::FromStr, thread};

use alpm_buildinfo::{BuildInfoSchema, BuildInfoV1, BuildInfoV2};
use alpm_types::{SchemaVersion, semver_version::Version};
use assert_cmd::Command;
use insta::assert_snapshot;
use rstest::rstest;
use strum::Display;
use tempfile::tempdir;
use testresult::TestResult;

pub const VALID_BUILDINFO_V1_DATA: &str = r#"
builddate = 1
builddir = /build
buildenv = ccache
buildenv = color
format = 1
installed = bar-1.2.3-1-any
installed = beh-2.2.3-4-any
options = lto
options = !strip
packager = Foobar McFooface <foobar@mcfooface.org>
pkgarch = any
pkgbase = foo
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = foo
pkgver = 1:1.0.0-1
"#;

pub const VALID_BUILDINFO_V2_DATA: &str = r#"
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
buildenv = ccache
buildenv = color
format = 2
installed = bar-1.2.3-1-any
installed = beh-2.2.3-4-any
options = lto
options = !strip
packager = Foobar McFooface <foobar@mcfooface.org>
pkgarch = any
pkgbase = foo
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = foo
pkgver = 1:1.0.0-1
"#;

#[derive(Clone, Debug, Default)]
pub struct BuildInfoInput {
    pub format: BuildInfoSchema,
    pub builddate: Option<String>,
    pub builddir: Option<String>,
    pub buildenv: Option<Vec<String>>,
    pub installed: Option<Vec<String>>,
    pub options: Option<Vec<String>>,
    pub packager: Option<String>,
    pub pkgarch: Option<String>,
    pub pkgbase: Option<String>,
    pub pkgbuild_sha256sum: Option<String>,
    pub pkgname: Option<String>,
    pub pkgver: Option<String>,

    // V2 fields
    pub startdir: Option<String>,
    pub buildtool: Option<String>,
    pub buildtoolver: Option<String>,
}

/// Validate the V1 schema.
/// The version is automatically determined from the file
#[test]
fn validate_valid_buildinfov1() -> TestResult {
    let mut cmd = Command::cargo_bin("alpm-buildinfo")?;
    cmd.arg("validate");
    cmd.write_stdin(VALID_BUILDINFO_V1_DATA);
    cmd.assert().success();
    Ok(())
}

/// Validate the V2 schema.
/// The version is automatically determined from the file
#[test]
fn validate_valid_buildinfov2() -> TestResult {
    let mut cmd = Command::cargo_bin("alpm-buildinfo")?;
    cmd.arg("validate");
    cmd.write_stdin(VALID_BUILDINFO_V2_DATA);
    cmd.assert().success();
    Ok(())
}

/// Force a v2 validation on a v1 buildinfo
#[test]
fn wrong_schema_buildinfov1_as_v2() -> TestResult {
    let mut cmd = Command::cargo_bin("alpm-buildinfo")?;
    cmd.args(["validate", "-s", "2"]);
    cmd.write_stdin(VALID_BUILDINFO_V1_DATA);
    cmd.assert().failure().code(1);
    Ok(())
}

/// Force a v1 validation on a v2 buildinfo
#[test]
fn wrong_schema_buildinfov2_as_v1() -> TestResult {
    let mut cmd = Command::cargo_bin("alpm-buildinfo")?;
    cmd.args(["validate", "-s", "1"]);
    cmd.write_stdin(VALID_BUILDINFO_V2_DATA);
    cmd.assert().failure().code(1);
    Ok(())
}

/// Format BUILDINFO as JSON.
#[rstest]
#[case::buildinfov1_as_json(VALID_BUILDINFO_V1_DATA)]
#[case::buildinfov2_as_json(VALID_BUILDINFO_V2_DATA)]
fn format_buildinfo_and_serialize_as_json(#[case] data: &str) -> TestResult {
    let mut cmd = Command::cargo_bin("alpm-buildinfo")?;
    cmd.args(["format", "-p"]);
    cmd.write_stdin(data);
    let cmd = cmd.unwrap();
    let build_info = String::from_utf8_lossy(&cmd.stdout);
    assert_snapshot!(
        thread::current()
            .name()
            .unwrap()
            .to_string()
            .replace("::", "__"),
        build_info.to_string()
    );
    Ok(())
}

#[rstest]
#[case::buildinfov1_all_fields(
    BuildInfoInput {
        format: BuildInfoSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))),
        builddate: Some("1".to_string()),
        builddir: Some("/build".to_string()),
        buildenv: Some(vec!["distcc".to_string(), "color".to_string()]),
        installed: Some(vec!["bar-1.2.3-1-any".to_string(), "beh-2.2.3-4-any".to_string()]),
        options: Some(vec!["lto".to_string(), "!strip".to_string()]),
        packager: Some("Foobar McFooface <foobar@mcfooface.org>".to_string()),
        pkgarch: Some("any".to_string()),
        pkgbase: Some("foo".to_string()),
        pkgbuild_sha256sum: Some("b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c".to_string()),
        pkgname: Some("foo".to_string()),
        pkgver: Some("1:1.0.0-1".to_string()),
        startdir: None,
        buildtool: None,
        buildtoolver: None,
    },
)]
#[case::buildinfov1_optional_fields(
    BuildInfoInput {
        format: BuildInfoSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))),
        builddate: Some("1".to_string()),
        builddir: Some("/build".to_string()),
        buildenv: None,
        installed: None,
        options: None,
        packager: Some("Foobar McFooface <foobar@mcfooface.org>".to_string()),
        pkgarch: Some("any".to_string()),
        pkgbase: Some("foo".to_string()),
        pkgbuild_sha256sum: Some("b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c".to_string()),
        pkgname: Some("foo".to_string()),
        pkgver: Some("1:1.0.0-1".to_string()),
        startdir: None,
        buildtool: None,
        buildtoolver: None,
    },
)]
#[case::buildinfov2_all_fields(
    BuildInfoInput {
        format: BuildInfoSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))),
        builddate: Some("1".to_string()),
        builddir: Some("/build".to_string()),
        buildenv: Some(vec!["distcc".to_string(), "color".to_string()]),
        installed: Some(vec!["bar-1.2.3-1-any".to_string(), "beh-2.2.3-4-any".to_string()]),
        options: Some(vec!["lto".to_string(), "!strip".to_string()]),
        packager: Some("Foobar McFooface <foobar@mcfooface.org>".to_string()),
        pkgarch: Some("any".to_string()),
        pkgbase: Some("foo".to_string()),
        pkgbuild_sha256sum: Some("b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c".to_string()),
        pkgname: Some("foo".to_string()),
        pkgver: Some("1:1.0.0-1".to_string()),
        startdir: Some("/startdir/".to_string()),
        buildtool: Some("devtools".to_string()),
        buildtoolver: Some("1:1.2.1-1-any".to_string()),
    },
)]
#[case::buildinfov2_optional_fields(
    BuildInfoInput {
        format: BuildInfoSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))),
        builddate: Some("1".to_string()),
        builddir: Some("/build".to_string()),
        buildenv: None,
        installed: None,
        options: None,
        packager: Some("Foobar McFooface <foobar@mcfooface.org>".to_string()),
        pkgarch: Some("any".to_string()),
        pkgbase: Some("foo".to_string()),
        pkgbuild_sha256sum: Some("b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c".to_string()),
        pkgname: Some("foo".to_string()),
        pkgver: Some("1:1.0.0-1".to_string()),
        startdir: Some("/startdir/".to_string()),
        buildtool: Some("devtools".to_string()),
        buildtoolver: Some("1:1.2.1-1-any".to_string()),
    },
)]
fn write_buildinfo(#[case] buildinfo_input: BuildInfoInput) -> TestResult {
    // Test the buildinfo write process via environment variables.
    test_write_buildinfo(buildinfo_input.clone(), WriteMode::Environment)?;
    // Test the buildinfo write process via argument flags.
    test_write_buildinfo(buildinfo_input, WriteMode::Cli)
}

/// The mode to use when writing BuildInfo data to output.
#[derive(Display)]
enum WriteMode {
    #[strum(serialize = "env")]
    Environment,
    #[strum(serialize = "cli")]
    Cli,
}

/// Test writing a buildinfo file either via CLI or environment variables.
fn test_write_buildinfo(buildinfo_input: BuildInfoInput, write_mode: WriteMode) -> TestResult {
    let dir = tempdir()?;
    let mut test_name = thread::current()
        .name()
        .unwrap()
        .to_string()
        .replace("::", "__");
    test_name.push_str(&format!("_via_{write_mode}"));

    let mut cmd = Command::cargo_bin("alpm-buildinfo")?;
    cmd.args(["create".to_string(), format!("v{}", buildinfo_input.format)])
        .current_dir(dir.path());

    match write_mode {
        WriteMode::Environment => set_buildinfo_env(&mut cmd, &buildinfo_input),
        WriteMode::Cli => set_buildinfo_args(&mut cmd, &buildinfo_input),
    };

    cmd.assert().success();
    let file = dir.path().join(".BUILDINFO");
    assert!(file.exists());

    let contents = std::fs::read_to_string(&file)?;
    let build_info = match buildinfo_input.format {
        BuildInfoSchema::V1(_) => BuildInfoV1::from_str(&contents)?.to_string(),
        BuildInfoSchema::V2(_) => BuildInfoV2::from_str(&contents)?.to_string(),
    };
    assert_snapshot!(test_name, build_info.to_string());

    let mut cmd = Command::cargo_bin("alpm-buildinfo")?;
    cmd.args([
        "validate".to_string(),
        "-s".to_string(),
        buildinfo_input.format.to_string(),
        file.to_string_lossy().to_string(),
    ]);
    cmd.assert().success();

    Ok(())
}

fn set_buildinfo_args(cmd: &mut Command, input: &BuildInfoInput) {
    if let Some(ref builddate) = input.builddate {
        cmd.args(["--builddate", builddate]);
    }
    if let Some(ref builddir) = input.builddir {
        cmd.args(["--builddir", builddir]);
    }
    if let Some(ref buildenv) = input.buildenv {
        for env in buildenv.iter() {
            cmd.args(["--buildenv", env]);
        }
    }
    if let Some(ref installed) = input.installed {
        for package in installed.iter() {
            cmd.args(["--installed", package]);
        }
    }
    if let Some(ref options) = input.options {
        for option in options.iter() {
            cmd.args(["--options", option]);
        }
    }
    if let Some(ref packager) = input.packager {
        cmd.args(["--packager", packager]);
    }
    if let Some(ref pkgarch) = input.pkgarch {
        cmd.args(["--pkgarch", pkgarch]);
    }
    if let Some(ref pkgbase) = input.pkgbase {
        cmd.args(["--pkgbase", pkgbase]);
    }
    if let Some(ref pkgbuild_sha256sum) = input.pkgbuild_sha256sum {
        cmd.args(["--pkgbuild-sha256sum", pkgbuild_sha256sum]);
    }
    if let Some(ref pkgname) = input.pkgname {
        cmd.args(["--pkgname", pkgname]);
    }
    if let Some(ref pkgver) = input.pkgver {
        cmd.args(["--pkgver", pkgver]);
    }

    if let BuildInfoSchema::V2(_) = input.format {
        if let Some(ref startdir) = input.startdir {
            cmd.args(["--startdir", startdir]);
        }
        if let Some(ref buildtool) = input.buildtool {
            cmd.args(["--buildtool", buildtool]);
        }
        if let Some(ref buildtoolver) = input.buildtoolver {
            cmd.args(["--buildtoolver", buildtoolver]);
        }
    }
}

fn set_buildinfo_env(cmd: &mut Command, input: &BuildInfoInput) {
    if let Some(ref builddate) = input.builddate {
        cmd.env("BUILDINFO_BUILDDATE", builddate);
    }
    if let Some(ref builddir) = input.builddir {
        cmd.env("BUILDINFO_BUILDDIR", builddir);
    }
    if let Some(ref buildenv) = input.buildenv {
        cmd.env("BUILDINFO_BUILDENV", buildenv.join(" "));
    }
    if let Some(ref installed) = input.installed {
        cmd.env("BUILDINFO_INSTALLED", installed.join(" "));
    }
    if let Some(ref options) = input.options {
        cmd.env("BUILDINFO_OPTIONS", options.join(" "));
    }
    if let Some(ref packager) = input.packager {
        cmd.env("BUILDINFO_PACKAGER", packager);
    }
    if let Some(ref pkgarch) = input.pkgarch {
        cmd.env("BUILDINFO_PKGARCH", pkgarch);
    }
    if let Some(ref pkgbase) = input.pkgbase {
        cmd.env("BUILDINFO_PKGBASE", pkgbase);
    }
    if let Some(ref pkgbuild_sha256sum) = input.pkgbuild_sha256sum {
        cmd.env("BUILDINFO_PKGBUILD_SHA256SUM", pkgbuild_sha256sum);
    }
    if let Some(ref pkgname) = input.pkgname {
        cmd.env("BUILDINFO_PKGNAME", pkgname);
    }
    if let Some(ref pkgver) = input.pkgver {
        cmd.env("BUILDINFO_PKGVER", pkgver);
    }

    if let BuildInfoSchema::V2(_) = input.format {
        if let Some(ref startdir) = input.startdir {
            cmd.env("BUILDINFO_STARTDIR", startdir);
        }
        if let Some(ref buildtool) = input.buildtool {
            cmd.env("BUILDINFO_BUILDTOOL", buildtool);
        }
        if let Some(ref buildtoolver) = input.buildtoolver {
            cmd.env("BUILDINFO_BUILDTOOLVER", buildtoolver);
        }
    }
}
