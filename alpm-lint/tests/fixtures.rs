//! Test fixtures for lint rule testing.
//!
//! Provides simple default data fixtures for all composite data types representing the various ALPM
//! file types.
//!
//! These fixtures are meant to be adjusted in the actual tests for their respective need.

// We explicitly allow unused imports, as these imports aren't used/included by all test suites.
// Otherwise, this can lead to flaky clippy issues when looking at specific files.
#![allow(dead_code)]

use std::str::FromStr;

use alpm_buildinfo::BuildInfoV2;
use alpm_pkginfo::{PackageInfoV2, RelationOrSoname};
use alpm_srcinfo::{
    SourceInfoV1,
    source_info::v1::{package::Package, package_base::PackageBase},
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
/// The data provides a single package for "any" architecture.
pub fn default_source_info_v1() -> TestResult<SourceInfoV1> {
    Ok(SourceInfoV1 {
        base: PackageBase {
            architectures: vec![Architecture::Any],
            ..PackageBase::new_with_defaults(
                Name::from_str("test-package")?,
                FullVersion::from_str("1:1.0.0-1")?,
            )
        },
        packages: vec![Package::new_with_defaults(Name::from_str("test-package")?)],
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
