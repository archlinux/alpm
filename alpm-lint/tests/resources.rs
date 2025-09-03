//! Integration tests for Resources gathering functionality.

use std::{
    fs::{create_dir, write},
    os::unix::fs::symlink,
    path::Path,
};

use alpm_lint::{LintScope, Resources};
use rstest::rstest;
use tempfile::tempdir;
use testresult::TestResult;

mod fixtures;

use factories::{
    create_buildinfo_file,
    create_package_files,
    create_pkgbuild_file,
    create_pkginfo_file,
    create_source_repo_files,
    create_srcinfo_file,
};

type FileFactory = Box<dyn FnOnce(&Path) -> TestResult<()>>;

/// This helper module contains helper functions to create various ALPM metadata stub files at a
/// given path. To use them as rstest case parameters, all functions have the same function
/// signature.
mod factories {
    use super::{
        Path,
        TestResult,
        fixtures::{default_build_info_v2, default_package_info_v2, default_source_info_v1},
        write,
    };

    /// Helper function to create a file with content in a the given directory.
    pub fn create_file_with_content(dir: &Path, filename: &str, content: &str) -> TestResult<()> {
        let file_path = dir.join(filename);
        write(file_path, content)?;
        Ok(())
    }

    /// Helper function to create all files inside a source repo scope.
    pub fn create_source_repo_files(dir: &Path) -> TestResult<()> {
        create_srcinfo_file(dir)?;
        create_pkgbuild_file(dir)
    }

    /// Helper function to create all files inside a Package scope.
    pub fn create_package_files(dir: &Path) -> TestResult<()> {
        create_pkginfo_file(dir)?;
        create_buildinfo_file(dir)
    }

    /// Helper function to create a .SRCINFO file with test data.
    pub fn create_srcinfo_file(dir: &Path) -> TestResult<()> {
        let source_info = default_source_info_v1()?;
        let content = source_info.as_srcinfo();
        create_file_with_content(dir, ".SRCINFO", &content)
    }

    /// Helper function to create a .PKGINFO file with test data.
    pub fn create_pkginfo_file(dir: &Path) -> TestResult<()> {
        let package_info = default_package_info_v2()?;
        let content = package_info.to_string();
        create_file_with_content(dir, ".PKGINFO", &content)
    }

    /// Helper function to create a .BUILDINFO file with test data.
    pub fn create_buildinfo_file(dir: &Path) -> TestResult<()> {
        let build_info = default_build_info_v2()?;
        let content = build_info.to_string();
        create_file_with_content(dir, ".BUILDINFO", &content)
    }

    /// Helper function to create a PKGBUILD file with minimal content.
    pub fn create_pkgbuild_file(dir: &Path) -> TestResult<()> {
        let content = r#"# Maintainer: Test User <test@example.org>
pkgname=test-package
pkgver=1.0.0
pkgrel=1
pkgdesc="A test package"
arch=('any')
url="https://example.com"
license=('GPL-3.0-or-later')
source=()
sha256sums=()

package() {
    install -Dm644 /dev/null "$pkgdir/usr/share/doc/$pkgname/README"
}
"#;
        create_file_with_content(dir, "PKGBUILD", content)
    }
}

/// Test that Resources::gather_file correctly loads individual files for single-file scopes.
#[rstest]
#[case::build_info(".BUILDINFO", LintScope::BuildInfo, Box::new(create_buildinfo_file))]
#[case::package_info(".PKGINFO", LintScope::PackageInfo, Box::new(create_pkginfo_file))]
#[case::pkgbuild("PKGBUILD", LintScope::PackageBuild, Box::new(create_pkgbuild_file))]
#[case::srcinfo(".SRCINFO", LintScope::SourceInfo, Box::new(create_srcinfo_file))]
fn single_file(
    #[case] filename: &str,
    #[case] scope: LintScope,
    #[case] create_file: FileFactory,
) -> TestResult<()> {
    let temp_dir = tempdir()?;
    let path = temp_dir.path();

    create_file(path)?;
    let file_path = path.join(filename);

    let resources = Resources::gather_file(&file_path, scope)?;

    assert_eq!(
        resources.scope(),
        scope,
        "Resource scope doesn't match expected scope."
    );
    Ok(())
}

/// Test that Resources::gather_file correctly loads individual files inside a directory for
/// single-file scopes.
#[rstest]
#[case::build_info(LintScope::BuildInfo, Box::new(create_buildinfo_file))]
#[case::package_info(LintScope::PackageInfo, Box::new(create_pkginfo_file))]
#[case::pkgbuild(LintScope::PackageBuild, Box::new(create_pkgbuild_file))]
#[case::srcinfo(LintScope::SourceInfo, Box::new(create_srcinfo_file))]
fn single_file_in_dir(
    #[case] scope: LintScope,
    #[case] create_file: FileFactory,
) -> TestResult<()> {
    let temp_dir = tempdir()?;
    let path = temp_dir.path();

    create_file(path)?;
    let resources = Resources::gather_file(path, scope)?;

    assert_eq!(
        resources.scope(),
        scope,
        "Resource scope doesn't match expected scope."
    );

    Ok(())
}

/// Test that resource gathering works for multi-file scopes.
#[rstest]
#[case::source_repository_scope(LintScope::SourceRepository, Box::new(create_source_repo_files))]
#[case::package_scope(LintScope::Package, Box::new(create_package_files))]
fn multi_file(#[case] scope: LintScope, #[case] setup_files: FileFactory) -> TestResult<()> {
    let temp_dir = tempdir()?;
    let path = temp_dir.path();

    setup_files(path)?;

    let resources = Resources::gather(path, scope)?;
    assert_eq!(
        resources.scope(),
        scope,
        "Resource scope doesn't match expected scope."
    );

    Ok(())
}

/// Test that Resources::gather fails appropriately when looking at directories and not finding all
/// required files for any of the "larger" multi-file scopes.
#[rstest]
#[case::missing_pkgbuild(LintScope::SourceRepository, Box::new(create_srcinfo_file))]
#[case::missing_srcinfo(LintScope::SourceRepository, Box::new(create_pkgbuild_file))]
#[case::missing_pkginfo(LintScope::Package, Box::new(create_buildinfo_file))]
#[case::missing_buildinfo(LintScope::Package, Box::new(create_pkginfo_file))]
fn multi_file_missing_files(
    #[case] scope: LintScope,
    #[case] create_partial_files: FileFactory,
) -> TestResult<()> {
    let temp_dir = tempdir()?;
    let path = temp_dir.path();

    create_partial_files(path)?;

    let result = Resources::gather(path, scope);
    assert!(
        result.is_err(),
        "Expected error when files are missing for scope {scope:?}"
    );

    Ok(())
}

/// Test that single-file symlinks are respected.
#[test]
fn gather_follows_symlinks() -> TestResult<()> {
    let temp_dir = tempdir()?;
    let tempdir_path = temp_dir.path();

    // Create the source repo dir.
    let srcinfo_path = tempdir_path.join(".SRCINFO");
    create_srcinfo_file(tempdir_path)?;

    // Create symlink to the source directory
    let symlink_path = tempdir_path.join("symlink");
    symlink(&srcinfo_path, &symlink_path)?;

    let resources = Resources::gather(&symlink_path, LintScope::SourceInfo)?;
    assert_eq!(resources.scope(), LintScope::SourceInfo);

    Ok(())
}

/// Test that multi-file dir symlinks are respected.
#[test]
fn gather_follows_dir_symlinks() -> TestResult<()> {
    let temp_dir = tempdir()?;
    let source_dir = temp_dir.path().join("source");

    // Create the source repo dir.
    create_dir(&source_dir)?;
    create_pkgbuild_file(&source_dir)?;
    create_srcinfo_file(&source_dir)?;

    // Create symlink to the source directory
    let symlink_path = temp_dir.path().join("symlink");
    symlink(&source_dir, &symlink_path)?;

    let resources = Resources::gather(&symlink_path, LintScope::SourceRepository)?;
    assert_eq!(resources.scope(), LintScope::SourceRepository);

    Ok(())
}
