// SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Error as IOError;
use std::path::PathBuf;

use assert_cmd::Command;

use testdir::testdir;

use testresult::TestResult;

use rstest::rstest;

mod common;
use common::valid_buildinfov1;
use common::BuildInfoV1Input;

#[rstest]
fn validate_valid_buildinfov1(valid_buildinfov1: Result<PathBuf, IOError>) -> TestResult {
    let mut cmd = Command::cargo_bin("alpm-buildinfo")?;
    cmd.args([
        "validate",
        valid_buildinfov1?.as_path().as_os_str().to_str().unwrap(),
    ]);
    cmd.assert().success();

    Ok(())
}

#[rstest]
#[case(BuildInfoV1Input{
    builddate: (Some("1".to_string()), true),
    builddir: (Some("/build".to_string()), true),
    buildenv: (Some(vec!["foo".to_string(), "bar".to_string()]), true),
    installed: (Some(vec!["bar-1.2.3-1-any".to_string(), "beh-2.2.3-4-any".to_string()]), true),
    options: (Some(vec!["some_option".to_string(), "!other_option".to_string()]), true),
    packager: (Some("Foobar McFooface <foobar@mcfooface.org>".to_string()), true),
    pkgarch: (Some("any".to_string()), true),
    pkgbase: (Some("foo".to_string()), true),
    pkgbuild_sha256sum: (Some("b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c".to_string()), true),
    pkgname: (Some("foo".to_string()), true),
    pkgver: (Some("1:1.0.0-1".to_string()), true),
    should_be_valid: true,
})]
#[case(BuildInfoV1Input{
    builddate: (Some("1".to_string()), true),
    builddir: (Some("/build".to_string()), true),
    buildenv: (None, true),
    installed: (None, true),
    options: (None, true),
    packager: (Some("Foobar McFooface <foobar@mcfooface.org>".to_string()), true),
    pkgarch: (Some("any".to_string()), true),
    pkgbase: (Some("foo".to_string()), true),
    pkgbuild_sha256sum: (Some("b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c".to_string()), true),
    pkgname: (Some("foo".to_string()), true),
    pkgver: (Some("1:1.0.0-1".to_string()), true),
    should_be_valid: true,
})]
#[case(BuildInfoV1Input{
    builddate: (Some("1".to_string()), false),
    builddir: (Some("/build".to_string()), false),
    buildenv: (Some(vec!["foo".to_string(), "bar".to_string()]), false),
    installed: (Some(vec!["bar-1.2.3-1-any".to_string(), "beh-2.2.3-4-any".to_string()]), false),
    options: (Some(vec!["some_option".to_string(), "!other_option".to_string()]), false),
    packager: (Some("Foobar McFooface <foobar@mcfooface.org>".to_string()), false),
    pkgarch: (Some("any".to_string()), false),
    pkgbase: (Some("foo".to_string()), false),
    pkgbuild_sha256sum: (Some("b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c".to_string()), false),
    pkgname: (Some("foo".to_string()), false),
    pkgver: (Some("1:1.0.0-1".to_string()), false),
    should_be_valid: true,
})]
#[case(BuildInfoV1Input{
    builddate: (Some("1".to_string()), false),
    builddir: (Some("/build".to_string()), false),
    buildenv: (None, false),
    installed: (None, false),
    options: (None, false),
    packager: (Some("Foobar McFooface <foobar@mcfooface.org>".to_string()), false),
    pkgarch: (Some("any".to_string()), false),
    pkgbase: (Some("foo".to_string()), false),
    pkgbuild_sha256sum: (Some("b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c".to_string()), false),
    pkgname: (Some("foo".to_string()), false),
    pkgver: (Some("1:1.0.0-1".to_string()), false),
    should_be_valid: true,
})]
#[case(BuildInfoV1Input::default())]
fn write_buildinfov1(#[case] buildinfov1_input: BuildInfoV1Input) -> TestResult {
    let dir = testdir!();
    let mut cmd = Command::cargo_bin("alpm-buildinfo")?;
    cmd.args(["create", "v1"]).current_dir(dir.clone());

    if let (Some(input), env) = buildinfov1_input.builddate {
        if env {
            cmd.env("BUILDINFO_BUILDDATE", input);
        } else {
            cmd.args(&["--builddate", &input]);
        }
    }

    if let (Some(input), env) = buildinfov1_input.builddir {
        if env {
            cmd.env("BUILDINFO_BUILDDIR", input);
        } else {
            cmd.args(&["--builddir", &input]);
        }
    }

    if let (Some(input), env) = buildinfov1_input.buildenv {
        if env {
            cmd.env("BUILDINFO_BUILDENV", input.join(" "));
        } else {
            for input in input.iter() {
                cmd.args(&["--buildenv", &input]);
            }
        }
    }

    if let (Some(input), env) = buildinfov1_input.installed {
        if env {
            cmd.env("BUILDINFO_INSTALLED", input.join(" "));
        } else {
            for input in input.iter() {
                cmd.args(&["--installed", &input]);
            }
        }
    }

    if let (Some(input), env) = buildinfov1_input.options {
        if env {
            cmd.env("BUILDINFO_OPTIONS", input.join(" "));
        } else {
            for input in input.iter() {
                cmd.args(&["--options", &input]);
            }
        }
    }

    if let (Some(input), env) = buildinfov1_input.packager {
        if env {
            cmd.env("BUILDINFO_PACKAGER", input);
        } else {
            cmd.args(&["--packager", &input]);
        }
    }

    if let (Some(input), env) = buildinfov1_input.pkgarch {
        if env {
            cmd.env("BUILDINFO_PKGARCH", input);
        } else {
            cmd.args(&["--pkgarch", &input]);
        }
    }

    if let (Some(input), env) = buildinfov1_input.pkgbase {
        if env {
            cmd.env("BUILDINFO_PKGBASE", input);
        } else {
            cmd.args(&["--pkgbase", &input]);
        }
    }

    if let (Some(input), env) = buildinfov1_input.pkgbuild_sha256sum {
        if env {
            cmd.env("BUILDINFO_PKGBUILD_SHA256SUM", input);
        } else {
            cmd.args(&["--pkgbuild-sha256sum", &input]);
        }
    }

    if let (Some(input), env) = buildinfov1_input.pkgname {
        if env {
            cmd.env("BUILDINFO_PKGNAME", input);
        } else {
            cmd.args(&["--pkgname", &input]);
        }
    }

    if let (Some(input), env) = buildinfov1_input.pkgver {
        if env {
            cmd.env("BUILDINFO_PKGVER", input);
        } else {
            cmd.args(&["--pkgver", &input]);
        }
    }

    if buildinfov1_input.should_be_valid {
        cmd.assert().success();
        let file = dir.join(".BUILDINFO");
        assert!(file.exists());
        let mut cmd = Command::cargo_bin("alpm-buildinfo")?;
        cmd.args(["validate", file.as_os_str().to_str().unwrap()]);
        cmd.assert().success();
    } else {
        cmd.assert().failure();
    }

    Ok(())
}
