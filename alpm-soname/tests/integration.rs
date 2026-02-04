//! This integration test verifies that shared object dependencies and provisions
//! are correctly identified in packages.
//!
//! It does so by creating a simple C project with Meson that builds a shared library
//! and a binary that depends on it. The test then creates packages for both the library
//! and the binary, and checks that the binary's dependencies include the library's soname,
//! and that the library's provisions include its own soname.
//!
//! These tests are only executed when the `cli` feature flag is enabled.
#![cfg(feature = "cli")]

use std::{
    env,
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_soname::{ElfSonames, extract_elf_sonames, find_dependencies, find_provisions};
use alpm_types::{Soname, SonameLookupDirectory, SonameV2};
use assert_cmd::cargo::cargo_bin_cmd;
use rstest::rstest;
use tempfile::TempDir;
use testresult::TestResult;

mod shared;

use shared::*;

const MESON_FILES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/test_files");

/// Invoke the CLI to get provisions.
fn get_provisions_via_cli(pkg: &Path, lookup: &SonameLookupDirectory) -> TestResult<Vec<SonameV2>> {
    let mut cmd = cargo_bin_cmd!("alpm-soname");
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
    let mut cmd = cargo_bin_cmd!("alpm-soname");
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
fn get_raw_dependencies_via_cli<I, S>(pkg: &Path, args: I) -> Vec<u8>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut cmd = cargo_bin_cmd!("alpm-soname");
    cmd.args(["get-raw-dependencies"])
        .args(args)
        .arg(pkg.to_str().unwrap())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone()
}

fn get_raw_dependencies_default_via_cli(pkg: &Path) -> TestResult<Vec<Soname>> {
    let output = get_raw_dependencies_via_cli(pkg, ["--output-format", "json"]);
    Ok(serde_json::from_slice(&output)?)
}

fn get_raw_dependencies_detail_via_cli(pkg: &Path) -> TestResult<Vec<ElfSonames>> {
    let output = get_raw_dependencies_via_cli(pkg, ["--output-format", "json", "--detail"]);
    Ok(serde_json::from_slice(&output)?)
}

fn get_raw_dependencies_elf1_via_cli(pkg: &Path) -> TestResult<ElfSonames> {
    let output =
        get_raw_dependencies_via_cli(pkg, ["--output-format", "json", "--elf", "usr/bin/sotest"]);
    Ok(serde_json::from_slice(&output)?)
}

fn get_raw_dependencies_elf2_via_cli(pkg: &Path) -> TestResult<ElfSonames> {
    let output =
        get_raw_dependencies_via_cli(pkg, ["--output-format", "json", "-e", "/usr/bin/sotest2"]);
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

    let sonames_detail = extract_elf_sonames(bin.to_path_buf())?;
    let soname_binsotest = ElfSonames {
        path: PathBuf::from("usr/bin/sotest"),
        sonames: vec![
            Soname {
                name: format!("lib{}.so", config.libname).parse()?,
                version: Some("1".parse()?),
            },
            "libc.so.6".parse()?,
        ],
    };
    let soname_binsotest2 = {
        let mut tmp = soname_binsotest.clone();
        tmp.path.set_file_name("sotest2");
        tmp
    };
    assert!(
        sonames_detail.contains(&soname_binsotest),
        "Expected to find {soname_binsotest:?} in {sonames_detail:?}"
    );
    assert!(
        sonames_detail.contains(&soname_binsotest2),
        "Expected to find {soname_binsotest2:?} in {sonames_detail:?}"
    );
    let sonames_default = {
        let mut sonames_default: Vec<_> = sonames_detail
            .iter()
            .flat_map(|elf| elf.sonames.clone())
            .collect();
        sonames_default.sort();
        sonames_default.dedup();
        sonames_default
    };
    assert_eq!(
        get_raw_dependencies_default_via_cli(&bin.to_path_buf())?,
        sonames_default
    );
    assert_eq!(
        get_raw_dependencies_detail_via_cli(&bin.to_path_buf())?,
        sonames_detail
    );
    assert_eq!(
        get_raw_dependencies_elf1_via_cli(&bin.to_path_buf())?,
        soname_binsotest
    );
    assert_eq!(
        get_raw_dependencies_elf2_via_cli(&bin.to_path_buf())?,
        soname_binsotest2
    );

    Ok(())
}
