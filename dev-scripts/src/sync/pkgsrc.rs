//! Handles the download of package source repositories.
//!
//! This requires interaction with `git` and the official Arch Linux Gitlab, where all of the
//! package source repositories for the official packages are located.

use std::{
    collections::{HashMap, HashSet},
    fs::{copy, create_dir_all, remove_dir_all},
    path::{Path, PathBuf},
    process::Command,
};

use alpm_types::{PKGBUILD_FILE_NAME, SRCINFO_FILE_NAME};
use log::{error, info, trace};
use rayon::prelude::*;
use reqwest::blocking::get;

use super::filenames_in_dir;
use crate::{
    Error,
    cmd::ensure_success,
    consts::{DOWNLOAD_DIR, PKGSRC_DIR},
    ui::get_progress_bar,
};

const PKGBASE_MAINTAINER_URL: &str = "https://archlinux.org/packages/pkgbase-maintainer";
const SSH_HOST: &str = "git@gitlab.archlinux.org";
const REPO_BASE_URL: &str = "archlinux/packaging/packages";

/// Some package repositories' names differ from the name of the package.
/// These are only few and need to be handled separately.
const PACKAGE_REPO_RENAMES: [(&str, &str); 3] = [
    ("gtk2+extra", "gtk2-extra"),
    ("dvd+rw-tools", "dvd-rw-tools"),
    ("tree", "unix-tree"),
];

/// This struct is the entry point for downloading package source repositories from ArchLinux's
/// Gitlab.
///
/// Look at [Self::download_package_source_repositories] for more information.
#[derive(Clone, Debug)]
pub struct PkgSrcDownloader {
    /// The destination folder into which files should be downloaded.
    pub dest: PathBuf,
}

impl PkgSrcDownloader {
    /// Download all official package source git repositories.
    pub fn download_package_source_repositories(&self) -> Result<(), Error> {
        // Query the arch web API to get a list all official active repositories
        // The returned json is a map where the keys are the package names
        // and the value is a list of maintainer names.
        let repos = get(PKGBASE_MAINTAINER_URL)
            .map_err(|source| Error::HttpQueryFailed {
                context: "retrieving the list of pkgbases".to_string(),
                source,
            })?
            .json::<HashMap<String, Vec<String>>>()
            .map_err(|source| Error::HttpQueryFailed {
                context: "deserializing the response as JSON".to_string(),
                source,
            })?;

        let all_repo_names: Vec<String> = repos.keys().map(String::from).collect();
        info!("Found {} official packages.", all_repo_names.len());

        let download_dir = self.dest.join(DOWNLOAD_DIR).join(PKGSRC_DIR);

        // Remove all old repos before trying to update them.
        self.remove_old_repos(&all_repo_names, &download_dir)?;

        // Copy all .SRCINFO files to the target directory
        self.parallel_update_or_clone(&all_repo_names, &download_dir)?;

        // Copy .SRCINFO and PKGBUILD files to the target directory
        for repo in all_repo_names {
            let download_path = download_dir.join(&repo);
            for file in [SRCINFO_FILE_NAME, PKGBUILD_FILE_NAME] {
                if download_path.join(file).exists() {
                    let target_dir = self.dest.join(PKGSRC_DIR).join(&repo);
                    create_dir_all(&target_dir).map_err(|source| Error::IoPath {
                        path: target_dir.to_path_buf(),
                        context: "recursively creating a directory".to_string(),
                        source,
                    })?;
                    copy(download_path.join(file), target_dir.join(file)).map_err(|source| {
                        Error::IoPath {
                            path: download_path.join(file),
                            context: "copying the file to the target directory".to_string(),
                            source,
                        }
                    })?;
                }
            }
        }

        Ok(())
    }

    /// Remove all local repositories for packages that no longer exist in the official
    /// repositories.
    ///
    /// Get the list of all locally available pkgsrc repositories.
    /// If we find any that are not in the list of official packages, we remove them.
    fn remove_old_repos(&self, repos: &[String], download_dir: &Path) -> Result<(), Error> {
        // First up, read the names of all repositories in the local download folder.
        let local_repositories = filenames_in_dir(download_dir)?;

        // Get the list of packages that no longer exist on the mirrors (and thereby archweb).
        let remote_pkgs: HashSet<String> = HashSet::from_iter(repos.iter().map(String::from));

        // Now remove all local repositories for which there's no longer an entry in the archweb
        // response, as those packages are no longer served by the official mirrors.
        let removed_pkgs: Vec<&String> = local_repositories.difference(&remote_pkgs).collect();

        // Delete the repositories
        if !removed_pkgs.is_empty() {
            info!("Found {} repositories for cleanup:", removed_pkgs.len());
            for removed in removed_pkgs {
                remove_dir_all(download_dir.join(removed)).map_err(|source| Error::IoPath {
                    path: download_dir.join(removed),
                    context: "removing the file".to_string(),
                    source,
                })?;
            }
        }

        Ok(())
    }

    /// Update/clone all git repositories in parallel with rayon.
    ///
    /// A progress bar is added for progress indication.
    fn parallel_update_or_clone(&self, repos: &[String], download_dir: &Path) -> Result<(), Error> {
        let progress_bar = get_progress_bar(repos.len() as u64);

        // Prepare a ssh session for better performance.
        warmup_ssh_session()?;

        // Clone/update all repositories in parallel
        let results: Vec<Result<(), Error>> = repos
            .par_iter()
            .map(|repo| {
                let target_dir = download_dir.join(repo);

                // If the repo already exists, only pull it.
                // Otherwise do a clone.
                let result = if target_dir.exists() {
                    update_repo(repo, &target_dir)
                } else {
                    clone_repo(repo.to_string(), &target_dir)
                };

                // Increment the counter
                progress_bar.inc(1);
                result
            })
            .collect();

        // Finish the spinner
        progress_bar.finish_with_message("All repositories cloned or updated.");

        // Display any errors during cloning/updating to the user.
        let mut error_iter = results.into_iter().filter_map(Result::err).peekable();
        if error_iter.peek().is_some() {
            error!("The command failed for the following repositories:");
            for error in error_iter {
                error!("{error}");
            }
        }

        Ok(())
    }
}

/// Create a new ssh connection that doesn't get bound to a given session.
/// This allows that session to be reused, effectively eliminating the need to authenticate every
/// time a git repository is cloned/pulled.
///
/// This is especially necessary for users that have their SSH key on a physical device, such as a
/// NitroKey, as authentications with such devices are sequential and take quite some time.
pub fn warmup_ssh_session() -> Result<(), Error> {
    let mut ssh_command = Command::new("ssh");
    ssh_command.args(vec!["-T", SSH_HOST]);
    trace!("Running command: {ssh_command:?}");
    let output = &ssh_command.output().map_err(|source| Error::Io {
        context: "running the SSH warmup command".to_string(),
        source,
    })?;

    ensure_success(output, "Failed to run ssh warmup command".to_string())
}

/// Update a local git repository to the newest state.
/// Resets any local changes in case in each repository beforehand to prevent any conflicts.
fn update_repo(repo: &str, target_dir: &Path) -> Result<(), Error> {
    // Reset any possible local changes.
    let output = Command::new("git")
        .current_dir(target_dir)
        .args(vec!["reset", "--hard"])
        .output()
        .map_err(|source| Error::Io {
            context: format!("resetting the package source repository \"{repo}\""),
            source,
        })?;

    ensure_success(
        &output,
        format!("Resetting the package source repository \"{repo}\""),
    )?;

    let output = &Command::new("git")
        .current_dir(target_dir)
        .args(["pull", "--force"])
        .output()
        .map_err(|source| Error::Io {
            context: format!("pulling the package source repository \"{repo}\""),
            source,
        })?;

    ensure_success(
        output,
        format!("Pulling the package source repository \"{repo}\""),
    )
}

/// Clone a git repository into a target directory.
fn clone_repo(mut repo: String, target_dir: &Path) -> Result<(), Error> {
    // Check if this is one of the few packages that needs to be replaced.
    for (to_replace, replace_with) in PACKAGE_REPO_RENAMES {
        if repo == to_replace {
            repo = replace_with.to_string();
        }
    }

    // Arch linux replaces the literal `+` chars with spelled out `plus` equivalents in their
    // repository urls. This is to prevent any issues with external tooling and such.
    repo = repo.replace("+", "plus");

    let ssh_url = format!("{SSH_HOST}:{REPO_BASE_URL}/{repo}.git");

    let output = &Command::new("git")
        .arg("clone")
        .arg(&ssh_url)
        .arg(target_dir)
        .output()
        .map_err(|source| Error::Io {
            context: format!("cloning the package source repository \"{repo}\""),
            source,
        })?;

    ensure_success(
        output,
        format!("Cloning the package source repository \"{repo}\""),
    )
}
