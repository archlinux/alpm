//! Representation and handling of linting scopes.

use std::{
    collections::HashSet,
    fmt::Display,
    fs::{metadata, read_dir},
    path::Path,
};

use alpm_types::{MetadataFileName, PKGBUILD_FILE_NAME, SRCINFO_FILE_NAME};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use strum::{Display as StrumDisplay, VariantArray};

use crate::Error;

/// The fully qualified name of a lint rule.
///
/// A [`ScopedName`] combines the [`LintScope`] with the rule’s identifier,
/// forming a unique name in the format `{scope}::{name}`.
///
/// # Examples
///
/// ```
/// use alpm_lint::{LintScope, ScopedName};
///
/// let name = ScopedName::new(LintScope::SourceRepository, "my_rule");
/// assert_eq!("source_repository::my_rule", name.to_string());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ScopedName {
    scope: LintScope,
    name: &'static str,
}

impl ScopedName {
    /// Create a new instance of [`ScopedName`]
    pub fn new(scope: LintScope, name: &'static str) -> Self {
        Self { scope, name }
    }
}

impl Display for ScopedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.scope, self.name)
    }
}

/// The possible scope used to categorize lint rules.
///
/// Scopes are used to determine what lints should be executed based on a specific linting
/// operation. For example, selecting [`LintScope::SourceInfo`] will run all
/// [`SourceInfo`](alpm_srcinfo::SourceInfo) specific linting rules. Linting scopes can also be
/// fully enabled or disabled via configuration files.
#[derive(
    Clone, Copy, Debug, Deserialize, PartialEq, Serialize, StrumDisplay, ValueEnum, VariantArray,
)]
#[strum(serialize_all = "snake_case")]
pub enum LintScope {
    /// Lint rules with this scope are specific to an [alpm-source-repo].
    ///
    /// Such lint rules check the consistency of an Arch Linux package source repository.
    /// This includes the consistency of data between several metadata files.
    ///
    /// When this scope is selected, the following lint scopes are implied:
    /// - [`LintScope::PackageBuild`]
    /// - [`LintScope::SourceInfo`]
    ///
    /// [alpm-source-repo]: https://alpm.archlinux.page/specifications/alpm-source-repo.7.html
    SourceRepository,
    /// Lint rules with this scope are specific to an [alpm-package].
    ///
    /// Such lint rules check the consistency of an Arch Linux package file.
    /// This includes the consistency of data between various metadata files.
    ///
    /// When this scope is selected, the following lint scopes are implied:
    /// - [`LintScope::PackageInfo`]
    /// - [`LintScope::BuildInfo`]
    ///
    /// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
    Package,
    /// Lint rules with this scope are specific to a single [BUILDINFO] file.
    ///
    /// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
    BuildInfo,
    /// Lint rules with this scope are specific to a single [PKGBUILD] file.
    ///
    /// [PKGBUILD]: https://man.archlinux.org/man/PKGBUILD.5
    PackageBuild,
    /// Lint rules with this scope are specific to a single [PKGINFO] file.
    ///
    /// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
    PackageInfo,
    /// Lint rules with this scope are specific to a single [SRCINFO] file.
    ///
    /// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
    SourceInfo,
}

impl LintScope {
    /// Determines whether a [`LintScope`] contains or matches another.
    ///
    /// In this context "contains" and "matches" means that either `self` is identical to `other`,
    /// or that the scope of `other` is contained in the scope of `self`.
    ///
    /// # Examples
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

    /// Attempts to return all applicable lint scopes based on a provided `path`.
    ///
    /// Usually, when calling `alpm-lint check`, [`LintScope::detect`] is used to
    /// automatically determine the available linting scope based on files in the specified
    /// directory. The current scope can also be overridden by the user.
    ///
    /// Based on that scope, files will be loaded and linting rules are selected for execution.
    ///
    /// # Errors
    ///
    /// - The path cannot be read/accessed
    /// - The scope cannot be determined based on the file/s at the given path.
    pub fn detect(path: &Path) -> Result<LintScope, Error> {
        // `metadata` automatically follows symlinks, so we get the target's metadata
        let metadata = metadata(path).map_err(|source| Error::IoPath {
            path: path.to_owned(),
            context: "getting metadata of path",
            source,
        })?;

        // Handle the case where the path is a single file.
        if metadata.is_file() {
            let filename = path.file_name().ok_or(Error::NoLintScope {
                path: path.to_owned(),
            })?;

            // Package source repository related scopes
            if filename == alpm_types::PKGBUILD_FILE_NAME {
                return Ok(LintScope::PackageBuild);
            } else if filename == alpm_types::SRCINFO_FILE_NAME {
                return Ok(LintScope::SourceInfo);
            // Package related scopes
            } else if filename == Into::<&'static str>::into(MetadataFileName::BuildInfo) {
                return Ok(LintScope::BuildInfo);
            } else if filename == Into::<&'static str>::into(MetadataFileName::PackageInfo) {
                return Ok(LintScope::PackageInfo);
            } else {
                return Err(Error::NoLintScope {
                    path: path.to_path_buf(),
                });
            }
        }

        // At this point, we know that this is a directory.
        // Look at the contained files and try to figure out which scope fits best.

        let entries = read_dir(path).map_err(|source| Error::IoPath {
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

        if filenames.contains(PKGBUILD_FILE_NAME) && filenames.contains(SRCINFO_FILE_NAME) {
            Ok(LintScope::SourceRepository)
        } else if filenames.contains(MetadataFileName::BuildInfo.into())
            && filenames.contains(MetadataFileName::PackageInfo.into())
        {
            Ok(LintScope::Package)
        } else if filenames.contains(PKGBUILD_FILE_NAME) {
            Ok(LintScope::PackageBuild)
        } else if filenames.contains(SRCINFO_FILE_NAME) {
            Ok(LintScope::SourceInfo)
        } else if filenames.contains(MetadataFileName::BuildInfo.into()) {
            Ok(LintScope::BuildInfo)
        } else if filenames.contains(MetadataFileName::PackageInfo.into()) {
            Ok(LintScope::PackageInfo)
        } else {
            Err(Error::NoLintScope {
                path: path.to_path_buf(),
            })
        }
    }

    /// Checks whether the [`LintScope`] is for a single file.
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
    use testresult::TestResult;

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
                panic!("Expected an error for scope detection for file set {files:?}, got {scope}");
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
