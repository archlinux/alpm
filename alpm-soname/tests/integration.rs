//! Integration tests for alpm-soname.

use std::{
    fs::{copy, create_dir_all, write},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use alpm_mtree::create_mtree_v2_from_input_dir;
use alpm_package::{
    CompressionSettings,
    InputDir,
    OutputDir,
    Package,
    PackageCreationConfig,
    PackageInput,
};
use alpm_soname::{ElfSonames, extract_elf_sonames, find_dependencies, find_provisions};
use alpm_types::{MetadataFileName, Soname, SonameLookupDirectory, SonameV2};
use rstest::rstest;
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
#[derive(Debug)]
struct SotestConfig {
    /// The name of the shared object we generate.
    /// The soname and filename become lib{libname}.so.1
    libname: &'static str,
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

fn pkginfo_lib(dep: &SonameV2) -> String {
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

fn pkginfo_bin(dep: &SonameV2) -> String {
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

fn setup_lib(config: &SotestConfig, path: &Path) -> TestResult {
    let status = Command::new("meson")
        .arg("setup")
        .arg(format!("-Dlibname={}", config.libname))
        .arg(path.join("build"))
        .arg(MESON_FILES_DIR)
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
        return Err(TestError::from("failed to setup sotest c project"));
    }

    Ok(())
}

fn create_bin(path: &Path, config: &SotestConfig) -> TestResult<Package> {
    let input_dir = path.join("input_bin");
    create_dir_all(&input_dir)?;
    let input_dir = InputDir::new(input_dir)?;
    let output_dir = OutputDir::new(path.join("output_bin"))?;

    write(
        input_dir.join(MetadataFileName::PackageInfo.as_ref()),
        pkginfo_bin(&config.dep),
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

fn create_lib(path: &Path, config: &SotestConfig) -> TestResult<Package> {
    let input_dir = path.join("input_lib");
    create_dir_all(&input_dir)?;
    let input_dir = InputDir::new(input_dir)?;
    let output_dir = OutputDir::new(path.join("output_lib"))?;

    write(
        input_dir.join(MetadataFileName::PackageInfo.as_ref()),
        pkginfo_lib(&config.dep),
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

#[rstest]
#[case::normal(
    SotestConfig {
        libname: "sotest",
        lookup: SonameLookupDirectory::from_str("lib:/usr/lib").unwrap(),
        dep: "lib:libsotest.so.1".parse()?,
        expect_dep: Some("lib:libsotest.so.1".parse()?),
        expect_provide: Some("lib:libsotest.so.1".parse()?),
    },
)]
#[case::no_ver(
    SotestConfig {
        libname: "sotest",
        lookup: SonameLookupDirectory::from_str("lib:/usr/lib").unwrap(),
        dep: "lib:libsotest.so".parse()?,
        expect_dep: None,
        expect_provide: Some("lib:libsotest.so".parse()?),
    },
)]
#[case::wrong_ver(
    SotestConfig {
        libname: "sotest",
        lookup: SonameLookupDirectory::from_str("lib:/usr/lib")?,
        dep: "lib:libsotest.so.2".parse()?,
        expect_dep: None,
        expect_provide: Some("lib:libsotest.so.2".parse()?),
    },
)]
#[case::alt_soname(
    SotestConfig {
        libname: "foo",
        lookup: SonameLookupDirectory::from_str("lib:/usr/lib")?,
        dep: "lib:libfoo.so.1".parse()?,
        expect_dep: Some("lib:libfoo.so.1".parse()?),
        expect_provide: Some("lib:libfoo.so.1".parse()?),
    },
)]
#[case::mismatch_soname(
    SotestConfig {
        libname: "missing",
        lookup: SonameLookupDirectory::from_str("lib:/usr/lib")?,
        dep: "lib:libsotest.so.1".parse()?,
        expect_dep: None,
        expect_provide: Some("lib:libsotest.so.1".parse()?),
    },
)]
#[case::wrong_prefix(
    SotestConfig {
        libname: "sotest",
        lookup: SonameLookupDirectory::from_str("lib64:/usr/lib")?,
        dep: "lib:libsotest.so.1".parse()?,
        expect_dep: None,
        expect_provide: None,
    },
)]
#[case::alt_prefix(
    SotestConfig {
        libname: "sotest",
        lookup: SonameLookupDirectory::from_str("lib64:/usr/lib")?,
        dep: "lib64:libsotest.so.1".parse()?,
        expect_dep: Some("lib64:libsotest.so.1".parse()?),
        expect_provide: Some("lib64:libsotest.so.1".parse()?),
    },
)]
fn test_so(#[case] config: SotestConfig) -> TestResult {
    let temp_dir = TempDir::new()?;
    let path = temp_dir.path();

    setup_lib(&config, path)?;
    let lib = create_lib(path, &config)?;
    let bin = create_bin(path, &config)?;

    let provisions = find_provisions(lib.to_path_buf(), config.lookup.clone())?;
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

    Ok(())
}
