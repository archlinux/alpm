use std::{str::FromStr, thread};

use alpm_buildinfo::BuildInfoV1;
use assert_cmd::Command;
use insta::assert_snapshot;
use rstest::rstest;
use testdir::testdir;
use testresult::TestResult;

pub const VALID_BUILDINFO_DATA: &str = r#"
builddate = 1
builddir = /build
buildenv = envfoo
buildenv = envbar
format = 1
installed = bar-1.2.3-1-any
installed = beh-2.2.3-4-any
options = some_option
options = !other_option
packager = Foobar McFooface <foobar@mcfooface.org>
pkgarch = any
pkgbase = foo
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = foo
pkgver = 1:1.0.0-1
"#;

#[derive(Default)]
pub struct BuildInfoV1Input {
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
}

#[test]
fn validate_valid_buildinfov1() -> TestResult {
    let mut cmd = Command::cargo_bin("alpm-buildinfo")?;
    cmd.args(["validate", "--schema", "1"]);
    cmd.write_stdin(VALID_BUILDINFO_DATA);
    cmd.assert().success();
    Ok(())
}

#[rstest]
#[case::all_fields_with_env(
    BuildInfoV1Input {
        builddate: Some("1".to_string()),
        builddir: Some("/build".to_string()),
        buildenv: Some(vec!["foo".to_string(), "bar".to_string()]),
        installed: Some(vec!["bar-1.2.3-1-any".to_string(), "beh-2.2.3-4-any".to_string()]),
        options: Some(vec!["some_option".to_string(), "!other_option".to_string()]),
        packager: Some("Foobar McFooface <foobar@mcfooface.org>".to_string()),
        pkgarch: Some("any".to_string()),
        pkgbase: Some("foo".to_string()),
        pkgbuild_sha256sum: Some("b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c".to_string()),
        pkgname: Some("foo".to_string()),
        pkgver: Some("1:1.0.0-1".to_string()),
    },
    true,
    true
)]
#[case::optional_fields_with_env(
    BuildInfoV1Input {
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
    },
    true,
    true
)]
#[case::all_fields_with_cli(
    BuildInfoV1Input {
        builddate: Some("1".to_string()),
        builddir: Some("/build".to_string()),
        buildenv: Some(vec!["foo".to_string(), "bar".to_string()]),
        installed: Some(vec!["bar-1.2.3-1-any".to_string(), "beh-2.2.3-4-any".to_string()]),
        options: Some(vec!["some_option".to_string(), "!other_option".to_string()]),
        packager: Some("Foobar McFooface <foobar@mcfooface.org>".to_string()),
        pkgarch: Some("any".to_string()),
        pkgbase: Some("foo".to_string()),
        pkgbuild_sha256sum: Some("b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c".to_string()),
        pkgname: Some("foo".to_string()),
        pkgver: Some("1:1.0.0-1".to_string()),
    },
    true,
    false
)]
#[case::optional_fields_with_cli(
    BuildInfoV1Input {
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
    },
    true,
    false
)]
fn write_buildinfov1(
    #[case] buildinfov1_input: BuildInfoV1Input,
    #[case] should_be_valid: bool,
    #[case] use_env: bool,
) -> TestResult {
    let dir = testdir!();
    let mut cmd = Command::cargo_bin("alpm-buildinfo")?;
    cmd.args(["create", "v1"]).current_dir(dir.clone());
    if use_env {
        set_buildinfo_env(&mut cmd, &buildinfov1_input);
    } else {
        set_buildinfo_args(&mut cmd, &buildinfov1_input);
    }
    if should_be_valid {
        cmd.assert().success();
        let file = dir.join(".BUILDINFO");
        assert!(file.exists());

        let contents = std::fs::read_to_string(&file)?;
        let build_info = BuildInfoV1::from_str(&contents)?;
        assert_snapshot!(
            thread::current().name().unwrap_or("?").to_string(),
            build_info.to_string()
        );

        let mut cmd = Command::cargo_bin("alpm-buildinfo")?;
        cmd.args([
            "validate",
            "--schema",
            "1",
            file.as_os_str().to_str().unwrap(),
        ]);
        cmd.assert().success();
    } else {
        cmd.assert().failure();
    }

    Ok(())
}

fn set_buildinfo_args(cmd: &mut Command, input: &BuildInfoV1Input) {
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
}

fn set_buildinfo_env(cmd: &mut Command, input: &BuildInfoV1Input) {
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
}
