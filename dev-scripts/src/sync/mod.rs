use std::{
    collections::HashSet,
    fs::{DirEntry, read_dir},
    path::Path,
};

use anyhow::Result;
use clap::ValueEnum;
use strum::{Display, EnumIter};

/// The [mirror] module contains all logic to download data from an Arch Linux package mirror.
/// This includes the database files or packages.
pub mod mirror;
/// The [pkgsrc] module handles the download of package source repositories.
/// This requires interaction with `git` and the official Arch Linux Gitlab, where all of the
/// package source repositories for the official packages are located.
pub mod pkgsrc;

/// All Arch Linux package repositories we may want to test.
#[derive(Clone, Debug, Display, EnumIter, PartialEq, ValueEnum)]
pub enum PackageRepositories {
    #[strum(to_string = "core")]
    Core,
    #[strum(to_string = "extra")]
    Extra,
    #[strum(to_string = "multilib")]
    Multilib,
}

/// A small helper function that returns a list of unique file names in a directory.
pub fn filenames_in_dir(path: &Path) -> Result<HashSet<String>> {
    let entries = read_dir(path)?;
    let entries: Vec<DirEntry> = entries.collect::<Result<Vec<DirEntry>, std::io::Error>>()?;
    let files: HashSet<String> = entries
        .iter()
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();

    Ok(files)
}
