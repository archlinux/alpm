//! Facilities for creating a package file from input.

use std::{
    fs::read_dir,
    path::{Path, PathBuf},
};

use alpm_buildinfo::BuildInfo;
use alpm_common::{INSTALL_SCRIPTLET_FILENAME, MetadataFile, MetadataFileName};
use alpm_mtree::Mtree;
use alpm_pkginfo::PackageInfo;

use crate::scriptlet::check_scriptlet;

/// An error that can occur when dealing with package input directories and package files.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Passed in BuildInfo data doesn't match the file it is supposedly coming from.
    #[error("The provided BuildInfo data is not that of the file {path}")]
    BuildInfoMismatch {
        /// The path to a .BUILDINFO file that mismatches with the input.
        path: PathBuf,
    },

    /// A path is not the child of another path.
    #[error("The path {path} is not a child of the parent directory {parent_path}")]
    ChildPath {
        /// The parent path.
        parent_path: PathBuf,
        /// The path that is not a child of `parent_path`.
        path: PathBuf,
    },

    /// A metadata file is missing in a package input path.
    #[error("There is no {metadata_file} file in package input path {path}")]
    MetadataFileMissing {
        /// The type of the metadata file.
        metadata_file: MetadataFileName,
        /// The path to the package input directory.
        path: PathBuf,
    },

    /// Passed in mtree data doesn't match the file it is supposedly coming from.
    #[error("The provided ALPM-MTREE data is not that of the file {path}")]
    MtreeMismatch {
        /// The path to an .MTREE file that mismatches with the input.
        path: PathBuf,
    },

    /// A path is not present in Mtree data.
    #[error("The path {path} is not present in the Mtree data")]
    MtreePathMissing {
        /// The path that is missing in the [`Mtree`] data.
        path: PathBuf,
    },

    /// A path is not present in Mtree data.
    #[error("The path {path} should not be present in the Mtree data, because {context}")]
    MtreeInvalidPath {
        /// The path that is missing in the [`Mtree`] data.
        path: PathBuf,
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "The path {path} should not be present in the
        /// Mtree data, because {context}".
        context: &'static str,
    },

    /// A path does not match what it is supposed to be.
    #[error("The path {path} {context} should be {required_path}")]
    PathMismatch {
        /// The path that is not correct.
        path: PathBuf,
        /// The required (correct) path.
        required_path: PathBuf,
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "The path {path} {context} should be
        /// {required_path}".
        context: &'static str,
    },
}

fn recurse_files(path: impl AsRef<Path>) -> Result<Vec<PathBuf>, crate::Error> {
    let mut paths = Vec::new();
    let entries = read_dir(path.as_ref()).map_err(|source| crate::Error::IoPath {
        path: path.as_ref().to_path_buf(),
        context: "reading children of directory",
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| crate::Error::IoPath {
            path: path.as_ref().to_path_buf(),
            context: "reading entry in directory",
            source,
        })?;
        let meta = entry.metadata().map_err(|source| crate::Error::IoPath {
            path: entry.path(),
            context: "getting metadata of file",
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
    package_info: PackageInfo,
    mtree: Mtree,
    base_dir: PathBuf,
    scriptlet: Option<PathBuf>,
    data_files: Vec<PathBuf>,
}

impl PackageInput {
    pub fn new(
        build_info: BuildInfo,
        package_info: PackageInfo,
        mtree: Mtree,
        base_dir: PathBuf,
        scriptlet: Option<PathBuf>,
        data_files: Vec<PathBuf>,
    ) -> Result<Self, crate::Error> {
        // Check base_dir exists and is a directory.
        if !base_dir.exists() {
            return Err(crate::Error::PathDoesNotExist { path: base_dir });
        }
        if !base_dir.is_dir() {
            return Err(crate::Error::PathNotADir { path: base_dir });
        }

        // Check that all data_files exist and are in base_dir.
        for path in data_files.iter() {
            if !path.exists() {
                return Err(crate::Error::PathDoesNotExist {
                    path: path.to_path_buf(),
                });
            }
            if !path.starts_with(base_dir.as_path()) {
                return Err(Error::ChildPath {
                    parent_path: base_dir,
                    path: path.to_path_buf(),
                }
                .into());
            }

            // Make sure metadata and script files are not in the data files.
            if path.ends_with(INSTALL_SCRIPTLET_FILENAME) {
                return Err(Error::MtreeInvalidPath {
                    path: path.to_path_buf(),
                    context: "alpm-install-scriptlets are not part of package data files",
                }
                .into());
            }
            if path.ends_with(MetadataFileName::Mtree.to_string()) {
                return Err(Error::MtreeInvalidPath {
                    path: path.to_path_buf(),
                    context: "ALPM-MTREE files are not part of package data files",
                }
                .into());
            }
            if path.ends_with(MetadataFileName::BuildInfo.to_string()) {
                return Err(Error::MtreeInvalidPath {
                    path: path.to_path_buf(),
                    context: "BUILDINFO files are not part of package data files",
                }
                .into());
            }
            if path.ends_with(MetadataFileName::PackageInfo.to_string()) {
                return Err(Error::MtreeInvalidPath {
                    path: path.to_path_buf(),
                    context: "PKGINFO files are not part of package data files",
                }
                .into());
            }
        }

        // Check that the alpm-install-scriptlet is in base_dir, exists and is somewhat valid.
        if let Some(path) = scriptlet.as_deref() {
            let required_path = base_dir.join(INSTALL_SCRIPTLET_FILENAME);
            if required_path != path {
                return Err(Error::PathMismatch {
                    path: path.to_path_buf(),
                    required_path,
                    context: "is an alpm-install-scriptlet and",
                }
                .into());
            }
            if !path.exists() {
                return Err(crate::Error::PathDoesNotExist {
                    path: path.to_path_buf(),
                });
            }
            check_scriptlet(path)?;
        }

        // Check that the .MTREE file is in base_dir, exists and that the Mtree data can be
        // re-created from file.
        {
            let path = base_dir.join(MetadataFileName::Mtree.to_string());

            if !path.exists() {
                return Err(crate::Error::PathDoesNotExist {
                    path: path.to_path_buf(),
                });
            }

            let data_from_file = Mtree::from_file(path.as_path()).map_err(crate::Error::Mtree)?;
            if data_from_file != mtree {
                return Err(Error::MtreeMismatch {
                    path: path.to_path_buf(),
                }
                .into());
            }
        }

        // Check that the .BUILDINFO file is in base_dir, exists, is contained in the mtree data and
        // that the BuildInfo data can be re-created from file.
        {
            let path = base_dir.join(MetadataFileName::BuildInfo.to_string());

            if !path.exists() {
                return Err(crate::Error::PathDoesNotExist {
                    path: path.to_path_buf(),
                });
            }

            let mtree_paths = match &mtree {
                Mtree::V1(mtree) | Mtree::V2(mtree) => mtree.as_slice(),
            };
            if !mtree_paths.iter().any(|mtree_path| match mtree_path {
                alpm_mtree::mtree::v2::Path::File(file) => file.path == path,
                _ => false,
            }) {
                return Err(Error::MtreePathMissing {
                    path: path.to_path_buf(),
                }
                .into());
            }

            let data_from_file =
                BuildInfo::from_file(path.as_path()).map_err(crate::Error::BuildInfo)?;
            if data_from_file != build_info {
                return Err(Error::BuildInfoMismatch {
                    path: path.to_path_buf(),
                }
                .into());
            }
        }

        // Check that the .PKGINFO file exists in base_dir, is contained in the mtree data and
        // that the PackageInfo data can be re-created from file.
        {
            let path = base_dir.join(MetadataFileName::PackageInfo.to_string());

            if !path.exists() {
                return Err(crate::Error::PathDoesNotExist {
                    path: path.to_path_buf(),
                });
            }

            let mtree_paths = match &mtree {
                Mtree::V1(mtree) | Mtree::V2(mtree) => mtree.as_slice(),
            };
            if !mtree_paths.iter().any(|mtree_path| match mtree_path {
                alpm_mtree::mtree::v2::Path::File(file) => file.path == path,
                _ => false,
            }) {
                return Err(Error::MtreePathMissing {
                    path: path.to_path_buf(),
                }
                .into());
            }

            let data_from_file =
                PackageInfo::from_file(path.as_path()).map_err(crate::Error::PackageInfo)?;
            if data_from_file != package_info {
                return Err(Error::BuildInfoMismatch {
                    path: path.to_path_buf(),
                }
                .into());
            }
        }

        Ok(Self {
            build_info,
            package_info,
            mtree,
            base_dir,
            scriptlet,
            data_files,
        })
    }

    pub fn get_base_dir(&self) -> &Path {
        &self.base_dir
    }

    pub fn get_build_info(&self) -> &BuildInfo {
        &self.build_info
    }

    pub fn get_package_info(&self) -> &PackageInfo {
        &self.package_info
    }

    pub fn get_mtree(&self) -> &Mtree {
        &self.mtree
    }

    pub fn get_install_scriptlet(&self) -> Option<&Path> {
        self.scriptlet.as_deref()
    }

    pub fn get_data_files(&self) -> &[PathBuf] {
        &self.data_files
    }
}

impl TryFrom<&Path> for PackageInput {
    type Error = crate::Error;

    /// Creates a [`PackageInput`] from path.
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let buildinfo_path = value.join(MetadataFileName::BuildInfo.to_string());
        if !buildinfo_path.exists() {
            return Err(Error::MetadataFileMissing {
                metadata_file: MetadataFileName::BuildInfo,
                path: value.to_path_buf(),
            }
            .into());
        }
        let build_info = BuildInfo::from_file(&buildinfo_path).map_err(crate::Error::BuildInfo)?;

        let mtree_path = value.join(MetadataFileName::Mtree.to_string());
        if !mtree_path.exists() {
            return Err(Error::MetadataFileMissing {
                metadata_file: MetadataFileName::Mtree,
                path: value.to_path_buf(),
            }
            .into());
        }
        let mtree = Mtree::from_file(&mtree_path).map_err(crate::Error::Mtree)?;

        let pkginfo_path = value.join(MetadataFileName::PackageInfo.to_string());
        if !pkginfo_path.exists() {
            return Err(Error::MetadataFileMissing {
                metadata_file: MetadataFileName::PackageInfo,
                path: value.to_path_buf(),
            }
            .into());
        }
        let package_info =
            PackageInfo::from_file(&pkginfo_path).map_err(crate::Error::PackageInfo)?;

        let scriptlet_path = value.join(INSTALL_SCRIPTLET_FILENAME);
        let scriptlet = if scriptlet_path.exists() {
            Some(scriptlet_path)
        } else {
            None
        };

        let data_files = recurse_files(value)?;

        Self::new(
            build_info,
            package_info,
            mtree,
            value.to_path_buf(),
            scriptlet,
            data_files,
        )
    }
}
