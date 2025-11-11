//! Handles the download of packages from the AUR.
//!
//! This requires interaction with `git` and the experimental aur.git GitHub mirror.

use std::{
    fs::{File, create_dir_all},
    io::Read,
    path::PathBuf,
    process::{Command, Stdio},
};

use alpm_types::{PKGBUILD_FILE_NAME, SRCINFO_FILE_NAME};
use flate2::read::GzDecoder;
use log::{error, info};
use rayon::prelude::*;
use reqwest::blocking::get;

use crate::{
    Error,
    cmd::ensure_success,
    consts::{AUR_DIR, DOWNLOAD_DIR},
    ui::get_progress_bar,
};

const AUR_PKGBASE_URL: &str = "https://aur.archlinux.org/pkgbase.gz";
const AUR_GIT_MIRROR_URL: &str = "https://github.com/archlinux/aur.git";

/// The entry point for downloading packages from the AUR.
///
/// See [`AurDownloader::download_packages`] for more information.
#[derive(Clone, Debug)]
pub struct AurDownloader {
    /// The destination folder into which files should be downloaded.
    pub dest: PathBuf,
}

impl AurDownloader {
    /// Clone (or update) the AUR git repo and extract
    /// .SRCINFO + PKGBUILD for every package
    pub fn download_packages(&self) -> Result<(), Error> {
        self.update_or_clone()?;
        self.parallel_extract_files()?;
        Ok(())
    }

    /// Ensure we have a bare clone of aur.git locally. Update if already present.
    fn update_or_clone(&self) -> Result<(), Error> {
        if self.repo_dir().exists() {
            info!("Updating aur.git mirror...");
            let output = Command::new("git")
                .arg("-C")
                .arg(self.repo_dir())
                .args(["fetch", "--depth=1", "--prune"])
                .output()
                .map_err(|source| Error::Io {
                    context: "fetching latest AUR git sources".to_string(),
                    source,
                })?;
            ensure_success(&output, "Fetching latest AUR git sources".to_string())?;
        } else {
            create_dir_all(self.download_dir()).map_err(|source| Error::IoPath {
                path: self.download_dir(),
                context: "recursively creating the directory".to_string(),
                source,
            })?;
            info!("Cloning aur.git mirror...");
            let output = Command::new("git")
                .args([
                    // We want all remote branches locally,
                    // using a mirror clone is the simplest solution.
                    "clone",
                    "--mirror",
                    "--depth=1",
                    "--no-single-branch",
                    AUR_GIT_MIRROR_URL,
                ])
                .arg(self.repo_dir())
                .output()
                .map_err(|source| Error::Io {
                    context: "cloning AUR git sources".to_string(),
                    source,
                })?;
            ensure_success(&output, "Cloning AUR git sources".to_string())?;
        }
        Ok(())
    }

    /// Extract .SRCINFO and PKGBUILD files from aur.git branches.
    fn parallel_extract_files(&self) -> Result<(), Error> {
        let packages: Vec<String> = get_packages_list()?;

        let progress_bar = get_progress_bar(packages.len() as u64);

        create_dir_all(self.target_dir()).map_err(|source| Error::IoPath {
            path: self.download_dir(),
            context: "recursively creating the directory".to_string(),
            source,
        })?;

        let results: Vec<Result<(), Error>> = packages
            .par_iter()
            .map(|pkg| {
                let pkg_dir = self.target_dir().join(pkg);
                create_dir_all(&pkg_dir).map_err(|source| Error::IoPath {
                    path: pkg_dir.clone(),
                    context: "recursively creating the directory".to_string(),
                    source,
                })?;

                for file_type in [SRCINFO_FILE_NAME, PKGBUILD_FILE_NAME] {
                    let out_file =
                        File::create(pkg_dir.join(file_type)).map_err(|source| Error::IoPath {
                            path: pkg_dir.join(file_type),
                            context: "recursively creating the directory".to_string(),
                            source,
                        })?;
                    let output = Command::new("git")
                        .arg("show")
                        .arg(format!("{pkg}:{file_type}"))
                        .current_dir(self.repo_dir())
                        .stdout(Stdio::from(out_file))
                        .output()
                        .map_err(|source| Error::Io {
                            context: format!("extracting {file_type:?} from AUR repo {pkg:?}"),
                            source,
                        })?;
                    ensure_success(
                        &output,
                        format!("Extracting {file_type:?} from AUR repo {pkg:?}"),
                    )?;
                }
                progress_bar.inc(1);
                Ok(())
            })
            .collect();

        progress_bar.finish_with_message("All files extracted.");

        // Log all errors during parallel extraction.
        for error in results.into_iter().filter_map(Result::err) {
            error!("{error:?}");
        }

        Ok(())
    }

    fn download_dir(&self) -> PathBuf {
        self.dest.join(DOWNLOAD_DIR)
    }

    fn repo_dir(&self) -> PathBuf {
        self.download_dir().join(AUR_DIR)
    }

    fn target_dir(&self) -> PathBuf {
        self.dest.join(AUR_DIR)
    }
}

/// Downloads pkgbase.gz from aurweb and extracts the package names.
fn get_packages_list() -> Result<Vec<String>, Error> {
    let resp = get(AUR_PKGBASE_URL)
        .map_err(|source| Error::HttpQueryFailed {
            context: "retrieving the list of AUR packages".to_string(),
            source,
        })?
        .error_for_status()
        .map_err(|source| Error::HttpQueryFailed {
            context: "retrieving the list of AUR packages".to_string(),
            source,
        })?;
    let bytes = resp.bytes().map_err(|source| Error::HttpQueryFailed {
        context: "retrieving the response body as bytes for the list of AUR packages".to_string(),
        source,
    })?;
    let mut decoder = GzDecoder::new(&bytes[..]);
    let mut aur_packages_raw = String::new();
    decoder
        .read_to_string(&mut aur_packages_raw)
        .map_err(|source| Error::Io {
            context:
                "reading the gzip decoded response body for the list of AUR packages as string"
                    .to_string(),
            source,
        })?;
    Ok(aur_packages_raw.lines().map(|s| s.to_string()).collect())
}
