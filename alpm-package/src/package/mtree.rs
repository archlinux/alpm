use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

use alpm_mtree::parse_mtree_v2;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Alpm-mtree error.
    #[error("An alpm-mtree error:\n{0}")]
    AlpmMtree(#[from] alpm_mtree::Error),

    /// IO error
    #[error("I/O error at path {0:?} while {1}:\n{2}")]
    IoPathError(PathBuf, &'static str, std::io::Error),
}

pub struct Mtree {
    paths: Vec<alpm_mtree::mtree_v2::Path>,
}

impl Mtree {
    pub fn new(input: &Path) -> Result<Self, Error> {
        let contents = read_to_string(input)
            .map_err(|e| Error::IoPathError(input.to_path_buf(), "reading file contents", e))?;
        let paths = parse_mtree_v2(contents)?;

        Ok(Self { paths })
    }
}
