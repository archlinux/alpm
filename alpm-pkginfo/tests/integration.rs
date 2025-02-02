use std::{str::FromStr, thread};

use alpm_pkginfo::{PackageInfoV1, PackageInfoV2};
use assert_cmd::Command;
use insta::assert_snapshot;
use rstest::rstest;
use tempfile::tempdir;
use testresult::TestResult;

pub const VALID_PKGINFO_V1_DATA: &str = r#"
pkgname = example
pkgbase = example
pkgver = 1:1.0.0-1
pkgdesc = A project that does something
url = https://example.org/
builddate = 1729181726
packager = John Doe <john@example.org>
size = 181849963
arch = any
license = GPL-3.0-or-later
license = LGPL-3.0-or-later
replaces = other-package>0.9.0-3
group = package-group
group = other-package-group
conflict = conflicting-package<1.0.0
conflict = other-conflicting-package<1.0.0
provides = some-component
provides = some-other-component=1:1.0.0-1
backup = etc/example/config.toml
backup = etc/example/other-config.txt
depend = glibc
depend = gcc-libs
optdepend = python: for special-python-script.py
optdepend = ruby: for special-ruby-script.rb
makedepend = cmake
makedepend = python-sphinx
checkdepend = extra-test-tool
checkdepend = other-extra-test-tool
"#;

pub const VALID_PKGINFO_V2_DATA: &str = r#"
pkgname = example
pkgbase = example
xdata = pkgtype=pkg
pkgver = 1:1.0.0-1
pkgdesc = A project that does something
url = https://example.org/
builddate = 1729181726
packager = John Doe <john@example.org>
size = 181849963
arch = any
license = GPL-3.0-or-later
license = LGPL-3.0-or-later
replaces = other-package>0.9.0-3
group = package-group
group = other-package-group
conflict = conflicting-package<1.0.0
conflict = other-conflicting-package<1.0.0
provides = some-component
provides = some-other-component=1:1.0.0-1
backup = etc/example/config.toml
backup = etc/example/other-config.txt
depend = glibc
depend = gcc-libs
optdepend = python: for special-python-script.py
optdepend = ruby: for special-ruby-script.rb
makedepend = cmake
makedepend = python-sphinx
checkdepend = extra-test-tool
checkdepend = other-extra-test-tool
"#;

#[derive(Default)]
pub struct PackageInfoInput {
    pub pkgname: String,
    pub pkgbase: String,
    pub pkgver: String,
    pub pkgdesc: String,
    pub url: String,
    pub builddate: String,
    pub packager: String,
    pub size: String,
    pub arch: String,
    pub license: Option<Vec<String>>,
    pub replaces: Option<Vec<String>>,
    pub group: Option<Vec<String>>,
    pub conflict: Option<Vec<String>>,
    pub provides: Option<Vec<String>>,
    pub backup: Option<Vec<String>>,
    pub depend: Option<Vec<String>>,
    pub optdepend: Option<Vec<String>>,
    pub makedepend: Option<Vec<String>>,
    pub checkdepend: Option<Vec<String>>,

    // V2 fields
    pub xdata: Option<Vec<String>>,
}

/// Validate the V1 schema.
/// The version is automatically determined from the file
#[test]
fn validate_valid_pkginfov1() -> TestResult {
    let mut cmd = Command::cargo_bin("alpm-pkginfo")?;
    cmd.arg("validate");
    cmd.write_stdin(VALID_PKGINFO_V1_DATA);
    cmd.assert().success();
    Ok(())
}

/// Validate the V2 schema.
/// The version is automatically determined from the file
#[test]
fn validate_valid_pkginfov2() -> TestResult {
    let mut cmd = Command::cargo_bin("alpm-pkginfo")?;
    cmd.arg("validate");
    cmd.write_stdin(VALID_PKGINFO_V2_DATA);
    cmd.assert().success();
    Ok(())
}

/// Force a v2 validation on a v1 pkginfo
#[test]
fn validate_pkginfov1_as_v2() -> TestResult {
    let mut cmd = Command::cargo_bin("alpm-pkginfo")?;
    cmd.args(["validate"]);
    cmd.write_stdin(VALID_PKGINFO_V1_DATA);
    cmd.assert().success();
    Ok(())
}

/// Force a v1 validation on a v2 pkginfo
#[test]
fn wrong_schema_pkginfov2_as_v1() -> TestResult {
    let mut cmd = Command::cargo_bin("alpm-pkginfo")?;
    cmd.args(["validate"]);
    cmd.write_stdin(VALID_PKGINFO_V2_DATA);
    cmd.assert().success();
    Ok(())
}

/// Format PKGINFO as JSON.
#[rstest]
#[case::pkginfov1_as_json(VALID_PKGINFO_V1_DATA)]
#[case::pkginfov2_as_json(VALID_PKGINFO_V2_DATA)]
fn format_pkginfo_and_serialize_as_json(#[case] data: &str) -> TestResult {
    let mut cmd = Command::cargo_bin("alpm-pkginfo")?;
    cmd.args(["format", "-p"]);
    cmd.write_stdin(data);
    let cmd = cmd.unwrap();
    let pkg_info = String::from_utf8_lossy(&cmd.stdout);
    assert_snapshot!(
        thread::current().name().unwrap().to_string(),
        pkg_info.to_string()
    );
    Ok(())
}

/// Return a valid PKGINFO file with all fields filled in.
fn pkginfo_all_fields(xdata: Option<Vec<String>>) -> PackageInfoInput {
    PackageInfoInput {
        pkgname: "example".to_string(),
        pkgbase: "example".to_string(),
        pkgver: "1:1.0.0-1".to_string(),
        pkgdesc: "A project that does something".to_string(),
        url: "https://example.org/".to_string(),
        builddate: "1729181726".to_string(),
        packager: "John Doe <john@example.org>".to_string(),
        size: "181849963".to_string(),
        arch: "any".to_string(),
        license: Some(vec![
            "GPL-3.0-or-later".to_string(),
            "LGPL-3.0-or-later".to_string(),
        ]),
        replaces: Some(vec!["other-package>0.9.0-3".to_string()]),
        group: Some(vec![
            "package-group".to_string(),
            "other-package-group".to_string(),
        ]),
        conflict: Some(vec![
            "conflicting-package<1.0.0".to_string(),
            "other-conflicting-package<1.0.0".to_string(),
        ]),
        provides: Some(vec![
            "some-component".to_string(),
            "some-other-component=1:1.0.0-1".to_string(),
        ]),
        backup: Some(vec![
            "etc/example/config.toml".to_string(),
            "etc/example/other-config.txt".to_string(),
        ]),
        depend: Some(vec!["glibc".to_string(), "gcc-libs".to_string()]),
        optdepend: Some(vec![
            "python: for special-python-script.py".to_string(),
            "ruby: for special-ruby-script.rb".to_string(),
        ]),
        makedepend: Some(vec!["cmake".to_string(), "python-sphinx".to_string()]),
        checkdepend: Some(vec![
            "extra-test-tool".to_string(),
            "other-extra-test-tool".to_string(),
        ]),
        xdata,
    }
}

/// Return a valid PKGINFO file with only the required fields filled in.
fn pkginfo_optional_fields(xdata: Option<Vec<String>>) -> PackageInfoInput {
    PackageInfoInput {
        pkgname: "example".to_string(),
        pkgbase: "example".to_string(),
        pkgver: "1:1.0.0-1".to_string(),
        pkgdesc: "A project that does something".to_string(),
        url: "https://example.org/".to_string(),
        builddate: "1729181726".to_string(),
        packager: "John Doe <john@example.org>".to_string(),
        size: "181849963".to_string(),
        arch: "any".to_string(),
        license: None,
        replaces: None,
        group: None,
        conflict: None,
        provides: None,
        backup: None,
        depend: None,
        optdepend: None,
        makedepend: None,
        checkdepend: None,
        xdata,
    }
}

#[rstest]
#[case::pkginfov1_all_fields(pkginfo_all_fields(None))]
#[case::pkginfov1_optional_fields_with_cli(pkginfo_optional_fields(None))]
#[case::pkginfov2_all_fields(
    pkginfo_all_fields(Some( vec!["pkgtype=pkg".to_string()]))
)]
#[case::pkginfov2_optional_fields(
    pkginfo_optional_fields(Some( vec!["pkgtype=pkg".to_string()]))
)]
fn write_pkginfo_via_cli(#[case] pkginfo_input: PackageInfoInput) -> TestResult {
    test_write_pkginfo(pkginfo_input, false)
}

#[rstest]
#[case::pkginfov1_all_fields(pkginfo_all_fields(None))]
#[case::pkginfov1_optional_fields(pkginfo_optional_fields(None))]
#[case::pkginfov2_all_fields(
    pkginfo_all_fields(Some( vec!["pkgtype=pkg".to_string()]))
)]
#[case::pkginfov2_optional_fields(
    pkginfo_optional_fields(Some( vec!["pkgtype=pkg".to_string()]))
)]
fn write_pkginfo_via_env(#[case] pkginfo_input: PackageInfoInput) -> TestResult {
    test_write_pkginfo(pkginfo_input, true)
}

/// Test writing a pkginfo file either via CLI or environment variables.
fn test_write_pkginfo(pkginfo_input: PackageInfoInput, use_env: bool) -> TestResult {
    let test_name = thread::current().name().unwrap().to_string();

    // Create a temporary directory for the test
    let dir = tempdir()?;

    // Write the PKGINFO file
    let mut cmd = Command::cargo_bin("alpm-pkginfo")?;
    cmd.args([
        "create".to_string(),
        format!("v{}", if pkginfo_input.xdata.is_some() { 2 } else { 1 }),
    ])
    .current_dir(dir.path());
    if use_env {
        set_pkginfo_env(&mut cmd, &pkginfo_input);
    } else {
        set_pkginfo_args(&mut cmd, &pkginfo_input);
    }
    cmd.assert().success();
    let file = dir.path().join(".PKGINFO");
    assert!(file.exists());

    // Validate the contents of the PKGINFO file
    let contents = std::fs::read_to_string(&file)?;
    let pkg_info = if pkginfo_input.xdata.is_some() {
        PackageInfoV2::from_str(&contents)?.to_string()
    } else {
        PackageInfoV1::from_str(&contents)?.to_string()
    };
    assert_snapshot!(test_name, pkg_info.to_string());

    // Validate the PKGINFO file
    let mut cmd = Command::cargo_bin("alpm-pkginfo")?;
    cmd.args(["validate".to_string(), file.to_string_lossy().to_string()]);
    cmd.assert().success();

    Ok(())
}

fn set_pkginfo_args(cmd: &mut Command, input: &PackageInfoInput) {
    cmd.args(["--pkgname", &input.pkgname]);
    cmd.args(["--pkgbase", &input.pkgbase]);
    cmd.args(["--pkgver", &input.pkgver]);
    cmd.args(["--pkgdesc", &input.pkgdesc]);
    cmd.args(["--url", &input.url]);
    cmd.args(["--builddate", &input.builddate]);
    cmd.args(["--packager", &input.packager]);
    cmd.args(["--size", &input.size]);
    cmd.args(["--arch", &input.arch]);
    if let Some(ref license) = input.license {
        for license in license.iter() {
            cmd.args(["--license", license]);
        }
    }
    if let Some(ref replaces) = input.replaces {
        for package in replaces.iter() {
            cmd.args(["--replaces", package]);
        }
    }
    if let Some(ref group) = input.group {
        for group in group.iter() {
            cmd.args(["--group", group]);
        }
    }
    if let Some(ref conflict) = input.conflict {
        for package in conflict.iter() {
            cmd.args(["--conflict", package]);
        }
    }
    if let Some(ref provides) = input.provides {
        for package in provides.iter() {
            cmd.args(["--provides", package]);
        }
    }
    if let Some(ref backup) = input.backup {
        for file in backup.iter() {
            cmd.args(["--backup", file]);
        }
    }
    if let Some(ref depend) = input.depend {
        for package in depend.iter() {
            cmd.args(["--depend", package]);
        }
    }
    if let Some(ref optdepend) = input.optdepend {
        for package in optdepend.iter() {
            cmd.args(["--optdepend", package]);
        }
    }
    if let Some(ref makedepend) = input.makedepend {
        for package in makedepend.iter() {
            cmd.args(["--makedepend", package]);
        }
    }
    if let Some(ref checkdepend) = input.checkdepend {
        for package in checkdepend.iter() {
            cmd.args(["--checkdepend", package]);
        }
    }
    if let Some(ref xdata) = input.xdata {
        for data in xdata.iter() {
            cmd.args(["--xdata", data]);
        }
    }
}

fn set_pkginfo_env(cmd: &mut Command, input: &PackageInfoInput) {
    cmd.env("PKGINFO_PKGNAME", &input.pkgname);
    cmd.env("PKGINFO_PKGBASE", &input.pkgbase);
    cmd.env("PKGINFO_PKGVER", &input.pkgver);
    cmd.env("PKGINFO_PKGDESC", &input.pkgdesc);
    cmd.env("PKGINFO_URL", &input.url);
    cmd.env("PKGINFO_BUILDDATE", &input.builddate);
    cmd.env("PKGINFO_PACKAGER", &input.packager);
    cmd.env("PKGINFO_SIZE", &input.size);
    cmd.env("PKGINFO_ARCH", &input.arch);
    if let Some(ref license) = input.license {
        cmd.env("PKGINFO_LICENSE", license.join(" "));
    }
    if let Some(ref replaces) = input.replaces {
        cmd.env("PKGINFO_REPLACES", replaces.join(" "));
    }
    if let Some(ref group) = input.group {
        cmd.env("PKGINFO_GROUP", group.join(" "));
    }
    if let Some(ref conflict) = input.conflict {
        cmd.env("PKGINFO_CONFLICT", conflict.join(" "));
    }
    if let Some(ref provides) = input.provides {
        cmd.env("PKGINFO_PROVIDES", provides.join(" "));
    }
    if let Some(ref backup) = input.backup {
        cmd.env("PKGINFO_BACKUP", backup.join(" "));
    }
    if let Some(ref depend) = input.depend {
        cmd.env("PKGINFO_DEPEND", depend.join(" "));
    }
    if let Some(ref optdepend) = input.optdepend {
        cmd.env("PKGINFO_OPTDEPEND", optdepend.join(","));
    }
    if let Some(ref makedepend) = input.makedepend {
        cmd.env("PKGINFO_MAKEDEPEND", makedepend.join(" "));
    }
    if let Some(ref checkdepend) = input.checkdepend {
        cmd.env("PKGINFO_CHECKDEPEND", checkdepend.join(" "));
    }
    if let Some(ref xdata) = input.xdata {
        cmd.env("PKGINFO_XDATA", xdata.join(" "));
    }
}
