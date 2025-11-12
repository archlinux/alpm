use std::{
    fs::{remove_dir_all, write},
    path::PathBuf,
    process::exit,
};

use alpm_common::MetadataFile;
use alpm_srcinfo::{SourceInfo, SourceInfoV1};
use log::warn;
use serde_json::to_string_pretty;
use strum::IntoEnumIterator;

use crate::{
    CacheDir,
    Error,
    cli::{CleanCmd, DownloadCmd, TestFilesCmd},
    consts::{DATABASES_DIR, DOWNLOAD_DIR, PACKAGES_DIR, PKGSRC_DIR},
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
pub(crate) fn test_files(cmd: TestFilesCmd, cache_dir: CacheDir) -> Result<(), Error> {
    match cmd {
        TestFilesCmd::Test {
            repositories,
            file_type,
        } => {
            let repositories = PackageRepositories::iter()
                .filter(|v| repositories.clone().is_none_or(|r| r.contains(v)))
                .collect();
            let runner = TestRunner {
                cache_dir,
                file_type,
                repositories,
            };
            runner.run_tests()?;
        }
        TestFilesCmd::Download {
            repositories,
            source,
        } => {
            let repositories = PackageRepositories::iter()
                .filter(|v| repositories.clone().is_none_or(|r| r.contains(v)))
                .collect();

            match source {
                DownloadCmd::PkgSrcRepositories {} => {
                    let downloader = PkgSrcDownloader { cache_dir };
                    downloader.download_package_source_repositories()?;
                }
                DownloadCmd::Aur {} => {
                    let downloader = AurDownloader { cache_dir };
                    downloader.download_packages()?;
                }
                DownloadCmd::Databases {
                    mirror,
                    force_extract,
                } => {
                    let downloader = MirrorDownloader {
                        cache_dir,
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
                        cache_dir,
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
        TestFilesCmd::Clean { source } => {
            let dirs = match source {
                CleanCmd::PkgSrcRepositories => [
                    cache_dir.as_ref().join(DOWNLOAD_DIR).join(PKGSRC_DIR),
                    cache_dir.as_ref().join(PKGSRC_DIR),
                ],
                CleanCmd::Databases => [
                    cache_dir.as_ref().join(DOWNLOAD_DIR).join(DATABASES_DIR),
                    cache_dir.as_ref().join(DATABASES_DIR),
                ],
                CleanCmd::Packages => [
                    cache_dir.as_ref().join(DOWNLOAD_DIR).join(PACKAGES_DIR),
                    cache_dir.as_ref().join(PACKAGES_DIR),
                ],
            };

            for dir in dirs {
                remove_dir_all(&dir).map_err(|source| Error::IoPath {
                    path: dir,
                    context: "recursively deleting the directory".to_string(),
                    source,
                })?;
            }
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
    let pkgbuild_source_info: SourceInfoV1 = SourceInfoV1::from_pkgbuild(&pkgbuild_path)?;

    let source_info = SourceInfo::from_file_with_schema(srcinfo_path, None)?;
    let SourceInfo::V1(source_info) = source_info;

    if source_info != pkgbuild_source_info {
        let pkgbuild_source_info =
            to_string_pretty(&pkgbuild_source_info).map_err(|source| Error::Json {
                context: "deserializing a PKGBUILD  based SRCINFO as pretty JSON".to_string(),
                source,
            })?;
        let source_info = to_string_pretty(&source_info).map_err(|source| Error::Json {
            context: "deserializing a SRCINFO as pretty JSON".to_string(),
            source,
        })?;

        let pkgbuild_json_path = PathBuf::from("pkgbuild.json");
        write("pkgbuild.json", pkgbuild_source_info).map_err(|source| Error::IoPath {
            path: pkgbuild_json_path,
            context: "writing pkgbuild.json to file".to_string(),
            source,
        })?;
        let srcinfo_json_path = PathBuf::from("srcinfo.json");
        write("srcinfo.json", source_info).map_err(|source| Error::IoPath {
            path: srcinfo_json_path,
            context: "writing srcinfo.json to file".to_string(),
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
