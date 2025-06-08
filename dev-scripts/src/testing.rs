//! Tests against downloaded artifacts.

use std::path::PathBuf;

use alpm_buildinfo::cli::ValidateArgs;
use anyhow::{Context, Result};
use colored::Colorize;
use log::{debug, info};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{cli::TestFileType, sync::PackageRepositories, ui::get_progress_bar};

static PKGSRC_DIR: &str = "pkgsrc";
static PACKAGES_DIR: &str = "packages";
static DATABASES_DIR: &str = "databases";

/// This is the entry point for running validation tests of parsers on ALPM metadata files.
pub struct TestRunner {
    /// The directory in which test data is stored.
    pub test_data_dir: PathBuf,
    /// The type of file that is targeted in the test.
    pub file_type: TestFileType,
    /// The list of repositories against which the test runs.
    pub repositories: Vec<PackageRepositories>,
}

impl TestRunner {
    /// Run validation on all local test files that have been downloaded via the
    /// `test-files download` command.
    pub fn run_tests(&self) -> Result<()> {
        let test_files = self.find_files_of_type().context(format!(
            "Failed to detect files for type {}",
            self.file_type
        ))?;
        info!(
            "Found {} {} files for testing",
            test_files.len(),
            self.file_type
        );

        let progress_bar = get_progress_bar(test_files.len() as u64);

        // Run the validate subcommand for all files in parallel.
        let asserts: Vec<(PathBuf, Result<()>)> = test_files
            .into_par_iter()
            .map(|file| {
                let result = match self.file_type {
                    TestFileType::BuildInfo => alpm_buildinfo::commands::validate(ValidateArgs {
                        file: Some(file.clone()),
                        schema: None,
                    })
                    .map_err(|err| err.into()),
                    TestFileType::SrcInfo => alpm_srcinfo::commands::validate(Some(&file), None)
                        .map_err(|err| err.into()),
                    TestFileType::MTree => {
                        alpm_mtree::commands::validate(Some(&file), None).map_err(|err| err.into())
                    }
                    TestFileType::PackageInfo => {
                        alpm_pkginfo::commands::validate(Some(file.clone()), None)
                            .map_err(|err| err.into())
                    }
                    TestFileType::RemoteDesc => unimplemented!(),
                    TestFileType::RemoteFiles => unimplemented!(),
                    TestFileType::LocalDesc => unimplemented!(),
                    TestFileType::LocalFiles => unimplemented!(),
                };

                progress_bar.inc(1);
                (file, result)
            })
            .collect();

        // Finish the progress_bar
        progress_bar.finish_with_message("Validation run finished.");

        // Get all files and the respective error for which validation failed.
        let failures: Vec<(PathBuf, anyhow::Error)> = asserts
            .into_iter()
            .filter_map(|(path, result)| {
                if let Err(err) = result {
                    Some((path, err))
                } else {
                    None
                }
            })
            .collect();

        if !failures.is_empty() {
            for (index, failure) in failures.into_iter().enumerate() {
                let index = format!("[{index}]").bold().red();
                info!(
                    "{index} {} with error:\n {}\n",
                    failure.0.to_string_lossy().bold(),
                    failure.1
                );
            }
        }

        Ok(())
    }

    /// Searches the download directory for all files of the given type.
    ///
    /// Returns a list of Paths that were found in the process.
    pub fn find_files_of_type(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        // First up, determine which folders we should look at while searching for files.
        let type_folders = match self.file_type {
            // All package related file types are nested in the subdirectories of the respective
            // package's package repository.
            TestFileType::BuildInfo | TestFileType::PackageInfo | TestFileType::MTree => self
                .repositories
                .iter()
                .map(|repo| self.test_data_dir.join(PACKAGES_DIR).join(repo.to_string()))
                .collect(),
            TestFileType::SrcInfo => vec![self.test_data_dir.join(PKGSRC_DIR)],
            // The `desc` and `files` file types are nested in the subdirectories of the respective
            // package's package repository.
            TestFileType::RemoteDesc | TestFileType::RemoteFiles => self
                .repositories
                .iter()
                .map(|repo| {
                    self.test_data_dir
                        .join(DATABASES_DIR)
                        .join(repo.to_string())
                })
                .collect(),
            TestFileType::LocalDesc | TestFileType::LocalFiles => {
                unimplemented!();
            }
        };

        for folder in type_folders {
            debug!("Looking for files in {folder:?}");
            // Each top-level folder contains a number of sub-folders where each sub-folder
            // represents a single package. Check if the file we're interested in exists
            // for said package. If so, add it to the list
            for pkg_folder in
                std::fs::read_dir(&folder).context(format!("Failed to read folder {folder:?}"))?
            {
                let pkg_folder = pkg_folder?;
                let file_path = pkg_folder.path().join(self.file_type.to_string());
                if file_path.exists() {
                    files.push(file_path);
                }
            }
        }

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        fs::{OpenOptions, create_dir},
    };

    use rstest::rstest;
    use strum::IntoEnumIterator;

    use super::*;

    const PKG_NAMES: &[&str] = &[
        "pipewire-alsa-1:1.0.7-1-x86_64",
        "xorg-xvinfo-1.1.5-1-x86_64",
        "acl-2.3.2-1-x86_64",
        "archlinux-keyring-20240520-1-any",
    ];

    /// Ensure that files can be found in case they're nested inside
    /// sub-subdirectories if the directory structure is:
    /// `target-dir/packages/${pacman-repo}/${package-name}`
    #[rstest]
    #[case(TestFileType::BuildInfo)]
    #[case(TestFileType::PackageInfo)]
    #[case(TestFileType::MTree)]
    fn test_find_files_for_packages(#[case] file_type: TestFileType) -> Result<()> {
        // Create a temporary directory for testing.
        let tmp_dir = tempfile::tempdir()?;
        let packages_dir = tmp_dir.path().join(PACKAGES_DIR);
        create_dir(&packages_dir)?;

        // The list of files we're expecting to find.
        let mut expected_files = HashSet::new();

        // Create a test file for each repo.
        for (index, repo) in PackageRepositories::iter().enumerate() {
            // Create the repository folder
            let repo_dir = packages_dir.join(repo.to_string());
            create_dir(&repo_dir)?;

            // Create a package subfolder inside that repository folder.
            let pkg = PKG_NAMES[index];
            let pkg_dir = repo_dir.join(pkg);
            create_dir(&pkg_dir)?;

            // Touch the file inside the package folder.
            let file_path = pkg_dir.join(file_type.to_string());
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&file_path)?;
            expected_files.insert(file_path);
        }

        // Run the logic to find the files in question.
        let runner = TestRunner {
            test_data_dir: tmp_dir.path().to_owned(),
            file_type,
            repositories: PackageRepositories::iter().collect(),
        };
        let found_files = HashSet::from_iter(runner.find_files_of_type()?.into_iter());

        assert_eq!(
            found_files, expected_files,
            "Expected that all created package files are also found."
        );

        Ok(())
    }

    /// Ensure that files can be found in case they're nested inside
    /// sub-subdirectories if the directory structure is:
    /// `target-dir/databases/${pacman-repo}/${package-name}`
    #[rstest]
    #[case(TestFileType::RemoteFiles)]
    #[case(TestFileType::RemoteDesc)]
    fn test_find_files_for_databases(#[case] file_type: TestFileType) -> Result<()> {
        // Create a temporary directory for testing.
        let tmp_dir = tempfile::tempdir()?;
        let databases_dir = tmp_dir.path().join(DATABASES_DIR);
        create_dir(&databases_dir)?;

        // The list of files we're expecting to find.
        let mut expected_files = HashSet::new();

        // Create a test file for each repo.
        for (index, repo) in PackageRepositories::iter().enumerate() {
            // Create the repository folder
            let repo_dir = databases_dir.join(repo.to_string());
            create_dir(&repo_dir)?;

            // Create a package subfolder inside that repository folder.
            let pkg = PKG_NAMES[index];
            let pkg_dir = repo_dir.join(pkg);
            create_dir(&pkg_dir)?;

            // Touch the file inside the package folder.
            let file_path = pkg_dir.join(file_type.to_string());
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&file_path)?;
            expected_files.insert(file_path);
        }

        // Run the logic to find the files in question.
        let runner = TestRunner {
            test_data_dir: tmp_dir.path().to_owned(),
            file_type,
            repositories: PackageRepositories::iter().collect(),
        };
        let found_files = HashSet::from_iter(runner.find_files_of_type()?.into_iter());

        assert_eq!(
            found_files, expected_files,
            "Expected that all created databases files are also found."
        );

        Ok(())
    }

    /// Ensure that files can be found in case they're nested inside
    /// sub-subdirectories if the directory structure is:
    /// `target-dir/pkgsrc/${package-name}`
    #[rstest]
    #[case(TestFileType::SrcInfo)]
    fn test_find_files_for_pkgsrc(#[case] file_type: TestFileType) -> Result<()> {
        // Create a temporary directory for testing.
        let tmp_dir = tempfile::tempdir()?;
        let pkgsrc_dir = tmp_dir.path().join(PKGSRC_DIR);
        create_dir(&pkgsrc_dir)?;

        // The list of files we're expecting to find.
        let mut expected_files = HashSet::new();

        // Create one subdirectory for each package name.
        // Then create the file in question for that package.
        for pkg in PKG_NAMES {
            let pkg_dir = pkgsrc_dir.join(pkg);
            create_dir(&pkg_dir)?;

            // Touch the file inside the package folder.
            let file_path = pkg_dir.join(file_type.to_string());
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&file_path)?;
            expected_files.insert(file_path);
        }

        // Run the logic to find the files in question.
        let runner = TestRunner {
            test_data_dir: tmp_dir.path().to_owned(),
            file_type,
            repositories: PackageRepositories::iter().collect(),
        };
        let found_files = HashSet::from_iter(runner.find_files_of_type()?.into_iter());

        assert_eq!(
            found_files, expected_files,
            "Expected that all created pkgsrc files are also found."
        );

        Ok(())
    }
}
