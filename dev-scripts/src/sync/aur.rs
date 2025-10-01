//! Handles the download of packages from the AUR.
//!
//! This requires interaction with `git` and the experimental aur.git GitHub mirror.

use std::{
    fs::{File, create_dir_all},
    io::Read,
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use log::{error, info};
use rayon::prelude::*;

use crate::{cmd::ensure_success, ui::get_progress_bar};

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
    pub fn download_packages(&self) -> Result<()> {
        self.update_or_clone()?;
        self.parallel_extract_files()?;
        Ok(())
    }

    /// Ensure we have a bare clone of aur.git locally. Update if already present.
    fn update_or_clone(&self) -> Result<()> {
        if self.repo_dir().exists() {
            info!("Updating aur.git mirror...");
            let output = Command::new("git")
                .arg("-C")
                .arg(self.repo_dir())
                .args(["fetch", "--depth=1", "--prune"])
                .output()
                .context("Failed to update aur.git.")?;
            ensure_success(&output).context("Failed to update aur.git.")?;
        } else {
            create_dir_all(self.download_dir())?;
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
                .context("Failed to clone aur.git.")?;
            ensure_success(&output).context("Failed to clone aur.git.")?;
        }
        Ok(())
    }

    /// Extract .SRCINFO and PKGBUILD files from aur.git branches.
    fn parallel_extract_files(&self) -> Result<()> {
        let packages: Vec<String> = get_packages_list()?;

        let progress_bar = get_progress_bar(packages.len() as u64);

        create_dir_all(self.target_dir())?;

        let results: Vec<Result<()>> = packages
            .par_iter()
            .map(|pkg| {
                let pkg_dir = self.target_dir().join(pkg);
                create_dir_all(&pkg_dir)
                    .context(format!("Failed to create package directory {pkg_dir:?}."))?;

                for file_type in [".SRCINFO", "PKGBUILD"] {
                    let out_file = File::create(pkg_dir.join(file_type))?;
                    let output = Command::new("git")
                        .arg("show")
                        .arg(format!("{pkg}:{file_type}"))
                        .current_dir(self.repo_dir())
                        .stdout(Stdio::from(out_file))
                        .output()
                        .context(format!(
                            "Failed to extract {file_type:?} from AUR package {pkg:?}."
                        ))?;
                    ensure_success(&output).context(format!(
                        "Failed to extract {file_type:?} for package {pkg:?}."
                    ))?;
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
        self.dest.join("download")
    }

    fn repo_dir(&self) -> PathBuf {
        self.download_dir().join("aur")
    }

    fn target_dir(&self) -> PathBuf {
        self.dest.join("aur")
    }
}

/// Downloads pkgbase.gz from aurweb and extracts the package names.
fn get_packages_list() -> Result<Vec<String>> {
    let resp = reqwest::blocking::get(AUR_PKGBASE_URL)?.error_for_status()?;
    let bytes = resp.bytes()?;
    let mut decoder = GzDecoder::new(&bytes[..]);
    let mut aur_packages_raw = String::new();
    decoder.read_to_string(&mut aur_packages_raw)?;
    Ok(aur_packages_raw.lines().map(|s| s.to_string()).collect())
}
