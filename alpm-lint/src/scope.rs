//! Representation and handling of linting scopes.

use std::{collections::HashSet, fs, path::Path};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use strum::Display as StrumDisplay;

use crate::Error;

/// The possible scope used to categorize linting rules.
///
/// Scopes are used to determine what lints should be executed based on a given linting operation.
/// For example, selecting [`LintScope::SourceInfo`] will run all SourceInfo specific linting
/// rules. Linting scopes can also be fully dis-/enabled via configuration files.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, StrumDisplay, ValueEnum)]
pub enum LintScope {
    /// Lint rules with this scope are specific to a ArchLinux package source repository.
    /// Such lint rules check the consistency of a source repository, such as consistency of data
    /// between various metadata files.
    ///
    /// When this scope is selected, the following LintScopes are also included:
    /// - PackageBuild
    /// - SourceInfo
    #[strum(to_string = "source_repo")]
    SourceRepository,
    /// Lint rules with this scope are specific to a ArchLinux packages.
    /// Such lint rules check the consistency of a package, such as consistency of data between
    /// various metadata files.
    ///
    /// When this scope is selected, the following LintScopes are also included:
    /// - PackageInfo
    /// - BuildInfo
    #[strum(to_string = "package")]
    Package,
    /// Lints on a `.BUILDINFO` file.
    #[strum(to_string = "build_info")]
    BuildInfo,
    /// Lints on a `PKGBUILD` file.
    #[strum(to_string = "package_build")]
    PackageBuild,
    /// Lints on a `.PKGINFO` file.
    #[strum(to_string = "package_info")]
    PackageInfo,
    /// Lints on a `.SRCINFO` file.
    #[strum(to_string = "source_info")]
    SourceInfo,
}

impl LintScope {
    /// Determine whether a given scope contains or matches this scope.
    ///
    /// "contains" and "matches" in this context means that it self is either identical or that the
    /// scope of `other` contains the scope of self.
    ///
    /// ```
    /// use alpm_lint::LintScope;
    ///
    /// let source_info = LintScope::SourceInfo;
    /// let source_repo = LintScope::SourceRepository;
    ///
    /// assert!(source_repo.contains(&source_info));
    /// assert!(source_info.contains(&source_info));
    /// assert!(!source_info.contains(&source_repo));
    /// ```
    pub fn contains(&self, other: &LintScope) -> bool {
        match self {
            // A `SourceRepository` scope may contain a SourceInfo or PackageBuild file.
            LintScope::SourceRepository => match other {
                LintScope::SourceRepository | LintScope::SourceInfo | LintScope::PackageBuild => {
                    true
                }
                LintScope::BuildInfo | LintScope::PackageInfo | LintScope::Package => false,
            },
            // A `Package` scope may contain a PackageBuild or PackageInfo file.
            LintScope::Package => match other {
                LintScope::Package | LintScope::PackageBuild | LintScope::PackageInfo => true,
                LintScope::BuildInfo | LintScope::SourceRepository | LintScope::SourceInfo => false,
            },
            // All scopes that are restricted to a single file require the exact same scope.
            LintScope::BuildInfo
            | LintScope::PackageBuild
            | LintScope::PackageInfo
            | LintScope::SourceInfo => self == other,
        }
    }

    /// Try to automatically detect all available linting scopes based on files from a specific
    /// directory.
    ///
    /// Usually, when calling `alpm-lint check`, [`LintScope::detect`] is used to
    /// automatically determine the available linting scope based on files in the specified
    /// directory. The current scopes can also be overridden by the user.
    ///
    /// Based on these scopes, files will be loaded and linting rules are selected for execution.
    pub fn detect(path: &Path) -> Result<LintScope, Error> {
        let mut metadata = fs::metadata(path).map_err(|source| Error::IoPath {
            path: path.to_owned(),
            context: "getting metadata of path",
            source,
        })?;

        // If the destiny is a symlink, follow it and check what the destination type is.
        if metadata.is_symlink() {
            metadata = fs::symlink_metadata(path).map_err(|source| Error::IoPath {
                path: path.to_owned(),
                context: "getting symlink metadata of path",
                source,
            })?;
        }

        // Handle the case where the path is a single file.
        if metadata.is_file() {
            let filename = path.file_name().ok_or(Error::NoLintScope {
                path: path.to_owned(),
            })?;

            // Package source repository related scopes
            if filename == "PKGBUILD" {
                return Ok(LintScope::PackageBuild);
            } else if filename == ".SRCINFO" {
                return Ok(LintScope::SourceInfo);
            // Package related scopes
            } else if filename == ".BUILDINFO" {
                return Ok(LintScope::BuildInfo);
            } else if filename == ".PKGINFO" {
                return Ok(LintScope::PackageInfo);
            } else {
                return Err(Error::NoLintScope {
                    path: path.to_path_buf(),
                });
            }
        }

        // ---
        // At this point, we know that this is a directory.
        // Look at the contained files and try to figure out which scope fits best.
        // ---

        let entries = fs::read_dir(path).map_err(|source| Error::IoPath {
            path: path.to_owned(),
            context: "read directory entries",
            source,
        })?;

        let mut filenames = HashSet::new();

        // Create a hashmap of filenames, so that we can easily determine which alpm files exist in
        // the directory.
        for entry in entries {
            let entry = entry.map_err(|source| Error::IoPath {
                path: path.to_owned(),
                context: "read a specific directory entries",
                source,
            })?;
            let entry_path = entry.path();
            let metadata = entry.metadata().map_err(|source| Error::IoPath {
                path: entry_path.to_owned(),
                context: "getting metadata of file",
                source,
            })?;

            // Make sure that the entry is a file. We're only interested in files for now.
            if !metadata.is_file() {
                continue;
            }

            let Some(filename) = entry_path.file_name() else {
                continue;
            };
            filenames.insert(filename.to_string_lossy().to_string());
        }

        if filenames.contains("PKGBUILD") && filenames.contains(".SRCINFO") {
            Ok(LintScope::SourceRepository)
        } else if filenames.contains(".BUILDINFO") && filenames.contains(".PKGINFO") {
            Ok(LintScope::Package)
        } else if filenames.contains("PKGBUILD") {
            Ok(LintScope::PackageBuild)
        } else if filenames.contains(".SRCINFO") {
            Ok(LintScope::SourceInfo)
        } else if filenames.contains(".BUILDINFO") {
            Ok(LintScope::BuildInfo)
        } else if filenames.contains(".PKGINFO") {
            Ok(LintScope::PackageInfo)
        } else {
            Err(Error::NoLintScope {
                path: path.to_path_buf(),
            })
        }
    }

    /// Check whether the [`LintScope`] is a single file.
    pub fn is_single_file(&self) -> bool {
        match self {
            LintScope::SourceRepository | LintScope::Package => false,
            LintScope::BuildInfo
            | LintScope::PackageBuild
            | LintScope::PackageInfo
            | LintScope::SourceInfo => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use rstest::rstest;
    use testresult::{TestError, TestResult};

    use super::*;

    /// Ensure that the correct scope is detected based on existing files in the given directory.
    #[rstest]
    #[case::package(vec!["PKGBUILD", ".SRCINFO"], LintScope::SourceRepository)]
    #[case::package_with_other_files(vec!["test_file", "PKGBUILD", ".SRCINFO", ".BUILDINFO", ".PKGINFO"], LintScope::SourceRepository)]
    #[case::package_build(vec!["PKGBUILD"], LintScope::PackageBuild)]
    #[case::source_info(vec![".SRCINFO"], LintScope::SourceInfo)]
    #[case::source_repo(vec![".BUILDINFO", ".PKGINFO"], LintScope::Package)]
    #[case::source_repo_with_other_files(vec!["test_file", "PKGBUILD", ".BUILDINFO", ".PKGINFO"], LintScope::Package)]
    #[case::build_info(vec![".BUILDINFO"], LintScope::BuildInfo)]
    #[case::package_info(vec![".PKGINFO"], LintScope::PackageInfo)]
    fn detect_scope_in_directory(
        #[case] files: Vec<&'static str>,
        #[case] expected: LintScope,
    ) -> TestResult<()> {
        // Create a temporary directory for testing.
        let tmp_dir = tempfile::tempdir()?;

        // Create all files
        for name in &files {
            let path = tmp_dir.path().join(name);
            File::create(&path)?;
        }

        let scope = LintScope::detect(tmp_dir.path())?;

        assert_eq!(
            scope, expected,
            "Expected '{expected}' scope for file set {files:?}"
        );

        Ok(())
    }

    /// Ensure that the correct scope is detected based on existing files in the given directory.
    #[rstest]
    #[case::unknown_files(vec!["test_file", "test_file2"])]
    #[case::no_files(vec![])]
    fn fail_to_detect_scope_in_directory(#[case] files: Vec<&'static str>) -> TestResult<()> {
        // Create a temporary directory for testing.
        let tmp_dir = tempfile::tempdir()?;

        // Create all files
        for name in &files {
            let path = tmp_dir.path().join(name);
            File::create(&path)?;
        }

        let error = match LintScope::detect(tmp_dir.path()) {
            Ok(scope) => {
                return Err(TestError::from(format!(
                    "Expected an error for scope detection for file set {files:?}, got {scope}"
                )));
            }
            Err(err) => err,
        };

        assert!(
            matches!(error, Error::NoLintScope { .. }),
            "Expected 'NoLintScope' error for file set {files:?}"
        );

        Ok(())
    }

    /// Ensure that the correct scope is detected based on a given single file.
    #[rstest]
    #[case::package_build("PKGBUILD", LintScope::PackageBuild)]
    #[case::source_info(".SRCINFO", LintScope::SourceInfo)]
    #[case::build_info(".BUILDINFO", LintScope::BuildInfo)]
    #[case::package_info(".PKGINFO", LintScope::PackageInfo)]
    fn detect_scope_of_file(
        #[case] file: &'static str,
        #[case] expected: LintScope,
    ) -> TestResult<()> {
        // Create a temporary directory for testing.
        let tmp_dir = tempfile::tempdir()?;

        // Create all files
        let path = tmp_dir.path().join(file);
        File::create(&path)?;

        let scope = LintScope::detect(&path)?;

        assert_eq!(
            scope, expected,
            "Expected '{expected}' scope for file {file:?}"
        );
        assert!(scope.is_single_file());

        Ok(())
    }
}
