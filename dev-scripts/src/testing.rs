//! Tests against downloaded artifacts.

use std::{collections::HashSet, fs::read_dir, path::PathBuf, str::FromStr};

use alpm_buildinfo::BuildInfo;
use alpm_common::MetadataFile;
use alpm_mtree::Mtree;
use alpm_pkginfo::PackageInfo;
use alpm_srcinfo::SourceInfo;
use log::{debug, info};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use voa::{
    commands::{
        PurposeAndContext,
        get_technology_settings,
        get_voa_config,
        openpgp_verify,
        read_openpgp_signatures,
        read_openpgp_verifiers,
    },
    core::{Context, Os, Purpose},
    openpgp::ModelBasedVerifier,
    utils::RegularFile,
};

use crate::{
    CacheDir,
    Error,
    cli::TestFileType,
    consts::{AUR_DIR, DATABASES_DIR, DOWNLOAD_DIR, PACKAGES_DIR, PKGSRC_DIR},
    sync::PackageRepositories,
    ui::get_progress_bar,
};

/// Verifies a `file` using a `signature` and a [`ModelBasedVerifier`].
///
/// The success or failure of the verification is transmitted through logging.
///
/// # Errors
///
/// Returns an error if
///
/// - the `signature` cannot be read as an OpenPGP signature
/// - the `file` cannot be read
fn openpgp_verify_file(
    file: PathBuf,
    signature: PathBuf,
    model_verifier: &ModelBasedVerifier,
) -> Result<(), Error> {
    debug!("Verifying {file:?} with {signature:?}");

    let signatures = read_openpgp_signatures(&HashSet::from_iter([RegularFile::try_from(
        signature.clone(),
    )?]))?;

    let check_results = openpgp_verify(
        model_verifier,
        &signatures,
        &RegularFile::try_from(file.clone())?,
    )?;

    // Look at the signer info of all check results and return an error if there is none.
    for check_result in check_results {
        if let Some(signer_info) = check_result.signer_info() {
            debug!(
                "Successfully verified using {} {}",
                signer_info
                    .certificate()
                    .fingerprint()
                    .map_err(voa::Error::VoaOpenPgp)?,
                signer_info.component_fingerprint()
            )
        } else {
            return Err(Error::VoaVerificationFailed {
                file,
                signature,
                context: "".to_string(),
            });
        }
    }

    Ok(())
}

/// This is the entry point for running validation tests of parsers on ALPM metadata files.
#[derive(Clone, Debug)]
pub struct TestRunner {
    /// The directory in which test data is stored.
    pub cache_dir: CacheDir,
    /// The type of file that is targeted in the test.
    pub file_type: TestFileType,
    /// The list of repositories against which the test runs.
    pub repositories: Vec<PackageRepositories>,
}

impl TestRunner {
    /// Run validation on all local test files that have been downloaded via the
    /// `test-files download` command.
    pub fn run_tests(&self) -> Result<(), Error> {
        let test_files = self.find_files_of_type()?;
        info!(
            "Found {} {} files for testing",
            test_files.len(),
            self.file_type
        );

        // Cache the certificates used for VOA-based verification as that significantly increases
        // speed.
        let os = Os::from_str("arch").map_err(voa::Error::VoaCore)?;

        let (artifact_verifiers, anchors) = if matches!(self.file_type, TestFileType::Signatures) {
            let artifact_verifiers = read_openpgp_verifiers(
                os.clone(),
                Purpose::from_str("package").map_err(voa::Error::VoaCore)?,
                Context::Default,
            );
            let anchors = read_openpgp_verifiers(
                os.clone(),
                Purpose::from_str("trust-anchor-package").map_err(voa::Error::VoaCore)?,
                Context::Default,
            );
            (artifact_verifiers, anchors)
        } else {
            (Vec::new(), Vec::new())
        };

        let config = get_voa_config();
        let purpose_and_context = PurposeAndContext::new(
            Some(Purpose::from_str("package").map_err(voa::Error::VoaCore)?),
            Some(Context::Default),
        );
        let openpgp_settings =
            get_technology_settings(&config, &os, purpose_and_context.as_ref()).openpgp_settings();

        let model_verifier =
            ModelBasedVerifier::new(openpgp_settings, &artifact_verifiers, &anchors);

        let progress_bar = get_progress_bar(test_files.len() as u64);

        // Run the validate subcommand for all files in parallel.
        let asserts: Vec<(PathBuf, Result<(), Error>)> = test_files
            .into_par_iter()
            .map(|file| {
                let result = match self.file_type {
                    TestFileType::BuildInfo => BuildInfo::from_file_with_schema(&file, None)
                        .map(|_| ())
                        .map_err(|err| err.into()),
                    TestFileType::SrcInfo => SourceInfo::from_file_with_schema(&file, None)
                        .map(|_| ())
                        .map_err(|err| err.into()),
                    TestFileType::MTree => Mtree::from_file_with_schema(&file, None)
                        .map(|_| ())
                        .map_err(|err| err.into()),
                    TestFileType::PackageInfo => PackageInfo::from_file_with_schema(&file, None)
                        .map(|_| ())
                        .map_err(|err| err.into()),
                    TestFileType::RemoteDesc => unimplemented!(),
                    TestFileType::RemoteFiles => unimplemented!(),
                    TestFileType::LocalDesc => unimplemented!(),
                    TestFileType::LocalFiles => unimplemented!(),
                    TestFileType::Signatures => {
                        let data = {
                            let mut data = file.clone();
                            data.set_extension("");
                            data
                        };

                        openpgp_verify_file(data, file.clone(), &model_verifier)
                    }
                };

                progress_bar.inc(1);
                (file, result)
            })
            .collect();

        // Finish the progress_bar
        progress_bar.finish_with_message("Validation run finished.");

        // Get all files and the respective error for which validation failed.
        let failures: Vec<(PathBuf, Error)> = asserts
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
            return Err(Error::TestFailed {
                failures: failures
                    .iter()
                    .enumerate()
                    .map(|(index, failure)| (index, failure.0.clone(), failure.1.to_string()))
                    .collect::<Vec<_>>(),
            });
        }

        Ok(())
    }

    /// Searches the download directory for all files of the given type.
    ///
    /// Returns a list of Paths that were found in the process.
    pub fn find_files_of_type(&self) -> Result<Vec<PathBuf>, Error> {
        debug!("Searching for files of type {}", self.file_type);

        let mut files = Vec::new();

        // First up, determine which folders we should look at while searching for files.
        let type_folders = match self.file_type {
            // All package related file types are nested in the subdirectories of the respective
            // package's package repository.
            TestFileType::BuildInfo | TestFileType::PackageInfo | TestFileType::MTree => self
                .repositories
                .iter()
                .map(|repo| {
                    self.cache_dir
                        .as_ref()
                        .join(PACKAGES_DIR)
                        .join(repo.to_string())
                })
                .collect(),
            TestFileType::SrcInfo => vec![
                self.cache_dir.as_ref().join(PKGSRC_DIR),
                self.cache_dir.as_ref().join(AUR_DIR),
            ],
            // The `desc` and `files` file types are nested in the subdirectories of the respective
            // package's package repository.
            TestFileType::RemoteDesc | TestFileType::RemoteFiles => self
                .repositories
                .iter()
                .map(|repo| {
                    self.cache_dir
                        .as_ref()
                        .join(DATABASES_DIR)
                        .join(repo.to_string())
                })
                .collect(),
            TestFileType::Signatures => {
                let dirs: Vec<PathBuf> = self
                    .repositories
                    .iter()
                    .map(|repo| {
                        self.cache_dir
                            .as_ref()
                            .join(DOWNLOAD_DIR)
                            .join(PACKAGES_DIR)
                            .join(repo.to_string())
                    })
                    .collect();

                // We return early because we are collecting files based on extension.
                return files_in_dirs_by_extension(
                    dirs.as_slice(),
                    &TestFileType::Signatures.to_string(),
                );
            }
            TestFileType::LocalDesc | TestFileType::LocalFiles => {
                unimplemented!();
            }
        };

        for folder in type_folders {
            debug!("Looking for files in {folder:?}");
            if !folder.exists() {
                info!("The directory {folder:?} doesn't exist, skipping.");
                continue;
            }
            // Each top-level folder contains a number of sub-folders where each sub-folder
            // represents a single package. Check if the file we're interested in exists
            // for said package. If so, add it to the list
            for pkg_folder in read_dir(&folder).map_err(|source| Error::IoPath {
                path: folder.clone(),
                context: "reading entries in directory".to_string(),
                source,
            })? {
                let pkg_folder = pkg_folder.map_err(|source| Error::IoPath {
                    path: folder.clone(),
                    context: "reading an entry of the directory".to_string(),
                    source,
                })?;
                let file_path = pkg_folder.path().join(self.file_type.to_string());
                if file_path.exists() {
                    files.push(file_path);
                }
            }
        }

        Ok(files)
    }
}

/// Collects all regular files in a list of directories and filters them by extension.
///
/// Skips non-existent paths in `dirs`.
/// Only considers regular files, that have a matching `extension`.
///
/// # Errors
///
/// Returns an error if
///
/// - the entries in a directory in one of the paths in `dirs` cannot be read
/// - one of the entries in a directory cannot be read
fn files_in_dirs_by_extension(dirs: &[PathBuf], extension: &str) -> Result<Vec<PathBuf>, Error> {
    let mut files = Vec::new();

    for dir in dirs {
        debug!("Looking for files in {dir:?}");
        if !dir.exists() {
            info!("Skipping directory {dir:?} as it does not exist.");
            continue;
        }

        for entry in read_dir(dir).map_err(|source| Error::IoPath {
            path: dir.clone(),
            context: "reading entries in directory".to_string(),
            source,
        })? {
            let entry = entry.map_err(|source| Error::IoPath {
                path: dir.clone(),
                context: "reading an entry of the directory".to_string(),
                source,
            })?;

            let file_path = entry.path();
            if file_path.is_file()
                && file_path
                    .extension()
                    .is_some_and(|ext| ext.to_str() == Some(extension))
            {
                files.push(file_path);
            }
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        fs::{OpenOptions, create_dir},
    };

    use rstest::rstest;
    use strum::IntoEnumIterator;
    use testresult::TestResult;

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
    fn test_find_files_for_packages(#[case] file_type: TestFileType) -> TestResult {
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
            cache_dir: CacheDir::from(tmp_dir.path().to_owned()),
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
    fn test_find_files_for_databases(#[case] file_type: TestFileType) -> TestResult {
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
            cache_dir: CacheDir::from(tmp_dir.path().to_owned()),
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
    fn test_find_files_for_pkgsrc(#[case] file_type: TestFileType) -> TestResult {
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
            cache_dir: CacheDir::from(tmp_dir.path().to_owned()),
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
