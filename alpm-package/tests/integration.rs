use std::{
    fs::{File, create_dir, create_dir_all},
    io::Write,
    os::unix::fs::symlink,
};

use alpm_mtree::mtree_v2_from_input_dir;
use alpm_package::{
    CompressionSettings,
    Package,
    PackageInput,
    PackagePipeline,
    compression::{Bzip2Compression, GzipCompression, XzCompression, ZstandardCompression},
};
use alpm_types::MetadataFileName;
use log::{LevelFilter, debug};
use rstest::rstest;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use testdir::testdir;
use testresult::TestResult;

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

/// Initializes a global [`TermLogger`].
fn init_logger() -> TestResult {
    if TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .is_err()
    {
        debug!("Not initializing another logger, as one is initialized already.");
    }

    Ok(())
}

#[rstest]
#[case::compress_with_bzip2(CompressionSettings::Bzip2 { compression_level: Bzip2Compression::default() })]
#[case::compress_with_gzip(CompressionSettings::Gzip { compression_level: GzipCompression::default() })]
#[case::compress_with_xz(CompressionSettings::Xz { compression_level: XzCompression::default() })]
#[case::compress_with_zstandard(CompressionSettings::Zstandard { compression_level: ZstandardCompression::default() })]
#[case::no_compression(CompressionSettings::None)]
fn create_package_from_input(#[case] compression: CompressionSettings) -> TestResult {
    init_logger()?;

    // let temp_dir = TempDir::new()?;
    // let test_dir = temp_dir.into_path();
    let test_dir = testdir!();
    let input_dir = test_dir.as_path().join("input");
    let output_dir = test_dir.as_path().join("output");
    create_dir(output_dir.as_path())?;
    create_dir(input_dir.as_path())?;

    // Create .BUILDINFO file
    let mut file = File::create(
        input_dir
            .as_path()
            .join(MetadataFileName::BuildInfo.as_ref()),
    )?;
    write!(file, "{}", VALID_BUILDINFO_V2_DATA)?;

    // Create .PKGINFO file
    let mut file = File::create(
        input_dir
            .as_path()
            .join(MetadataFileName::PackageInfo.as_ref()),
    )?;
    write!(file, "{}", VALID_PKGINFO_V2_DATA)?;

    // Create dummy directory structure
    create_dir_all(input_dir.as_path().join("foo/bar/baz"))?;
    // Create dummy text file
    let mut output = File::create(input_dir.as_path().join("foo/beh.txt"))?;
    write!(output, "test")?;
    // Create relative symlink to actual text file
    symlink(
        "../../beh.txt",
        input_dir.as_path().join("foo/bar/baz/beh.txt"),
    )?;

    // Create .MTREE file.
    let _ = mtree_v2_from_input_dir(input_dir.as_path())?;

    // Create PackageInput
    let input = PackageInput::try_from(input_dir.as_path())?;
    eprintln!("input: {input:#?}");
    let pipeline = PackagePipeline::new(compression, input, output_dir);
    eprintln!("pipeline: {pipeline:#?}");

    let package = Package::try_from(pipeline)?;

    eprintln!("package: {package:#?}");
    Ok(())
}
