use std::{
    collections::{HashMap, HashSet},
    fs::remove_dir_all,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use log::{error, info, trace};
use rayon::prelude::*;
use strum::Display;

use super::filenames_in_dir;
use crate::{cmd::ensure_success, ui::get_progress_bar};

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
pub struct PkgSrcDownloader {
    /// The destination folder into which files should be downloaded.
    pub dest: PathBuf,
}

impl PkgSrcDownloader {
    /// Download all official package source git repositories.
    pub fn download_package_source_repositories(&self) -> Result<()> {
        // Query the arch web API to get a list all official active repositories
        // The returned json is a map where the keys are the package names
        // and the value is a list of maintainer names.
        let repos = reqwest::blocking::get(PKGBASE_MAINTAINER_URL)
            .context("Failed to query pkgbase url.")?
            .json::<HashMap<String, Vec<String>>>()
            .context("Failed to deserialize archweb pkglist.")?;

        let all_repo_names: Vec<String> = repos.keys().map(String::from).collect();
        info!("Found {} official packages.", all_repo_names.len());

        let download_dir = self.dest.join("download/pkgsrc");
        self.parallel_update_or_clone(&all_repo_names, &download_dir)?;

        self.remove_old_repos(&all_repo_names, &download_dir)?;

        // Copy all .SRCINFO files to the target directory
        for repo in all_repo_names {
            let download_path = download_dir.join(&repo);
            if download_path.join(".SRCINFO").exists() {
                let target_dir = self.dest.join("pkgsrc").join(&repo);
                std::fs::create_dir_all(&target_dir)?;
                std::fs::copy(download_path.join(".SRCINFO"), target_dir.join(".SRCINFO"))?;
            }
        }

        Ok(())
    }

    /// Remove all local repositories for packages that no longer exist in the official
    /// repositories.
    ///
    /// Get the list of all locally available pkgsrc repositories.
    /// If we find any that are not in the list of official packages, we remove them.
    fn remove_old_repos(&self, repos: &[String], download_dir: &Path) -> Result<()> {
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
                remove_dir_all(download_dir.join(removed))
                    .context("Failed to remove local repository {removed}")?;
            }
        }

        Ok(())
    }

    /// Update/clone all git repositories in parallel with rayon.
    ///
    /// A progress bar is added for progress indication.
    fn parallel_update_or_clone(&self, repos: &[String], download_dir: &Path) -> Result<()> {
        let progress_bar = get_progress_bar(repos.len() as u64);

        // Prepare a ssh session for better performance.
        warmup_ssh_session()?;

        // Clone/update all repositories in parallel
        let results: Vec<Result<(), RepoUpdateError>> = repos
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
                error!(
                    "{} failed for repo {} with error:\n{:?}",
                    error.operation, error.repo, error.inner
                );
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
pub fn warmup_ssh_session() -> Result<()> {
    let mut ssh_command = Command::new("ssh");
    ssh_command.args(vec!["-T", SSH_HOST]);
    trace!("running command: {ssh_command:?}");
    let output = &ssh_command
        .output()
        .context("Failed to start ssh warmup command")?;

    ensure_success(output).context("Failed to run ssh warmup command:")
}

#[derive(Display)]
enum RepoUpdateOperation {
    Clone,
    Update,
}

struct RepoUpdateError {
    repo: String,
    operation: RepoUpdateOperation,
    inner: anyhow::Error,
}

/// Update a local git repository to the newest state.
/// Resets any local changes in case in each repository beforehand to prevent any conflicts.
fn update_repo(repo: &str, target_dir: &Path) -> Result<(), RepoUpdateError> {
    // Reset any possible local changes.
    let output = Command::new("git")
        .current_dir(target_dir)
        .args(vec!["reset", "--hard"])
        .output()
        .map_err(|err| RepoUpdateError {
            repo: repo.to_string(),
            operation: RepoUpdateOperation::Update,
            inner: err.into(),
        })?;

    ensure_success(&output).map_err(|err| RepoUpdateError {
        repo: repo.to_string(),
        operation: RepoUpdateOperation::Update,
        inner: err,
    })?;

    let output = &Command::new("git")
        .current_dir(target_dir)
        .args(["pull", "--force"])
        .output()
        .map_err(|err| RepoUpdateError {
            repo: repo.to_string(),
            operation: RepoUpdateOperation::Update,
            inner: err.into(),
        })?;

    ensure_success(output).map_err(|err| RepoUpdateError {
        repo: repo.to_string(),
        operation: RepoUpdateOperation::Update,
        inner: err,
    })
}

/// Clone a git repository into a target directory.
fn clone_repo(mut repo: String, target_dir: &Path) -> Result<(), RepoUpdateError> {
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
        .map_err(|err| RepoUpdateError {
            repo: repo.to_string(),
            operation: RepoUpdateOperation::Clone,
            inner: err.into(),
        })?;

    ensure_success(output).map_err(|err| RepoUpdateError {
        repo: repo.to_string(),
        operation: RepoUpdateOperation::Clone,
        inner: err,
    })
}
