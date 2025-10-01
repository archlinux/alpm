//! Syncing of source and binary repositories.

use std::{
    collections::HashSet,
    fs::{DirEntry, read_dir},
    path::Path,
};

use anyhow::Result;
use clap::ValueEnum;
use strum::{Display, EnumIter};

pub mod aur;
pub mod mirror;
pub mod pkgsrc;

/// All Arch Linux package repositories we may want to test.
#[derive(Clone, Debug, Display, EnumIter, PartialEq, ValueEnum)]
pub enum PackageRepositories {
    /// The [core] repository.
    ///
    /// [core]: https://wiki.archlinux.org/title/Official_repositories#core
    #[strum(to_string = "core")]
    Core,

    /// The [extra] repository.
    ///
    /// [extra]: https://wiki.archlinux.org/title/Official_repositories#extra
    #[strum(to_string = "extra")]
    Extra,

    /// The [multilib] repository.
    ///
    /// [multilib]: https://wiki.archlinux.org/title/Official_repositories#multilib
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
