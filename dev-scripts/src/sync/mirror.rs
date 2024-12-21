use std::{
    collections::HashSet,
    fs::{create_dir_all, remove_dir_all, DirEntry},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, Context, Result};
use log::{debug, info};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use super::{filenames_in_dir, PackageRepositories};
use crate::{cmd::ensure_success, ui::get_progress_bar};

/// The entry point for downloading any data from package mirrors.
pub struct MirrorDownloader {
    /// The destination folder into which files should be downloaded.
    pub dest: PathBuf,
    /// The mirror url from which files will be downloaded.
    pub mirror: String,
    /// The repositories that should be downloaded.
    pub repositories: Vec<PackageRepositories>,
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

        for repo in self.repositories.iter() {
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

    /// Download all official repository packages and extract all files that're interesting to us.
    /// Specifically:
    ///
    ///  - `.BUILDINFO`
    ///  - `.MTREE`
    ///  - `.PKGINFO`
    ///  - `.INSTALL` (Optional)
    pub fn sync_remote_packages(&self) -> Result<()> {
        let download_dir = self.dest.join("download/packages");
        let target_dir = self.dest.join("packages");

        if !download_dir.exists() {
            create_dir_all(&download_dir).context("Failed to create download directory")?;
        }

        if !target_dir.exists() {
            create_dir_all(&target_dir)
                .context("Failed to create pacman cache target directory")?;
        }

        for repo in self.repositories.iter() {
            let repo_name = repo.to_string();
            info!("Downloading packages for repository {repo_name}");

            let file_source = format!("rsync://{}/{repo_name}/os/x86_64/", self.mirror);
            let download_dest = download_dir.join(&repo_name);
            self.download_packages(&repo_name, file_source, &download_dest)?;

            // Go through all packages of this repo and extract the respective relevant files.
            let packages = std::fs::read_dir(&download_dest)?
                .collect::<Result<Vec<DirEntry>, std::io::Error>>()?;

            // Filter out any dotfiles.
            // Those might be temporary download artifacts from previous rsync runs.
            let packages: Vec<DirEntry> = packages
                .into_iter()
                .filter(|entry| {
                    if let Some(path) = entry.file_name().to_str() {
                        !path.starts_with('.')
                    } else {
                        false
                    }
                })
                .collect();

            // TODO:
            // The extraction work can be cut down on successive runs by using rsync's `--itemize`
            // flag, which gives a list of changed packages.
            // That way, we only extract packages that actually changed and don't do any duplicate
            // work on old packages.
            // See https://gitlab.archlinux.org/archlinux/alpm/alpm/-/issues/68
            info!("Extracting packages for repository {repo_name}");
            let progress_bar = get_progress_bar(packages.len() as u64);
            packages
                .into_par_iter()
                .map(|pkg| {
                    // Extract all files that we're interested in.
                    let result = extract_pkg_files(&pkg, &target_dir, &repo_name);
                    progress_bar.inc(1);
                    result
                })
                .collect::<Result<Vec<()>>>()?;
            // Finish the progress_bar
            progress_bar.finish_with_message("Finished extracting files for repository {repo}.");
        }

        // Clean up package data of packages that're no longer on the mirror.
        for repo in self.repositories.iter() {
            let mirror_packages = filenames_in_dir(&download_dir.join(repo.to_string()))?
                .into_iter()
                .map(remove_tarball_suffix)
                .collect::<Result<HashSet<String>>>()?;

            let local_packages = filenames_in_dir(&target_dir.join(repo.to_string()))?;

            // Get the packages that no longer exist on the mirror.
            let removed_pkgs: Vec<&String> = local_packages.difference(&mirror_packages).collect();

            // Delete the package data
            if !removed_pkgs.is_empty() {
                info!("Found {} packages for cleanup:", removed_pkgs.len());
                for removed in removed_pkgs {
                    debug!("Removing local package: {removed}");
                    remove_dir_all(target_dir.join(repo.to_string()).join(removed)).context(
                        format!(
                            "Failed to remove local package {:?}",
                            target_dir.join(repo.to_string()).join(removed)
                        ),
                    )?;
                }
            }
        }

        Ok(())
    }

    /// Download all packages of a given arch package repository into the download directory.
    fn download_packages(
        &self,
        repo_name: &str,
        file_source: String,
        download_dest: &PathBuf,
    ) -> Result<()> {
        let mut cmd = Command::new("rsync");
        cmd.args([
            "--recursive",
            "--perms",
            "--times",
            "--delete",
            "--hard-links",
            // Copy actual files instead of symlinks.
            // Most symlinks point to files up the tree of where we're looking at,
            // which is why normal symlinks would be invalid.
            "--copy-links",
            // Check for deletions once everything has been transferred
            "--delete-after",
            // Only overwrite updated files in the very end.
            // This allows for a somewhat "atomic" update process.
            "--delay-updates",
            // Show total progress
            "--info=progress2",
            // Exclude package signatures
            "--exclude=*.sig",
        ]);

        // Don't download any files related to repository sync databases (signatures are generally
        // excluded by the rsync call).
        for variation in [
            ".db",
            ".db.tar.gz",
            ".db.tar.gz.old",
            ".links.tar.gz",
            ".files",
            ".files.tar.gz",
            ".files.tar.gz.old",
        ] {
            cmd.arg(format!("--exclude={repo_name}{variation}"));
        }

        let status = cmd
            .arg(file_source)
            .arg(download_dest)
            .spawn()
            .context(format!(
                "Failed to start package rsync for pacman db {repo_name}"
            ))?
            .wait()
            .context(format!(
                "Failed to start package rsync for pacman db {repo_name}"
            ))?;

        if !status.success() {
            bail!("Package rsync failed for pacman db {repo_name}");
        }

        Ok(())
    }
}

/// Get the list of all files inside a given compressed tarball.
///
/// This function provides data which is necessary to determine which subset of files should be
/// extracted.
fn get_tar_file_list(pkg: &DirEntry) -> Result<HashSet<String>> {
    let peek_output = Command::new("tar")
        .arg("-tf")
        .arg(pkg.path())
        .output()
        .context(format!("Failed to peek into pkg {:?}", pkg.path()))?;
    ensure_success(&peek_output).context("Error while peeking into package")?;

    Ok(String::from_utf8_lossy(&peek_output.stdout)
        .lines()
        .map(|line| line.to_string())
        .collect())
}

/// Use `tar` to extract relevant package metadata and script files from packages files.
///
/// This function attempts to extract ".MTREE", ".BUILDINFO", ".PKGINFO" and ".INSTALL" files.
/// Extracted files are placed in a directory structure that reflects the package's association with
/// a package repository.
///
/// ## Note
///
/// Since some files are optional, we have to take a look at the files in that tarball to determine
/// which of the files need to be actually extracted.
fn extract_pkg_files(pkg: &DirEntry, target_dir: &Path, repo_name: &str) -> Result<()> {
    let pkg_file_name = pkg.file_name().to_string_lossy().to_string();
    let pkg_name = remove_tarball_suffix(pkg_file_name)?;

    // Peek into the pkg tar to see what kind of files we need to extract.
    let files = get_tar_file_list(pkg)?;

    // Create the target directory where all the files should be extracted to.
    let pkg_target_dir = target_dir.join(repo_name).join(pkg_name);
    create_dir_all(&pkg_target_dir)?;

    let mut cmd_args = vec![
        "-C".to_string(),
        pkg_target_dir.to_string_lossy().to_string(),
        "-xf".to_string(),
        pkg.path().to_string_lossy().to_string(),
    ];

    // Check for each of the known filetypes, whether it exists in the package.
    // If it does, add it to the tar command for extraction.
    for filetype in [".MTREE", ".BUILDINFO", ".PKGINFO", ".INSTALL"] {
        if files.contains(filetype) {
            cmd_args.push(filetype.to_string());
        }
    }

    // Run the extraction command
    let output = Command::new("tar")
        .args(cmd_args)
        .output()
        .context(format!("Failed to extract files from pkg {:?}", pkg.path()))?;
    ensure_success(&output).context("Error while downloading packages via rsync")?;

    Ok(())
}

/// A small helper function that removes the `.pkg.tar.*` suffix of a tarball.
/// This is necessary to get the actual package name from a packages full file name.
pub fn remove_tarball_suffix(pkg_name: String) -> Result<String> {
    let pkg_name = if let Some(pkg_name) = pkg_name.strip_suffix(".pkg.tar.zst") {
        pkg_name
    } else if let Some(pkg_name) = pkg_name.strip_suffix(".pkg.tar.xz") {
        pkg_name
    } else {
        bail!("Found package with unknown tarball compression: {pkg_name:?}");
    };

    Ok(pkg_name.to_string())
}
