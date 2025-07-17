//! Test fixtures for lint rule testing.
//!
//! It provides bare-bones default fixtures for all "larger" datatypes of the various ALPM types.
//!
//! These are then adjusted in the actual tests for their respective need.

// We explicitly allow unused imports, as these imports aren't used/included by all test suites.
// Otherwise, this can lead to flaky clippy issues when looking at specific files.
#![allow(dead_code)]

use std::{collections::BTreeMap, str::FromStr};

use alpm_buildinfo::BuildInfoV2;
use alpm_pkginfo::{PackageInfoV2, RelationOrSoname};
use alpm_srcinfo::{
    SourceInfoV1,
    source_info::v1::{
        package::{Override, Package},
        package_base::PackageBase,
    },
};
use alpm_types::{
    Architecture,
    BuildDate,
    BuildDirectory,
    BuildTool,
    BuildToolVersion,
    Checksum,
    ExtraData,
    FullVersion,
    InstalledSize,
    License,
    Name,
    PackageDescription,
    Packager,
    SchemaVersion,
    StartDirectory,
    Url,
    digests::Sha256,
};
use testresult::TestResult;

/// Creates a default [`SourceInfoV1`] instance for testing.
///
/// The struct is as barebones as it gets.
pub fn default_source_info_v1() -> TestResult<SourceInfoV1> {
    Ok(SourceInfoV1 {
        base: PackageBase {
            name: Name::from_str("test-package")?,
            version: FullVersion::from_str("1:1.0.0-1")?,
            description: Some(PackageDescription::new(
                "A test package for lint rule testing",
            )),
            architectures: vec![Architecture::Any],
            url: Some(Url::from_str("https://example.com")?),
            licenses: vec![License::new("GPL-3.0".to_string())?],
            groups: Vec::new(),
            dependencies: Vec::new(),
            make_dependencies: Vec::new(),
            check_dependencies: Vec::new(),
            optional_dependencies: Vec::new(),
            provides: Vec::new(),
            conflicts: Vec::new(),
            replaces: Vec::new(),
            backups: Vec::new(),
            options: Vec::new(),
            install: None,
            changelog: None,
            sources: Vec::new(),
            no_extracts: Vec::new(),
            md5_checksums: Vec::new(),
            sha1_checksums: Vec::new(),
            sha224_checksums: Vec::new(),
            sha256_checksums: Vec::new(),
            sha384_checksums: Vec::new(),
            sha512_checksums: Vec::new(),
            b2_checksums: Vec::new(),
            pgp_fingerprints: Vec::new(),
            architecture_properties: BTreeMap::new(),
        },
        packages: vec![Package {
            name: Name::from_str("test-package")?,
            description: Override::No,
            architectures: None,
            url: Override::No,
            licenses: Override::No,
            backups: Override::No,
            groups: Override::No,
            dependencies: Override::No,
            optional_dependencies: Override::No,
            provides: Override::No,
            conflicts: Override::No,
            replaces: Override::No,
            options: Override::No,
            install: Override::No,
            changelog: Override::No,
            architecture_properties: BTreeMap::new(),
        }],
    })
}

/// Creates a default [`BuildInfoV2`] instance for testing.
pub fn default_build_info_v2() -> TestResult<BuildInfoV2> {
    Ok(BuildInfoV2::new(
        BuildDate::from_str("1")?,
        BuildDirectory::from_str("/build")?,
        StartDirectory::from_str("/startdir/")?,
        BuildTool::from_str("devtools")?,
        BuildToolVersion::from_str("1:1.2.1-1-any")?,
        vec![],
        SchemaVersion::from_str("2")?,
        vec![],
        vec![],
        Packager::from_str("Test User <test@example.org>")?,
        Architecture::Any,
        Name::new("test-package")?,
        Checksum::<Sha256>::calculate_from("test-content"),
        Name::new("test-package")?,
        FullVersion::from_str("1:1.0.0-1")?,
    )?)
}

/// Creates a default [`PackageInfoV2`] instance for testing.
pub fn default_package_info_v2() -> TestResult<PackageInfoV2> {
    Ok(PackageInfoV2::new(
        Name::new("test-package")?,
        Name::new("test-package")?,
        FullVersion::from_str("1:1.0.0-1")?,
        PackageDescription::from("A test package for lint rule testing"),
        Url::from_str("https://example.com")?,
        BuildDate::from_str("1729181726")?,
        Packager::from_str("Test User <test@example.org>")?,
        InstalledSize::from_str("1000000")?,
        Architecture::Any,
        vec![License::from_str("GPL-3.0-or-later")?],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![RelationOrSoname::from_str("glibc")?],
        vec![],
        vec![],
        vec![],
        vec![ExtraData::from_str("pkgtype=pkg")?],
    )?)
}
