use std::{
    fs::{create_dir_all, remove_dir_all},
    path::PathBuf,
    process::Command,
};

use anyhow::{bail, Context, Result};
use log::{debug, info};
use strum::IntoEnumIterator;

use super::PackageRepositories;
use crate::cmd::ensure_success;

/// The entry point for downloading any data from package mirrors.
pub struct MirrorDownloader {
    /// The destination folder into which files should be downloaded.
    pub dest: PathBuf,
    /// The mirror url from which files will be downloaded.
    pub mirror: String,
}

impl MirrorDownloader {
    /// Download all official repository file databases and unpack them.
    /// They contain the following files:
    ///
    /// - `desc`
    /// - `files`
    pub fn sync_remote_databases(&self) -> Result<()> {
        let download_dir = self.dest.join("download/databases/");
        let target_dir = self.dest.join("databases");

        if !download_dir.exists() {
            create_dir_all(&download_dir).context("Failed to create download directory")?;
        }

        if !target_dir.exists() {
            create_dir_all(&target_dir)
                .context("Failed to create pacman cache target directory")?;
        }

        for repo in PackageRepositories::iter() {
            let name = repo.to_string();
            info!("Downloading database for repository {name}");

            let filename = format!("{name}.files");
            let file_source = format!("rsync://{}/{name}/os/x86_64/{filename}", self.mirror);

            let download_dest = download_dir.join(filename);

            // Download the db from the mirror
            let status = Command::new("rsync")
                .args([
                    "--recursive",
                    "--perms",
                    "--times",
                    // Copy files instead of symlinks
                    // Symlinks may point to files up the tree of where we're looking at,
                    // which is why normal symlinks would be invalid.
                    "--copy-links",
                    // Show total progress
                    "--info=progress2",
                ])
                .arg(file_source)
                .arg(&download_dest)
                .spawn()
                .context(format!("Failed to run rsync for pacman db {name}"))?
                .wait()
                .context(format!("Failed to start rsync for pacman db {name}"))?;

            if !status.success() {
                bail!("rsync failed for pacman db {name}");
            }

            // Remove any old files.
            let repo_target_dir = target_dir.join(&name);
            if repo_target_dir.exists() {
                remove_dir_all(&repo_target_dir).context(format!(
                    "Failed to remove old repository: {repo_target_dir:?}"
                ))?;
            }
            create_dir_all(&repo_target_dir)?;

            debug!("Extracting db to {repo_target_dir:?}");

            // Extract the db into the target folder.
            let output = Command::new("tar")
                .arg("-x")
                .arg("-f")
                .arg(&download_dest)
                .arg("-C")
                .arg(&repo_target_dir)
                .output()
                .context(format!("Failed to start tar to extract pacman dbs {name}"))?;
            ensure_success(&output)?;
        }

        Ok(())
    }
}
