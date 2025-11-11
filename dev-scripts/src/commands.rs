use std::{
    fs::{remove_dir_all, write},
    path::PathBuf,
    process::exit,
};

use alpm_common::MetadataFile;
use alpm_pkgbuild::Error;
use alpm_srcinfo::{SourceInfo, SourceInfoV1};
use anyhow::{Context, Result};
use dirs::cache_dir;
use log::warn;
use serde_json::to_string_pretty;
use strum::IntoEnumIterator;

use crate::{
    cli::{CleanCmd, DownloadCmd, TestFilesCmd},
    consts::{DATABASES_DIR, DOWNLOAD_DIR, PACKAGES_DIR, PKGSRC_DIR, PROJECT_NAME, TESTING_DIR},
    sync::{
        PackageRepositories,
        aur::AurDownloader,
        mirror::MirrorDownloader,
        pkgsrc::PkgSrcDownloader,
    },
    testing::TestRunner,
};

/// Entry point for testing file handling binaries for official ArchLinux packages, source
/// repositories and databases.
///
/// This function relegates to functions that:
/// - Download packages.
/// - Test file parsers on all files.
/// - Clean up downloaded files.
pub(crate) fn test_files(cmd: TestFilesCmd) -> Result<()> {
    match cmd {
        TestFilesCmd::Test {
            test_data_dir,
            repositories,
            file_type,
        } => {
            // Set a default download destination.
            let test_data_dir = match test_data_dir {
                Some(test_data_dir) => test_data_dir,
                None => cache_dir()
                    .context("Failed to determine home user cache directory.")?
                    .join(PROJECT_NAME)
                    .join(TESTING_DIR),
            };
            let repositories = PackageRepositories::iter()
                .filter(|v| repositories.clone().is_none_or(|r| r.contains(v)))
                .collect();
            let runner = TestRunner {
                test_data_dir,
                file_type,
                repositories,
            };
            runner.run_tests()?;
        }
        TestFilesCmd::Download {
            destination,
            repositories,
            source,
        } => {
            // Set a default download destination.
            let dest = match destination {
                Some(dest) => dest,
                None => cache_dir()
                    .context("Failed to determine home user cache directory.")?
                    .join(PROJECT_NAME)
                    .join(TESTING_DIR),
            };
            let repositories = PackageRepositories::iter()
                .filter(|v| repositories.clone().is_none_or(|r| r.contains(v)))
                .collect();

            match source {
                DownloadCmd::PkgSrcRepositories {} => {
                    let downloader = PkgSrcDownloader { dest };
                    downloader.download_package_source_repositories()?;
                }
                DownloadCmd::Aur {} => {
                    let downloader = AurDownloader { dest };
                    downloader.download_packages()?;
                }
                DownloadCmd::Databases {
                    mirror,
                    force_extract,
                } => {
                    let downloader = MirrorDownloader {
                        dest,
                        mirror,
                        repositories,
                        extract_all: force_extract,
                    };
                    warn!(
                        "Beginning database retrieval\nIf the process is unexpectedly halted, rerun with `--force-extract` flag"
                    );
                    downloader.sync_remote_databases()?;
                }
                DownloadCmd::Packages {
                    mirror,
                    force_extract,
                } => {
                    let downloader = MirrorDownloader {
                        dest,
                        mirror,
                        repositories,
                        extract_all: force_extract,
                    };
                    warn!(
                        "Beginning package retrieval\nIf the process is unexpectedly halted, rerun with `--force-extract` flag"
                    );
                    downloader.sync_remote_packages()?;
                }
            };
        }
        TestFilesCmd::Clean {
            destination,
            source,
        } => {
            // Set a default download destination.
            let dest = match destination {
                Some(dest) => dest,
                None => cache_dir()
                    .context("Failed to determine home user cache directory.")?
                    .join(PROJECT_NAME)
                    .join(TESTING_DIR),
            };

            match source {
                CleanCmd::PkgSrcRepositories => {
                    remove_dir_all(dest.join(DOWNLOAD_DIR).join(PKGSRC_DIR))?;
                    remove_dir_all(dest.join(PKGSRC_DIR))?;
                }
                CleanCmd::Databases => {
                    remove_dir_all(dest.join(DOWNLOAD_DIR).join(DATABASES_DIR))?;
                    remove_dir_all(dest.join(DATABASES_DIR))?;
                }
                CleanCmd::Packages => {
                    remove_dir_all(dest.join(DOWNLOAD_DIR).join(PACKAGES_DIR))?;
                    remove_dir_all(dest.join(PACKAGES_DIR))?;
                }
            };
        }
    }

    Ok(())
}

/// Run the `alpm-pkgbuild srcinfo format` command on a PKGBUILD and compare its output with a
/// given .SRCINFO file.
///
/// If the generated and read [`SRCINFO`] representations do not match, the respective files
/// `pkgbuild.json` and `srcinfo.json` are output to the current working directory and the function
/// exits with an exit code of `1`.
///
/// These files contain pretty-printed JSON, which accurately depicts the internal representation
/// used to compare the two files.
///
/// # Errors
///
/// Returns an error if
///
/// - running the [`alpm-pkgbuild-bridge`] script fails,
/// - creating a [`SourceInfoV1`] from the script output fails,
/// - creating a [`SourceInfo`] from the the [`SRCINFO`] file fails,
/// - or creating JSON representations for either [`SRCINFO`] data fails in case of mismatch.
///
/// [`PKGBUILD`]: https://man.archlinux.org/man/PKGBUILD.5
/// [`SRCINFO`]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
/// [`alpm-pkgbuild-bridge`]: https://gitlab.archlinux.org/archlinux/alpm/alpm-pkgbuild-bridge
pub fn compare_source_info(pkgbuild_path: PathBuf, srcinfo_path: PathBuf) -> Result<()> {
    let pkgbuild_source_info: SourceInfoV1 = SourceInfoV1::from_pkgbuild(&pkgbuild_path)?;

    let source_info = SourceInfo::from_file_with_schema(srcinfo_path, None)?;
    let SourceInfo::V1(source_info) = source_info;

    if source_info != pkgbuild_source_info {
        let pkgbuild_source_info = to_string_pretty(&pkgbuild_source_info)?;
        let source_info = to_string_pretty(&source_info)?;

        let pkgbuild_json_path = PathBuf::from("pkgbuild.json");
        write("pkgbuild.json", pkgbuild_source_info).map_err(|source| Error::IoPath {
            path: pkgbuild_json_path,
            context: "writing pkgbuild.json file",
            source,
        })?;
        let srcinfo_json_path = PathBuf::from("srcinfo.json");
        write("srcinfo.json", source_info).map_err(|source| Error::IoPath {
            path: srcinfo_json_path,
            context: "writing srcinfo.json file",
            source,
        })?;

        eprintln!(
            "SRCINFO data generated from PKGBUILD file differs from the .SRCINFO file read from disk.\n\
            Compare the two generated files pkgbuild.json and srcinfo.json for details."
        );
        exit(1);
    } else {
        println!("The generated content matches that read from disk.");
    }

    Ok(())
}
