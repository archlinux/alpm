//! Shared helper functions for both, the `integration.rs` tests and the `tester.rs` rust script.

use std::{
    fs::{copy, create_dir_all, write},
    path::Path,
    process::Command,
};

use alpm_compress::compression::CompressionSettings;
use alpm_mtree::create_mtree_v2_from_input_dir;
use alpm_package::{InputDir, OutputDir, Package, PackageCreationConfig, PackageInput};
use alpm_types::{MetadataFileName, SonameLookupDirectory, SonameV2};
use serde::{Deserialize, Serialize};
use testresult::TestResult;

const BUILDINFO_BIN: &str = r#"
format = 2
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
packager = John Doe <john@example.org>
pkgarch = any
pkgbase = bin
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = bin
pkgver = 1:1.0.0-1
"#;

const BUILDINFO_LIB: &str = r#"
format = 2
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
packager = John Doe <john@example.org>
pkgarch = any
pkgbase = lib
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = lib
pkgver = 1:1.0.0-1
"#;

/// Configuration for each test run.
#[derive(Debug, Deserialize, Serialize)]
pub struct SotestConfig {
    /// The name of the shared object we generate.
    /// The soname and filename become lib{libname}.so.1
    pub libname: String,
    /// Lookup directory for soname resolution (prefix + directory).
    pub lookup: SonameLookupDirectory,
    /// The package relation added as a provision for the library's PKGINFO and as a dependency for
    /// the binary's PKGINFO data.
    pub dep: SonameV2,
    /// The string representation of the soname we expect find_dependencies to return
    pub expect_dep: Option<SonameV2>,
    /// The string representation of the soname we expect find_provisions to return
    pub expect_provide: Option<SonameV2>,
}

/// Generate PKGINFO content for the library package.
fn generate_lib_pkginfo(dep: &SonameV2) -> String {
    format!(
        r#"
pkgname = lib
pkgbase = lib
xdata = pkgtype=pkg
pkgver = 1:1.0.0-1
pkgdesc = A project that returns true
url = https://example.org/
builddate = 1
packager = John Doe <john@example.org>
size = 181849963
arch = any
license = GPL-3.0-or-later
provides = {dep}
"#
    )
}

/// Generate PKGINFO content for the binary package.
fn generate_bin_pkginfo(dep: &SonameV2) -> String {
    format!(
        r#"
pkgname = bin
pkgbase = bin
xdata = pkgtype=pkg
pkgver = 1:1.0.0-1
pkgdesc = A project that returns true
url = https://example.org/
builddate = 1
packager = John Doe <john@example.org>
size = 181849963
arch = any
license = GPL-3.0-or-later
depend = {dep}
"#
    )
}

/// Set up and compile the C project using Meson.
pub fn setup_lib(config: &SotestConfig, path: &Path, test_files_dir: &Path) -> TestResult {
    let status = Command::new("meson")
        .arg("setup")
        .arg(format!("-Dlibname={}", config.libname))
        .arg(path.join("build"))
        .arg(test_files_dir.to_string_lossy().to_string())
        .output()?;

    if !status.status.success() {
        panic!("failed to setup sotest c project");
    }

    let status = Command::new("meson")
        .arg("compile")
        .arg("-C")
        .arg(path.join("build"))
        .output()?;

    if !status.status.success() {
        panic!("failed to compile sotest c project");
    }

    Ok(())
}

/// Create a package for the binary.
pub fn create_bin_package(path: &Path, config: &SotestConfig) -> TestResult<Package> {
    let input_dir = path.join("input_bin");
    create_dir_all(&input_dir)?;
    let input_dir = InputDir::new(input_dir)?;
    let output_dir = OutputDir::new(path.join("output_bin"))?;

    write(
        input_dir.join(MetadataFileName::PackageInfo.as_ref()),
        generate_bin_pkginfo(&config.dep),
    )?;
    write(
        input_dir.join(MetadataFileName::BuildInfo.as_ref()),
        BUILDINFO_BIN,
    )?;

    create_dir_all(input_dir.join("usr/bin"))?;
    copy(path.join("build/sotest"), input_dir.join("usr/bin/sotest"))?;

    create_mtree_v2_from_input_dir(&input_dir)?;

    let package_input: PackageInput = input_dir.try_into()?;
    let config =
        PackageCreationConfig::new(package_input, output_dir, CompressionSettings::default())?;

    Ok(Package::try_from(&config)?)
}

/// Create a package for the binary.
pub fn create_lib_package(path: &Path, config: &SotestConfig) -> TestResult<Package> {
    let input_dir = path.join("input_lib");
    create_dir_all(&input_dir)?;
    let input_dir = InputDir::new(input_dir)?;
    let output_dir = OutputDir::new(path.join("output_lib"))?;

    write(
        input_dir.join(MetadataFileName::PackageInfo.as_ref()),
        generate_lib_pkginfo(&config.dep),
    )?;
    write(
        input_dir.join(MetadataFileName::BuildInfo.as_ref()),
        BUILDINFO_LIB,
    )?;

    create_dir_all(input_dir.join("usr/lib"))?;
    copy(
        path.join(format!("build/lib{}.so", config.libname)),
        input_dir.join(format!("usr/lib/lib{}.so", config.libname)),
    )?;

    create_mtree_v2_from_input_dir(&input_dir)?;

    let package_input: PackageInput = input_dir.try_into()?;
    let config =
        PackageCreationConfig::new(package_input, output_dir, CompressionSettings::default())?;

    Ok(Package::try_from(&config)?)
}
