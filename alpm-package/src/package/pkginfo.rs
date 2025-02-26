use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_pkginfo::{PackageInfoV1, PackageInfoV2};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Alpm-buildinfo error.
    #[error("An alpm-buildinfo error:\n{0}")]
    AlpmPkginfo(#[from] alpm_pkginfo::Error),

    /// IO error
    #[error("I/O error at path {0:?} while {1}:\n{2}")]
    IoPathError(PathBuf, &'static str, std::io::Error),
}

pub enum PkgInfo {
    V1(PackageInfoV1),
    V2(PackageInfoV2),
}

impl PkgInfo {
    pub fn new(input: &Path) -> Result<Self, Error> {
        let contents = read_to_string(input)
            .map_err(|e| Error::IoPathError(input.to_path_buf(), "reading file contents", e))?;
        match PackageInfoV2::from_str(&contents) {
            Ok(pkg_info) => Ok(PkgInfo::V2(pkg_info)),
            Err(_) => Ok(PkgInfo::V1(PackageInfoV1::from_str(&contents)?)),
        }
    }
}
