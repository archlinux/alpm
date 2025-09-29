#!/usr/bin/env rust-script
//!
//! This integration test verifies that shared object dependencies and provisions
//! are correctly identified in packages.
//!
//! It does so by creating a simple C project with Meson that builds a shared library
//! and a binary that depends on it. The test then creates packages for both the library
//! and the binary, and checks that the binary's dependencies include the library's soname,
//! and that the library's provisions include its own soname.
//!
//! It can be also run as a standalone program via [`rust-script`], taking an optional
//! JSON-encoded configuration as a command line argument:
//!
//! ```sh
//! ./integration.rs
//!
//! # or with a custom configuration:
//!
//! ./integration.rs '{"libname":"sotest", ...}'
//! ```
//!
//! [`rust-script`]: https://github.com/fornwall/rust-script
//!
//! ```cargo
//! [dependencies]
//! assert_cmd = "2"
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! tempfile = "3"
//! testresult = "0.4"
//! rstest = "0.17"
//! alpm-compress = { path = "../../alpm-compress" }
//! alpm-package = { path = "../../alpm-package" }
//! alpm-mtree = { path = "../../alpm-mtree" }
//! alpm-types = { path = "../../alpm-types" }
//! alpm-soname = { path = "../../alpm-soname" }
//! ```

use std::{
    env,
    fs::{copy, create_dir_all, write},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use alpm_compress::compression::CompressionSettings;
use alpm_mtree::create_mtree_v2_from_input_dir;
use alpm_package::{InputDir, OutputDir, Package, PackageCreationConfig, PackageInput};
use alpm_soname::{ElfSonames, extract_elf_sonames, find_dependencies, find_provisions};
use alpm_types::{MetadataFileName, Soname, SonameLookupDirectory, SonameV2};
use assert_cmd::{assert::OutputAssertExt, cargo::CommandCargoExt};
use rstest::rstest;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;
use testresult::{TestError, TestResult};

const MESON_FILES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/test_files");

const BUILDINFO_BIN: &str = r#"
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
format = 2
packager = John Doe <john@example.org>
pkgarch = any
pkgbase = bin
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = bin
pkgver = 1:1.0.0-1
"#;

const BUILDINFO_LIB: &str = r#"
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
format = 2
packager = John Doe <john@example.org>
pkgarch = any
pkgbase = lib
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = lib
pkgver = 1:1.0.0-1
"#;

/// Configuration for each test run.
#[derive(Debug, Deserialize, Serialize)]
struct SotestConfig {
    /// The name of the shared object we generate.
    /// The soname and filename become lib{libname}.so.1
    libname: String,
    /// Lookup directory for soname resolution (prefix + directory).
    lookup: SonameLookupDirectory,
    /// The package relation added as a provision for the library's PKGINFO and as a dependency for
    /// the binary's PKGINFO data.
    dep: SonameV2,
    /// The string representation of the soname we expect find_dependencies to return
    expect_dep: Option<SonameV2>,
    /// The string representation of the soname we expect find_provisions to return
    expect_provide: Option<SonameV2>,
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
fn setup_lib(config: &SotestConfig, path: &Path, test_files_dir: &Path) -> TestResult {
    let status = Command::new("meson")
        .arg("setup")
        .arg(format!("-Dlibname={}", config.libname))
        .arg(path.join("build"))
        .arg(test_files_dir.to_string_lossy().to_string())
        .output()?;

    if !status.status.success() {
        return Err(TestError::from("failed to setup sotest c project"));
    }

    let status = Command::new("meson")
        .arg("compile")
        .arg("-C")
        .arg(path.join("build"))
        .output()?;

    if !status.status.success() {
        return Err(TestError::from("failed to compile sotest c project"));
    }

    Ok(())
}

/// Create a package for the binary.
fn create_bin_package(path: &Path, config: &SotestConfig) -> TestResult<Package> {
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
    let config = PackageCreationConfig::new(
        package_input,
        output_dir,
        Some(CompressionSettings::default()),
    )?;

    Ok(Package::try_from(&config)?)
}

/// Create a package for the binary.
fn create_lib_package(path: &Path, config: &SotestConfig) -> TestResult<Package> {
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
    let config = PackageCreationConfig::new(
        package_input,
        output_dir,
        Some(CompressionSettings::default()),
    )?;

    Ok(Package::try_from(&config)?)
}

/// Invoke the CLI to get provisions.
fn get_provisions_via_cli(pkg: &Path, lookup: &SonameLookupDirectory) -> TestResult<Vec<SonameV2>> {
    let mut cmd = Command::cargo_bin("alpm-soname")?;
    let output = cmd
        .args([
            "get-provisions",
            "--output-format",
            "json",
            "--lookup-dir",
            &lookup.to_string(),
            pkg.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    Ok(serde_json::from_slice(&output)?)
}

/// Invoke the CLI to get dependencies.
fn get_dependencies_via_cli(
    pkg: &Path,
    lookup: &SonameLookupDirectory,
) -> TestResult<Vec<SonameV2>> {
    let mut cmd = Command::cargo_bin("alpm-soname")?;
    let output = cmd
        .args([
            "get-dependencies",
            "--output-format",
            "json",
            "--lookup-dir",
            &lookup.to_string(),
            pkg.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    Ok(serde_json::from_slice(&output)?)
}

/// Invoke the CLI to get raw dependencies.
fn get_raw_dependencies_via_cli(pkg: &Path) -> TestResult<Vec<Soname>> {
    let mut cmd = Command::cargo_bin("alpm-soname")?;
    let output = cmd
        .args([
            "get-raw-dependencies",
            "--output-format",
            "json",
            pkg.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    Ok(serde_json::from_slice(&output)?)
}

#[rstest]
#[case::normal(
    SotestConfig {
        libname: "sotest".to_string(),
        lookup: SonameLookupDirectory::from_str("lib:/usr/lib").unwrap(),
        dep: "lib:libsotest.so.1".parse()?,
        expect_dep: Some("lib:libsotest.so.1".parse()?),
        expect_provide: Some("lib:libsotest.so.1".parse()?),
    },
)]
#[case::no_ver(
    SotestConfig {
        libname: "sotest".to_string(),
        lookup: SonameLookupDirectory::from_str("lib:/usr/lib").unwrap(),
        dep: "lib:libsotest.so".parse()?,
        expect_dep: None,
        expect_provide: Some("lib:libsotest.so".parse()?),
    },
)]
#[case::wrong_ver(
    SotestConfig {
        libname: "sotest".to_string(),
        lookup: SonameLookupDirectory::from_str("lib:/usr/lib")?,
        dep: "lib:libsotest.so.2".parse()?,
        expect_dep: None,
        expect_provide: Some("lib:libsotest.so.2".parse()?),
    },
)]
#[case::alt_soname(
    SotestConfig {
        libname: "foo".to_string(),
        lookup: SonameLookupDirectory::from_str("lib:/usr/lib")?,
        dep: "lib:libfoo.so.1".parse()?,
        expect_dep: Some("lib:libfoo.so.1".parse()?),
        expect_provide: Some("lib:libfoo.so.1".parse()?),
    },
)]
#[case::mismatch_soname(
    SotestConfig {
        libname: "missing".to_string(),
        lookup: SonameLookupDirectory::from_str("lib:/usr/lib")?,
        dep: "lib:libsotest.so.1".parse()?,
        expect_dep: None,
        expect_provide: Some("lib:libsotest.so.1".parse()?),
    },
)]
#[case::wrong_prefix(
    SotestConfig {
        libname: "sotest".to_string(),
        lookup: SonameLookupDirectory::from_str("lib64:/usr/lib")?,
        dep: "lib:libsotest.so.1".parse()?,
        expect_dep: None,
        expect_provide: None,
    },
)]
#[case::alt_prefix(
    SotestConfig {
        libname: "sotest".to_string(),
        lookup: SonameLookupDirectory::from_str("lib64:/usr/lib")?,
        dep: "lib64:libsotest.so.1".parse()?,
        expect_dep: Some("lib64:libsotest.so.1".parse()?),
        expect_provide: Some("lib64:libsotest.so.1".parse()?),
    },
)]
fn test_soname_lookup(#[case] config: SotestConfig) -> TestResult {
    let temp_dir = TempDir::new()?;
    let path = temp_dir.path();

    setup_lib(&config, path, Path::new(MESON_FILES_DIR))?;
    let lib = create_lib_package(path, &config)?;
    let bin = create_bin_package(path, &config)?;

    let provisions = find_provisions(lib.to_path_buf(), config.lookup.clone())?;
    if let Some(prov) = &config.expect_provide {
        assert!(
            provisions.iter().any(|d| d == prov),
            "Expected provision not found: {prov}"
        );
    } else {
        assert!(
            provisions.is_empty(),
            "Expected no provisions, but found some."
        );
    }
    assert_eq!(
        get_provisions_via_cli(&lib.to_path_buf(), &config.lookup)?,
        provisions
    );

    let dependencies = find_dependencies(bin.to_path_buf(), config.lookup.clone())?;
    if let Some(dep) = &config.expect_dep {
        assert!(
            dependencies.iter().any(|d| d == dep),
            "Expected dependency not found: {dep}"
        );
    } else {
        assert!(
            dependencies.is_empty(),
            "Expected no dependencies, but found some."
        );
    }
    assert_eq!(
        get_dependencies_via_cli(&bin.to_path_buf(), &config.lookup)?,
        dependencies
    );

    let elf_sonames = extract_elf_sonames(bin.to_path_buf())?;
    let expected_soname = ElfSonames {
        path: PathBuf::from("usr/bin/sotest"),
        sonames: vec![
            Soname {
                name: format!("lib{}.so", config.libname).parse()?,
                version: Some("1".parse()?),
            },
            "libc.so.6".parse()?,
        ],
    };
    assert!(
        elf_sonames.contains(&expected_soname),
        "Expected to find {expected_soname:?} in {elf_sonames:?}"
    );
    let mut expected_sonames: Vec<_> = elf_sonames.iter().flat_map(|e| e.sonames.clone()).collect();
    expected_sonames.sort();
    assert_eq!(
        get_raw_dependencies_via_cli(&bin.to_path_buf())?,
        expected_sonames
    );

    Ok(())
}

/// Output structure when run as a standalone script.
#[derive(Serialize)]
struct ScriptOutput {
    /// Path to the created library package.
    lib_package_path: PathBuf,
    /// Path to the created binary package.
    bin_package_path: PathBuf,
}

/// Entry point when run as a standalone script (e.g. ./integration.rs).
///
/// Takes an optional JSON-encoded configuration as a command line argument.
/// If no argument is provided, a default configuration is used.
fn main() -> TestResult {
    let args: Vec<String> = env::args().collect();
    let current_dir = env::current_dir()?;
    let test_files_dir = current_dir.join("test_files");
    if !test_files_dir.exists() {
        return Err(TestError::from(format!(
            "test_files directory not found: {}",
            test_files_dir.display()
        )));
    }

    let output_dir = env::var("OUTPUT_DIR").unwrap_or_else(|_| "output".to_string());
    let path = current_dir.join(output_dir);

    let cfg = if let Some(arg) = args.get(1) {
        serde_json::from_str(arg)?
    } else {
        SotestConfig {
            libname: "example".to_string(),
            lookup: SonameLookupDirectory::from_str("lib:/usr/lib").unwrap(),
            dep: "lib:libexample.so.1".parse()?,
            expect_dep: None,
            expect_provide: None,
        }
    };

    setup_lib(&cfg, &path, &test_files_dir)?;
    let lib = create_lib_package(&path, &cfg)?;
    let bin = create_bin_package(&path, &cfg)?;

    let output = ScriptOutput {
        lib_package_path: lib.to_path_buf(),
        bin_package_path: bin.to_path_buf(),
    };

    println!("{}", serde_json::to_string(&output)?);

    Ok(())
}
