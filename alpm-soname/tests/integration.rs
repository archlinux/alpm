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
fn get_raw_dependencies_via_cli(pkg: &Path) -> TestResult<Vec<Soname>> {
    let mut cmd = cargo_bin_cmd!("alpm-soname");
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
