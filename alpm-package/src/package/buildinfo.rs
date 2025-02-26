use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_buildinfo::{BuildInfoV1, BuildInfoV2, Schema};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Alpm-buildinfo error.
    #[error("An alpm-buildinfo error:\n{0}")]
    AlpmBuildinfo(#[from] alpm_buildinfo::Error),

    /// IO error
    #[error("I/O error at path {0:?} while {1}:\n{2}")]
    IoPathError(PathBuf, &'static str, std::io::Error),
}

pub enum BuildInfo {
    V1(BuildInfoV1),
    V2(BuildInfoV2),
}

impl BuildInfo {
    pub fn new(input: &Path) -> Result<Self, Error> {
        let contents = read_to_string(input)
            .map_err(|e| Error::IoPathError(input.to_path_buf(), "reading file contents", e))?;
        match Schema::from_contents(&contents)? {
            Schema::V1(_) => Ok(BuildInfo::V1(BuildInfoV1::from_str(&contents)?)),
            Schema::V2(_) => Ok(BuildInfo::V2(BuildInfoV2::from_str(&contents)?)),
        }
    }
}
