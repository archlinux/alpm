use std::{fs::remove_dir_all, path::PathBuf};

use alpm_common::MetadataFile;
use alpm_pkgbuild::{BridgeOutput, Error};
use alpm_srcinfo::{SourceInfo, SourceInfoV1};
use anyhow::{Context, Result};
use log::warn;
use strum::IntoEnumIterator;

use crate::{
    cli::{self, TestFilesCmd},
    sync::{PackageRepositories, mirror::MirrorDownloader, pkgsrc::PkgSrcDownloader},
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
        cli::TestFilesCmd::Test {
            test_data_dir,
            repositories,
            file_type,
        } => {
            // Set a default download destination.
            let test_data_dir = match test_data_dir {
                Some(test_data_dir) => test_data_dir,
                None => dirs::cache_dir()
                    .context("Failed to determine home user cache directory.")?
                    .join("alpm/testing"),
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
        cli::TestFilesCmd::Download {
            destination,
            repositories,
            source,
        } => {
            // Set a default download destination.
            let dest = match destination {
                Some(dest) => dest,
                None => dirs::cache_dir()
                    .context("Failed to determine home user cache directory.")?
                    .join("alpm/testing"),
            };
            let repositories = PackageRepositories::iter()
                .filter(|v| repositories.clone().is_none_or(|r| r.contains(v)))
                .collect();

            match source {
                cli::DownloadCmd::PkgSrcRepositories {} => {
                    let downloader = PkgSrcDownloader { dest };
                    downloader.download_package_source_repositories()?;
                }
                cli::DownloadCmd::Databases {
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
                cli::DownloadCmd::Packages {
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
        cli::TestFilesCmd::Clean {
            destination,
            source,
        } => {
            // Set a default download destination.
            let dest = match destination {
                Some(dest) => dest,
                None => dirs::cache_dir()
                    .context("Failed to determine home user cache directory.")?
                    .join("alpm/testing"),
            };

            match source {
                cli::CleanCmd::PkgSrcRepositories => {
                    remove_dir_all(dest.join("download").join("pkgsrc"))?;
                    remove_dir_all(dest.join("pkgsrc"))?;
                }
                cli::CleanCmd::Databases => {
                    remove_dir_all(dest.join("download").join("databases"))?;
                    remove_dir_all(dest.join("databases"))?;
                }
                cli::CleanCmd::Packages => {
                    remove_dir_all(dest.join("download").join("packages"))?;
                    remove_dir_all(dest.join("packages"))?;
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
pub fn compare_source_info(pkgbuild_path: PathBuf, srcinfo_path: PathBuf) -> Result<(), Error> {
    let output = BridgeOutput::from_file(&pkgbuild_path)?;
    let pkgbuild_source_info: SourceInfoV1 = output.try_into()?;

    let source_info = SourceInfo::from_file_with_schema(srcinfo_path, None)?;
    let SourceInfo::V1(source_info) = source_info;

    if source_info != pkgbuild_source_info {
        let pkgbuild_source_info = serde_json::to_string_pretty(&pkgbuild_source_info)?;
        let source_info = serde_json::to_string_pretty(&source_info)?;

        let pkgbuild_json_path = PathBuf::from("pkgbuild.json");
        std::fs::write("pkgbuild.json", pkgbuild_source_info).map_err(|source| Error::IoPath {
            path: pkgbuild_json_path,
            context: "writing pkgbuild.json file",
            source,
        })?;
        let srcinfo_json_path = PathBuf::from("srcinfo.json");
        std::fs::write("srcinfo.json", source_info).map_err(|source| Error::IoPath {
            path: srcinfo_json_path,
            context: "writing srcinfo.json file",
            source,
        })?;

        eprintln!(
            "SRCINFO data generated from PKGBUILD file differs from the .SRCINFO file read from disk.\n\
            Compare the two generated files pkgbuild.json and srcinfo.json for details."
        );
        std::process::exit(1);
    } else {
        println!("The generated content matches that read from disk.");
    }

    Ok(())
}
