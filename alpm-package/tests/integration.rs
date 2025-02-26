use std::{
    fs::{File, create_dir_all},
    io::{Read, Write},
    os::unix::fs::symlink,
    process::{Command, Stdio},
};

use alpm_common::{MetadataFileName, relative_files};
use alpm_package::{Package, PackageCompression, PackageInput, PackagePipeline};
use rstest::rstest;
use tempfile::TempDir;
use testdir::testdir;
use testresult::TestResult;

pub const VALID_BUILDINFO_V2_DATA: &str = r#"
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
buildenv = envfoo
buildenv = envbar
format = 2
installed = bar-1.2.3-1-any
installed = beh-2.2.3-4-any
options = some_option
options = !other_option
packager = Foobar McFooface <foobar@mcfooface.org>
pkgarch = any
pkgbase = example
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = example
pkgver = 1:1.0.0-1
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

#[rstest]
#[case(PackageCompression::None)]
fn create_package_from_input(#[case] compression: PackageCompression) -> TestResult {
    let input_dir = TempDir::new()?;

    // Create .BUILDINFO file
    let mut file = File::create(input_dir.path().join(MetadataFileName::BuildInfo.as_ref()))?;
    write!(file, "{}", VALID_BUILDINFO_V2_DATA)?;

    // Create .PKGINFO file
    let mut file = File::create(
        input_dir
            .path()
            .join(MetadataFileName::PackageInfo.as_ref()),
    )?;
    write!(file, "{}", VALID_PKGINFO_V2_DATA)?;

    // Create dummy directory structure
    create_dir_all(input_dir.path().join("foo/bar/baz"))?;
    // Create dummy text file
    let mut output = File::create(input_dir.path().join("foo/beh.txt"))?;
    write!(output, "test")?;
    // Create relative symlink to actual text file
    symlink(
        "../../beh.txt",
        input_dir.path().join("foo/bar/baz/beh.txt"),
    )?;

    // Collect all files in input dir
    let collected_files = relative_files(&input_dir, &[])?;
    eprintln!("{:?}", collected_files);
    let collected_files_string: Vec<String> = collected_files
        .iter()
        .map(|file| file.to_string_lossy().to_string())
        .collect();
    let all_files = collected_files_string.join("\n");

    // Create mtree file
    let mut command = Command::new("bsdtar");
    command
        .current_dir(input_dir.path())
        .env("LANG", "C")
        .args([
            "--create",
            "--no-recursion",
            "--file",
            "-",
            "--format=mtree",
            "--options",
            "!all,use-set,type,uid,gid,mode,time,size,sha256,link",
            // "--null",
            "--files-from",
            "-",
            "--exclude",
            ".MTREE",
        ]);
    let mut command_child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut stdin = command_child.stdin.take().expect("handle present");
    let handle = std::thread::spawn(move || stdin.write_all(all_files.as_bytes()));
    let _handle_result = handle.join();

    let mut stdout = command_child.stdout.take().expect("handle present");
    let mut bsdtar_stdout = String::new();
    stdout.read_to_string(&mut bsdtar_stdout)?;
    eprintln!("bsdtar stdout: {}", bsdtar_stdout);

    // Create .MTREE file
    let mut file = File::create(input_dir.path().join(MetadataFileName::Mtree.as_ref()))?;
    write!(file, "{}", bsdtar_stdout)?;

    let output_dir = testdir!();
    let input = PackageInput::try_from(input_dir.path())?;
    eprintln!("input: {input:#?}");
    let pipeline = PackagePipeline::new(compression, input, output_dir);
    eprintln!("pipeline: {pipeline:#?}");

    let package = Package::try_from(pipeline)?;

    eprintln!("package: {package:#?}");
    Ok(())
}
