//! All logic for downloading data from an Arch Linux package mirror.
//!
//! This includes the database files or packages.

mod rsync_changes;

use std::{
    collections::HashSet,
    fs::{create_dir_all, read_dir, remove_dir_all},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use alpm_types::{INSTALL_SCRIPTLET_FILE_NAME, MetadataFileName, PackageFileName};
use log::{debug, info, trace};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use super::{PackageRepositories, filenames_in_dir};
use crate::{
    Error,
    cmd::ensure_success,
    consts::{DATABASES_DIR, DOWNLOAD_DIR, PACKAGES_DIR},
    sync::mirror::rsync_changes::Report,
    ui::get_progress_bar,
};

/// The entry point for downloading any data from package mirrors.
#[derive(Clone, Debug)]
pub struct MirrorDownloader {
    /// The destination folder into which files should be downloaded.
    pub dest: PathBuf,
    /// The mirror url from which files will be downloaded.
    pub mirror: String,
    /// The repositories that should be downloaded.
    pub repositories: Vec<PackageRepositories>,
    /// Whether to extract all packages (regardless of changes).
    pub extract_all: bool,
}

impl MirrorDownloader {
    /// Download all official repository file databases and unpack them.
    /// They contain the following files:
    ///
    /// - `desc`
    /// - `files`
    pub fn sync_remote_databases(&self) -> Result<(), Error> {
        let download_dir = self.dest.join(DOWNLOAD_DIR).join(DATABASES_DIR);
        let target_dir = self.dest.join(DATABASES_DIR);

        create_dir_all(&download_dir).map_err(|source| Error::IoPath {
            path: download_dir.clone(),
            context: "recursively creating the directory".to_string(),
            source,
        })?;

        create_dir_all(&target_dir).map_err(|source| Error::IoPath {
            path: target_dir.clone(),
            context: "recursively creating the directory".to_string(),
            source,
        })?;

        for repo in self.repositories.iter() {
            let name = repo.to_string();
            info!("Downloading database for repository {name}");

            let filename = format!("{name}.files");
            let file_source = format!("rsync://{}/{name}/os/x86_64/{filename}", self.mirror);

            let download_dest = download_dir.join(filename);

            // Download the db from the mirror
            let mut db_sync_command = Command::new("rsync");
            db_sync_command
                .args([
                    "--recursive",
                    "--perms",
                    "--times",
                    // Report changes status
                    "--itemize-changes",
                    // Copy files instead of symlinks
                    // Symlinks may point to files up the tree of where we're looking at,
                    // which is why normal symlinks would be invalid.
                    "--copy-links",
                ])
                .arg(file_source)
                .arg(&download_dest);

            trace!("Running command: {db_sync_command:?}");
            let output = db_sync_command.output().map_err(|source| Error::Io {
                context: format!("synchronizing repository database for {name}"),
                source,
            })?;

            ensure_success(
                &output,
                format!("synchronizing repository database for {name}"),
            )?;

            trace!(
                "Rsync reports: {}",
                String::from_utf8_lossy(&output.stdout).trim()
            );

            let repo_target_dir = target_dir.join(&name);
            if repo_target_dir.exists() {
                if !self.extract_all
                    && Report::parser(&output.stdout)
                        .map_err(|source| Error::Parser(source.to_string()))?
                        .file_content_updated()?
                        .is_none()
                {
                    debug!("Database {name} is unchanged upstream, skipping extraction");
                    continue;
                } else {
                    // There are old versions of the files, remove them.
                    remove_dir_all(&repo_target_dir).map_err(|source| Error::IoPath {
                        path: repo_target_dir.clone(),
                        context: "recursively removing the directory".to_string(),
                        source,
                    })?;
                }
            }
            create_dir_all(&repo_target_dir).map_err(|source| Error::IoPath {
                path: repo_target_dir.clone(),
                context: "recursively creating the directory".to_string(),
                source,
            })?;

            debug!("Extracting db to {repo_target_dir:?}");

            // Extract the db into the target folder.
            let mut tar_command = Command::new("tar");
            tar_command
                .arg("-x")
                .arg("-f")
                .arg(&download_dest)
                .arg("-C")
                .arg(&repo_target_dir);

            trace!("Running command: {tar_command:?}");
            let output = tar_command.output().map_err(|source| Error::Io {
                context: format!("extracting the repository database for {name}"),
                source,
            })?;
            ensure_success(
                &output,
                format!("Extracting the repository database for {name}"),
            )?;
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
    pub fn sync_remote_packages(&self) -> Result<(), Error> {
        let download_dir = self.dest.join(DOWNLOAD_DIR).join(PACKAGES_DIR);
        let target_dir = self.dest.join(PACKAGES_DIR);

        create_dir_all(&download_dir).map_err(|source| Error::IoPath {
            path: download_dir.clone(),
            context: "recursively creating the directory".to_string(),
            source,
        })?;

        create_dir_all(&target_dir).map_err(|source| Error::IoPath {
            path: target_dir.clone(),
            context: "recursively creating the directory".to_string(),
            source,
        })?;

        for repo in self.repositories.iter() {
            let repo_name = repo.to_string();
            info!("Downloading packages for repository {repo_name}");

            let file_source = format!("rsync://{}/{repo_name}/os/x86_64/", self.mirror);
            let download_dest = download_dir.join(&repo_name);
            let changed = self.download_packages(&repo_name, file_source, &download_dest)?;

            let packages: Vec<PathBuf> = if self.extract_all {
                let files: Vec<_> = read_dir(&download_dest)
                    .map_err(|source| Error::IoPath {
                        path: download_dest.to_path_buf(),
                        context: "reading entries in directory".to_string(),
                        source,
                    })?
                    .map(|result| {
                        result.map_err(|source| Error::IoPath {
                            path: download_dest.to_path_buf(),
                            context: "reading a directory entry".to_string(),
                            source,
                        })
                    })
                    .collect::<Result<_, Error>>()?;
                files
                    .into_iter()
                    .map(|entry| entry.path().to_owned())
                    .collect::<Vec<_>>()
            } else {
                changed
                    .into_iter()
                    .map(|pkg| download_dest.join(pkg))
                    .collect()
            }
            .into_iter()
            // Filter out any dotfiles.
            // Those might be temporary download artifacts from previous rsync runs.
            .filter(|entry| {
                if let Some(path) = entry.to_str() {
                    !path.starts_with('.')
                } else {
                    false
                }
            })
            .collect();

            info!("Extracting packages for repository {repo_name}");
            let progress_bar = get_progress_bar(packages.len() as u64);
            packages
                .into_par_iter()
                .filter(|file| {
                    file.extension()
                        .is_none_or(|ext| ext.to_str().is_none_or(|ext| ext != "sig"))
                })
                .map(|pkg| {
                    // Extract all files that we're interested in.
                    let result = extract_pkg_files(&pkg, &target_dir, &repo_name);
                    progress_bar.inc(1);
                    result
                })
                .collect::<Result<Vec<()>, Error>>()?;
            // Finish the progress_bar
            progress_bar.finish_with_message("Finished extracting files for repository {repo}.");
        }

        // Clean up package data of packages that're no longer on the mirror.
        for repo in self.repositories.iter() {
            let mirror_packages = filenames_in_dir(&download_dir.join(repo.to_string()))?
                .into_iter()
                .filter(|file| !file.ends_with(".sig"))
                .map(remove_tarball_suffix)
                .collect::<Result<HashSet<String>, Error>>()?;

            let local_packages = filenames_in_dir(&target_dir.join(repo.to_string()))?;

            // Get the packages that no longer exist on the mirror.
            let removed_pkgs: Vec<&String> = local_packages.difference(&mirror_packages).collect();

            // Delete the package data
            if !removed_pkgs.is_empty() {
                info!("Found {} packages for cleanup:", removed_pkgs.len());
                for removed in removed_pkgs {
                    debug!("Removing local package: {removed}");
                    remove_dir_all(target_dir.join(repo.to_string()).join(removed)).map_err(
                        |source| Error::IoPath {
                            path: target_dir.join(repo.to_string()).join(removed),
                            context: "recursively removing the directory".to_string(),
                            source,
                        },
                    )?;
                }
            }
        }

        Ok(())
    }

    /// Downloads all packages and signatures of a package repository to a local directory.
    fn download_packages(
        &self,
        repo_name: &str,
        file_source: String,
        download_dest: &PathBuf,
    ) -> Result<Vec<PathBuf>, Error> {
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
            // Print structured change information to be parsed
            "--itemize-changes",
        ]);

        // Don't download any files related to repository sync databases.
        for variation in [
            ".db",
            ".db.sig",
            ".db.tar.*",
            ".db.tar.*.sig",
            ".db.tar.*.old",
            ".db.tar.*.old.sig",
            ".links.tar.*",
            ".links.tar.*.sig",
            ".files",
            ".files.sig",
            ".files.tar.*",
            ".files.tar.*.sig",
            ".files.tar.*.old",
            ".files.tar.*.old.sig",
        ] {
            cmd.arg(format!("--exclude={repo_name}{variation}"));
        }

        trace!("Running command: {cmd:?}");
        let output = cmd
            .arg(file_source)
            .arg(download_dest)
            .output()
            .map_err(|source| Error::Io {
                context: format!(
                    "syncing all package and signature files for repository {repo_name}"
                ),
                source,
            })?;

        ensure_success(
            &output,
            format!("Syncing all package and signature files for repository {repo_name}"),
        )?;

        let mut changed_files = Vec::new();

        for line in output.stdout.split(|&b| b == b'\n') {
            if let Some(path) = Report::parser(line)
                .map_err(|source| Error::Parser(source.to_string()))?
                .file_content_updated()?
            {
                trace!("File at {path:?} changed, marking for extraction");
                changed_files.push(path.to_owned());
            }
        }

        Ok(changed_files)
    }
}

/// Get the list of all files inside a given compressed tarball.
///
/// This function provides data which is necessary to determine which subset of files should be
/// extracted.
fn get_tar_file_list(pkg: &Path) -> Result<HashSet<String>, Error> {
    let mut tar_command = Command::new("tar");
    tar_command.arg("-tf").arg(pkg);
    trace!("Running command: {tar_command:?}");
    let peek_output = tar_command.output().map_err(|source| Error::Io {
        context: format!("list contents of tar file {pkg:?}"),
        source,
    })?;
    ensure_success(
        &peek_output,
        format!("Listing contents of tar file {pkg:?}"),
    )?;

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
///
/// # Panics
///
/// Panics if `pkg` points to a directory.
fn extract_pkg_files(pkg: &Path, target_dir: &Path, repo_name: &str) -> Result<(), Error> {
    let pkg_file_name = pkg
        .file_name()
        .expect("got directory when expecting file")
        .to_string_lossy()
        .to_string();
    let pkg_name = remove_tarball_suffix(pkg_file_name)?;

    // Peek into the pkg tar to see what kind of files we need to extract.
    let files = get_tar_file_list(pkg)?;

    // Create the target directory where all the files should be extracted to.
    let pkg_target_dir = target_dir.join(repo_name).join(pkg_name);
    create_dir_all(&pkg_target_dir).map_err(|source| Error::IoPath {
        path: pkg_target_dir.clone(),
        context: "recursively creating the directory".to_string(),
        source,
    })?;

    let mut cmd_args = vec![
        "-C".to_string(),
        pkg_target_dir.to_string_lossy().to_string(),
        "-xf".to_string(),
        pkg.to_string_lossy().to_string(),
    ];

    // Check for each of the known filetypes, whether it exists in the package.
    // If it does, add it to the tar command for extraction.
    for filetype in [
        MetadataFileName::Mtree.as_ref(),
        MetadataFileName::BuildInfo.as_ref(),
        MetadataFileName::PackageInfo.as_ref(),
        INSTALL_SCRIPTLET_FILE_NAME,
    ] {
        if files.contains(filetype) {
            cmd_args.push(filetype.to_string());
        }
    }

    // Run the extraction command
    let mut tar_command = Command::new("tar");
    tar_command.args(cmd_args);

    trace!("Running command: {tar_command:?}");
    let output = tar_command.output().map_err(|source| Error::IoPath {
        path: pkg.to_path_buf(),
        context: "extracting files".to_string(),
        source,
    })?;
    ensure_success(&output, format!("Extracting files from tar file {pkg:?}"))?;

    Ok(())
}

/// A small helper function that removes the `.pkg.tar.*` suffix of a tarball.
/// This is necessary to get the actual package name from a packages full file name.
pub fn remove_tarball_suffix(pkg_name: String) -> Result<String, Error> {
    let package_file_name = PackageFileName::from_str(&pkg_name)?;

    Ok(format!(
        "{}-{}-{}",
        package_file_name.name(),
        package_file_name.version(),
        package_file_name.architecture()
    ))
}
