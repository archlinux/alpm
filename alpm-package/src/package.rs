//! Facilities for creating a package file from input.

use std::{
    fs::read_dir,
    path::{Path, PathBuf},
};

mod buildinfo;
mod mtree;
mod pkginfo;
use buildinfo::BuildInfo;
use mtree::Mtree;
use pkginfo::PkgInfo;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A BUILDINFO error.
    #[error("A BUILDINFO error:\n{0}")]
    BuildInfo(#[from] buildinfo::Error),

    /// A PKGINFO error.
    #[error("A PKGINFO error:\n{0}")]
    PkgInfo(#[from] pkginfo::Error),

    /// An ALPM-MTREE error.
    #[error("An ALPM-MTREE error:\n{0}")]
    Mtree(#[from] mtree::Error),

    /// IO error
    #[error("I/O error at path {path} while {message}:\n{source}")]
    IoPathError {
        path: PathBuf,
        message: &'static str,
        source: std::io::Error,
    },

    /// The BUILDINFO file is invalid.
    #[error("The .BUILDINFO file {buildinfo_file} is invalid:\n{source}")]
    InvalidBuildInfo {
        buildinfo_file: PathBuf,
        source: alpm_buildinfo::Error,
    },

    /// The .BUILDINFO file is missing.
    #[error("There is no .BUILDINFO file in package input path {input_dir}")]
    MissingBuildInfo { input_dir: PathBuf },

    /// The .MTREE file is missing.
    #[error("There is no .MTREE file in package input path {input_dir}")]
    MissingMtree { input_dir: PathBuf },

    /// The .PKGINFO file is missing.
    #[error("There is no .PKGINFO file in package input path {input_dir}")]
    MissingPkgInfo { input_dir: PathBuf },
}

fn recurse_files(path: impl AsRef<Path>) -> Result<Vec<PathBuf>, Error> {
    let mut paths = Vec::new();
    let entries = read_dir(path.as_ref()).map_err(|source| Error::IoPathError {
        path: path.as_ref().to_path_buf(),
        message: "reading children of directory",
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| Error::IoPathError {
            path: path.as_ref().to_path_buf(),
            message: "reading entry in directory",
            source,
        })?;
        let meta = entry.metadata().map_err(|source| Error::IoPathError {
            path: entry.path(),
            message: "getting metadata of file",
            source,
        })?;

        if meta.is_dir() {
            let mut subdir = recurse_files(entry.path())?;
            paths.append(&mut subdir);
        }

        if meta.is_file() {
            paths.push(entry.path());
        }
    }

    Ok(paths)
}

/// Representation of a package input directory.
pub struct PackageInput {
    build_info: BuildInfo,
    pkg_info: PkgInfo,
    mtree: Mtree,
    scriptlet: Option<PathBuf>,
    data_files: Vec<PathBuf>,
}

impl PackageInput {
    pub fn new(input_dir: &Path) -> Result<Self, Error> {
        let buildinfo_path = input_dir.join(".BUILDINFO");
        let mtree_path = input_dir.join(".MTREE");
        let pkginfo_path = input_dir.join(".PKGINFO");
        let scriptlet_path = input_dir.join(".INSTALL");

        if !buildinfo_path.exists() {
            return Err(Error::MissingBuildInfo {
                input_dir: input_dir.to_path_buf(),
            });
        }
        if !mtree_path.exists() {
            return Err(Error::MissingMtree {
                input_dir: input_dir.to_path_buf(),
            });
        }
        if !pkginfo_path.exists() {
            return Err(Error::MissingPkgInfo {
                input_dir: input_dir.to_path_buf(),
            });
        }

        let build_info = BuildInfo::new(&buildinfo_path)?;
        let mtree = Mtree::new(&mtree_path)?;
        let pkg_info = PkgInfo::new(&pkginfo_path)?;
        let scriptlet = if scriptlet_path.exists() {
            // TODO: check if text file
            Some(scriptlet_path)
        } else {
            None
        };

        let data_files = recurse_files(input_dir)?;

        Ok(Self {
            build_info,
            pkg_info,
            mtree,
            scriptlet,
            data_files,
        })
    }
}
