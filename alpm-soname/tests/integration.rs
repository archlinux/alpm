use std::{
    fs::{create_dir_all, write},
    path::Path,
    process::Command,
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
use alpm_soname::{Autodeps, find_dependencies, find_provisions};
use alpm_types::{AbsolutePath, MetadataFileName, SharedLibraryPrefix, SonameLookupDirectory};
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
    /// Libdir alpm-soname should use for soname lookup (e.g. /usr/lib)
    libdir: &'static str,
    /// The prefix alpm-soname should use for soname lookup (e.g. lib)
    prefix: &'static str,
    /// If all is true, alpm-soname will return all dependencies, even those without matching
    /// provisions.
    all: bool,
    /// the depstring added as a provide for the lib .PKGINFO and as a depend for the bin .PKGINFO.
    dep: Option<&'static str>,
    /// The string representation of the soname we expect find_dependencies to return
    expect_dep: Vec<&'static str>,
    /// The string representation of the soname we expect find_provisions to return
    expect_provide: Vec<&'static str>,
    /// Also place the shared object into the bin package.
    lib_in_bin: bool,
}

fn pkginfo_lib(dep: Option<&str>) -> String {
    let mut pkginfo = r#"
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
"#
    .to_string();
    if let Some(dep) = dep {
        pkginfo.push_str("provides = ");
        pkginfo.push_str(dep);
    }
    pkginfo
}

fn pkginfo_bin(dep: Option<&str>) -> String {
    let mut pkginfo = r#"
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
"#
    .to_string();

    if let Some(dep) = dep {
        pkginfo.push_str("depend = ");
        pkginfo.push_str(dep);
    }
    pkginfo
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
        pkginfo_bin(config.dep),
    )?;
    write(
        input_dir.join(MetadataFileName::BuildInfo.as_ref()),
        BUILDINFO_BIN,
    )?;

    create_dir_all(input_dir.join("usr/bin"))?;
    create_dir_all(input_dir.join("usr/lib"))?;
    create_dir_all(input_dir.join("empty"))?;
    std::fs::copy(path.join("build/sotest"), input_dir.join("usr/bin/sotest"))?;
    if config.lib_in_bin {
        std::fs::copy(
            path.join(format!("build/lib{}.so.1", config.libname)),
            input_dir.join(format!("usr/lib/lib{}.so.1", config.libname)),
        )?;
    }

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
        pkginfo_lib(config.dep),
    )?;
    write(
        input_dir.join(MetadataFileName::BuildInfo.as_ref()),
        BUILDINFO_LIB,
    )?;

    create_dir_all(input_dir.join("usr/bin"))?;
    create_dir_all(input_dir.join("usr/lib"))?;
    create_dir_all(input_dir.join("empty"))?;
    std::fs::copy(
        path.join(format!("build/lib{}.so.1", config.libname)),
        input_dir.join(format!("usr/lib/lib{}.so.1", config.libname)),
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
        prefix: "lib",
        libdir: "/usr/lib",
        all: false,
        dep: Some("lib:libsotest.so.1"),
        expect_dep: vec!["lib:libsotest.so.1"],
        expect_provide: vec!["lib:libsotest.so.1"],
        lib_in_bin: false,
    },
)]
#[case::normal_all(
    SotestConfig {
        libname: "sotest",
        prefix: "lib",
        libdir: "/usr/lib",
        all: true,
        dep: Some("lib:libsotest.so.1"),
        expect_dep: vec!["lib:libsotest.so.1"],
        expect_provide: vec!["lib:libsotest.so.1"],
        lib_in_bin: false,
    },
)]
#[case::no_ver(
    SotestConfig {
        libname: "sotest",
        prefix: "lib",
        libdir: "/usr/lib",
        all: false,
        dep: Some("lib:libsotest.so"),
        expect_dep: vec![],
        expect_provide: vec!["lib:libsotest.so"],
        lib_in_bin: false,
    },
)]
#[case::no_ver_all(
    SotestConfig {
        libname: "sotest",
        prefix: "lib",
        libdir: "/usr/lib",
        all: true,
        dep: Some("lib:libsotest.so"),
        expect_dep: vec!["lib:libsotest.so"],
        expect_provide: vec!["lib:libsotest.so"],
        lib_in_bin: false,
    },
)]
#[case::wrong_ver(
    SotestConfig {
        libname: "sotest",
        prefix: "lib",
        libdir: "/usr/lib",
        all: false,
        dep: Some("lib:libsotest.so.2"),
        expect_dep: vec![],
        expect_provide: vec!["lib:libsotest.so.2"],
        lib_in_bin: false,
    },
)]
#[case::wrong_ver_all(
    SotestConfig {
        libname: "sotest",
        prefix: "lib",
        libdir: "/usr/lib",
        all: true,
        dep: Some("lib:libsotest.so.2"),
        expect_dep: vec!["lib:libsotest.so.2"],
        expect_provide: vec!["lib:libsotest.so.2"],
        lib_in_bin: false,
    },
)]
#[case::alt_soname(
    SotestConfig {
        libname: "foo",
        prefix: "lib",
        libdir: "/usr/lib",
        all: false,
        dep: Some("lib:libfoo.so.1"),
        expect_dep: vec!["lib:libfoo.so.1"],
        expect_provide: vec!["lib:libfoo.so.1"],
        lib_in_bin: false,
    },
)]
#[case::alt_soname_all(
    SotestConfig {
        libname: "foo",
        prefix: "lib",
        libdir: "/usr/lib",
        all: true,
        dep: Some("lib:libfoo.so.1"),
        expect_dep: vec!["lib:libfoo.so.1"],
        expect_provide: vec!["lib:libfoo.so.1"],
        lib_in_bin: false,
    },
)]
#[case::mismatch_soname(
    SotestConfig {
        libname: "missing",
        prefix: "lib",
        libdir: "/usr/lib",
        all: false,
        dep: Some("lib:libsotest.so.1"),
        expect_dep: vec![],
        expect_provide: vec!["lib:libsotest.so.1"],
        lib_in_bin: false,
    },
)]
#[case::mismatch_soname_all(
    SotestConfig {
        libname: "missing",
        prefix: "lib",
        libdir: "/usr/lib",
        all: true,
        dep: Some("lib:libsotest.so.1"),
        expect_dep: vec!["lib:libsotest.so.1"],
        expect_provide: vec!["lib:libsotest.so.1"],
        lib_in_bin: false,
    },
)]
#[case::wrong_prefix(
    SotestConfig {
        libname: "sotest",
        prefix: "lib64",
        libdir: "/usr/lib",
        all: false,
        dep: Some("lib:libsotest.so.1"),
        expect_dep: vec![],
        expect_provide: vec![],
        lib_in_bin: false,
    },
)]
#[case::alt_prefi(
    SotestConfig {
        libname: "sotest",
        prefix: "lib64",
        libdir: "/usr/lib",
        all: false,
        dep: Some("lib64:libsotest.so.1"),
        expect_dep: vec!["lib64:libsotest.so.1"],
        expect_provide: vec!["lib64:libsotest.so.1"],
        lib_in_bin: false,
    },
)]
fn test_so(#[case] config: SotestConfig) -> TestResult {
    let temp_dir = TempDir::new()?;
    let path = temp_dir.path();

    setup_lib(&config, path)?;
    let lib = create_lib(path, &config)?;
    let bin = create_bin(path, &config)?;

    let lookup = SonameLookupDirectory::new(
        SharedLibraryPrefix::new(config.prefix)?,
        AbsolutePath::new(config.libdir.into())?,
    );

    let provisions = find_provisions(lib.to_path_buf(), lookup.clone())?;
    let dependencies = find_dependencies(bin.to_path_buf(), lookup, config.all)?;

    assert_eq!(
        provisions.iter().map(|d| d.to_string()).collect::<Vec<_>>(),
        config.expect_provide,
        "Provision mismatch for case: {config:#?}",
    );
    assert_eq!(
        dependencies
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>(),
        config.expect_dep,
        "Dependency mismatch for case: {config:#?}",
    );

    Ok(())
}

#[rstest]
#[case::normal(
    SotestConfig {
        libname: "sotest",
        prefix: "lib",
        libdir: "/usr/lib",
        all: false,
        dep: None,
        expect_dep: vec!["lib:libc.so.6", "lib:libsotest.so.1"],
        expect_provide: vec!["lib:libsotest.so.1"],
        lib_in_bin: false,
    },
)]
#[case::alt_libname(
    SotestConfig {
        libname: "lu",
        prefix: "lib",
        libdir: "/usr/lib",
        all: false,
        dep: None,
        expect_dep: vec!["lib:libc.so.6", "lib:liblu.so.1"],
        expect_provide: vec!["lib:liblu.so.1"],
        lib_in_bin: false,
    },
)]
#[case::alt_prefix(
    SotestConfig {
        libname: "sotest",
        prefix: "lib64",
        libdir: "/usr/lib",
        all: false,
        dep: None,
        expect_dep: vec!["lib64:libc.so.6", "lib64:libsotest.so.1"],
        expect_provide: vec!["lib64:libsotest.so.1"],
        lib_in_bin: false,
    },
)]
#[case::alt_libdir(
    SotestConfig {
        libname: "sotest",
        prefix: "lib",
        libdir: "/empty",
        all: false,
        dep: None,
        expect_dep: vec!["lib:libc.so.6", "lib:libsotest.so.1"],
        expect_provide: vec![],
        lib_in_bin: false,
    },
)]
#[case::alt_libdir2(
    SotestConfig {
        libname: "sotest",
        prefix: "lib",
        libdir: "/usr",
        all: false,
        dep: None,
        expect_dep: vec!["lib:libc.so.6", "lib:libsotest.so.1"],
        expect_provide: vec![],
        lib_in_bin: false,
    },
)]
#[case::package_provides_dependency(
    SotestConfig {
        libname: "sotest",
        prefix: "lib",
        libdir: "/usr/lib",
        all: false,
        dep: None,
        expect_dep: vec!["lib:libc.so.6"],
        expect_provide: vec!["lib:libsotest.so.1"],
        lib_in_bin: true,
    },
)]
#[case::libdir_missing(
    SotestConfig {
        libname: "sotest",
        prefix: "lib",
        libdir: "/foo/bar",
        all: false,
        dep: None,
        expect_dep: vec!["lib:libc.so.6", "lib:libsotest.so.1"],
        expect_provide: vec![],
        lib_in_bin: false,
    },
)]

fn test_autodeps(#[case] config: SotestConfig) -> TestResult {
    let temp_dir = TempDir::new()?;
    let path = temp_dir.path();

    setup_lib(&config, path)?;
    create_lib(path, &config)?;
    create_bin(path, &config)?;

    let lookup = SonameLookupDirectory::new(
        SharedLibraryPrefix::new(config.prefix)?,
        AbsolutePath::new(config.libdir.into())?,
    );

    let input_bin = path.join("input_bin");
    let input_lib = path.join("input_lib");

    let bin_deps = Autodeps::new(input_bin, lookup.clone())?;
    let lib_deps = Autodeps::new(input_lib, lookup.clone())?;

    assert_eq!(
        lib_deps
            .provides
            .into_iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>(),
        config.expect_provide,
        "Provision mismatch for case: {config:#?}",
    );

    assert_eq!(
        bin_deps
            .depends
            .into_iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>(),
        config.expect_dep,
        "Dependency mismatch for case: {config:#?}",
    );

    Ok(())
}
