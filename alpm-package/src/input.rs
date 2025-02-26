//! Facilities for creating a package file from input.

use std::{
    fs::read,
    path::{Path, PathBuf},
};

use alpm_buildinfo::BuildInfo;
use alpm_common::{MetadataFile, relative_files};
use alpm_mtree::Mtree;
use alpm_pkginfo::PackageInfo;
use alpm_types::{Blake2b512Checksum, INSTALL_SCRIPTLET_FILE_NAME, MetadataFileName};
use log::debug;

#[cfg(doc)]
use crate::Package;
use crate::scriptlet::check_scriptlet;

/// An error that can occur when dealing with package input directories and package files.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The hash digest of a file has changed.
    #[error(
        "The previously recorded hash digest {digest} for file {path} has changed to {path_digest}"
    )]
    DigestMismatch {
        /// The path of the file for which the digest changed.
        path: PathBuf,
        /// The initially recorded hash digest of the file.
        digest: Blake2b512Checksum,
        /// The new digest of the file.
        path_digest: Blake2b512Checksum,
    },

    /// A metadata file is missing in a package input path.
    #[error("There is no {metadata_file} file in package input path {path}")]
    MetadataFileMissing {
        /// The type of the metadata file.
        metadata_file: MetadataFileName,
        /// The path to the package input directory.
        path: PathBuf,
    },

    /// A metadata file does not match the provided data.
    #[error("The provided {metadata_type} data does not match the contents of {metadata_path}")]
    MetadataFileMismatch {
        /// The type of the metadata file.
        metadata_type: MetadataFileName,
        /// The path to the mismatching metadata file.
        metadata_path: PathBuf,
    },
}

/// A package input directory.
#[derive(Clone, Debug)]
pub struct PackageInput {
    build_info: BuildInfo,
    build_info_digest: Blake2b512Checksum,
    package_info: PackageInfo,
    package_info_digest: Blake2b512Checksum,
    mtree: Mtree,
    mtree_digest: Blake2b512Checksum,
    base_dir: PathBuf,
    scriptlet: Option<PathBuf>,
    data_files: Vec<PathBuf>,
}

impl PackageInput {
    /// Creates a [`PackageInput`] from an input directory.
    ///
    /// The input directory must contain
    ///
    /// - a valid [ALPM-MTREE] file,
    /// - a valid [BUILDINFO] file,
    /// - a valid [PKGINFO] file,
    ///
    /// Further, the input directory may contain an [alpm-install-scriptlet] file and zero or more
    /// package data files (see [alpm-package]).
    ///
    /// This function reads [ALPM-MTREE], [BUILDINFO] and [PKGINFO] files, collects the path of an
    /// existing [alpm-install-scriptlet] to validate them.
    /// All data files below `dir` are then checked against the [ALPM-MTREE] data.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - there is no valid [BUILDINFO] file,
    /// - there is no valid [ALPM-MTREE] file,
    /// - there is no valid [PKGINFO] file,
    /// - or one of the files below `dir` does not match the [ALPM-MTREE] data.
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    /// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
    /// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    /// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
    pub fn from_input_dir(dir: impl AsRef<Path>) -> Result<Self, crate::Error> {
        let dir = dir.as_ref();
        debug!("Create PackageInput from path {dir:?}");

        // Get BuildInfo data and file digest.
        debug!("Check that a valid BUILDINFO file exists in {dir:?}.");
        let buildinfo_path = dir.join(MetadataFileName::BuildInfo.as_ref());
        if !buildinfo_path.exists() {
            return Err(Error::MetadataFileMissing {
                metadata_file: MetadataFileName::BuildInfo,
                path: dir.to_path_buf(),
            }
            .into());
        }
        // Read the file to a buffer.
        let buf = read(buildinfo_path.as_path()).map_err(|source| crate::Error::IoPath {
            path: buildinfo_path,
            context: "reading the BUILDINFO file",
            source,
        })?;
        // Validate the metadata.
        let build_info = BuildInfo::from_reader(buf.as_slice()).map_err(crate::Error::BuildInfo)?;
        // Create a hash digest for the file.
        let build_info_digest = Blake2b512Checksum::calculate_from(buf);

        // Get PackageInfo data and file digest.
        debug!("Check that a valid PKGINFO file exists in {dir:?}.");
        let pkginfo_path = dir.join(MetadataFileName::PackageInfo.as_ref());
        if !pkginfo_path.exists() {
            return Err(Error::MetadataFileMissing {
                metadata_file: MetadataFileName::PackageInfo,
                path: dir.to_path_buf(),
            }
            .into());
        }
        // Read the file to a buffer.
        let buf = read(pkginfo_path.as_path()).map_err(|source| crate::Error::IoPath {
            path: pkginfo_path,
            context: "reading the PKGINFO file",
            source,
        })?;
        // Validate the metadata.
        let package_info =
            PackageInfo::from_reader(buf.as_slice()).map_err(crate::Error::PackageInfo)?;
        // Create a hash digest for the file.
        let package_info_digest = Blake2b512Checksum::calculate_from(buf);

        // Get optional scriptlet file.
        debug!("Check that an alpm-install-scriptlet is valid if it exists in {dir:?}.");
        let scriptlet_path = dir.join(INSTALL_SCRIPTLET_FILE_NAME);
        let scriptlet = if scriptlet_path.exists() {
            // Validate the scriptlet.
            check_scriptlet(scriptlet_path.as_path())?;

            Some(scriptlet_path)
        } else {
            None
        };

        // Get Mtree data and file digest.
        debug!("Check that a valid .MTREE file exists in {dir:?}.");
        let mtree_path = dir.join(MetadataFileName::Mtree.as_ref());
        if !mtree_path.exists() {
            return Err(Error::MetadataFileMissing {
                metadata_file: MetadataFileName::Mtree,
                path: dir.to_path_buf(),
            }
            .into());
        }
        // Read the file to a buffer.
        let buf = read(mtree_path.as_path()).map_err(|source| crate::Error::IoPath {
            path: mtree_path,
            context: "reading the ALPM-MTREE file",
            source,
        })?;
        // Validate the metadata.
        let mtree = Mtree::from_reader(buf.as_slice()).map_err(crate::Error::Mtree)?;
        debug!(".MTREE data:\n{mtree}");
        // Create a hash digest for the file.
        let mtree_digest = Blake2b512Checksum::calculate_from(buf);

        // Get all data files in input_dir, excluding the ALPM-MTREE file, for comparison with
        // ALPM-MTREE data.
        let mut relative_files = relative_files(dir, &[".MTREE"])?;
        debug!("relative files:\n{relative_files:?}");
        mtree.validate_paths(dir, &relative_files)?;

        // Remove all metadata files and scriptlet files.
        relative_files.retain(|value| {
            !value.to_str().is_some_and(|name| {
                name == MetadataFileName::BuildInfo.as_ref()
                    || name == MetadataFileName::PackageInfo.as_ref()
                    || name == MetadataFileName::Mtree.as_ref()
                    || name == INSTALL_SCRIPTLET_FILE_NAME
            })
        });

        Ok(Self {
            build_info,
            build_info_digest,
            package_info,
            package_info_digest,
            mtree,
            mtree_digest,
            base_dir: dir.to_path_buf(),
            scriptlet,
            data_files: relative_files,
        })
    }

    /// Returns the base directory of the [`PackageInput`] as [`Path`] reference.
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Returns a reference to the [`BuildInfo`] data of the [`PackageInput`].
    ///
    /// Compares the stored hash digest of the file with that of the file on disk.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - the file on disk can no longer be read,
    /// - or the file on disk has a changed hash digest.
    pub fn build_info(&self) -> Result<&BuildInfo, crate::Error> {
        let path = self.base_dir.join(MetadataFileName::BuildInfo.as_ref());
        let buf = read(path.as_path()).map_err(|source| crate::Error::IoPath {
            path: path.clone(),
            context: "reading the BUILDINFO file",
            source,
        })?;
        let path_digest = Blake2b512Checksum::calculate_from(buf);
        if path_digest != self.build_info_digest {
            return Err(Error::DigestMismatch {
                path: path.clone(),
                digest: self.build_info_digest.clone(),
                path_digest,
            }
            .into());
        }

        Ok(&self.build_info)
    }

    /// Returns a reference to the [`PackageInfo`] data of the [`PackageInput`].
    ///
    /// Compares the stored hash digest of the file with that of the file on disk.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - the file on disk can no longer be read,
    /// - or the file on disk has a changed hash digest.
    pub fn package_info(&self) -> Result<&PackageInfo, crate::Error> {
        let path = self.base_dir.join(MetadataFileName::PackageInfo.as_ref());
        let buf = read(path.as_path()).map_err(|source| crate::Error::IoPath {
            path: path.clone(),
            context: "reading the PKGINFO file",
            source,
        })?;
        let path_digest = Blake2b512Checksum::calculate_from(buf);
        if path_digest != self.package_info_digest {
            return Err(Error::DigestMismatch {
                path: path.clone(),
                digest: self.package_info_digest.clone(),
                path_digest,
            }
            .into());
        }

        Ok(&self.package_info)
    }

    /// Returns a reference to the [`Mtree`] data of the [`PackageInput`].
    ///
    /// Compares the stored hash digest of the file with that of the file on disk.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - the file on disk can no longer be read,
    /// - or the file on disk has a changed hash digest.
    pub fn mtree(&self) -> Result<&Mtree, crate::Error> {
        let path = self.base_dir.join(MetadataFileName::Mtree.as_ref());
        let buf = read(path.as_path()).map_err(|source| crate::Error::IoPath {
            path: path.clone(),
            context: "reading the ALPM-MTREE file",
            source,
        })?;
        let path_digest = Blake2b512Checksum::calculate_from(buf);
        if path_digest != self.mtree_digest {
            return Err(Error::DigestMismatch {
                path: path.clone(),
                digest: self.mtree_digest.clone(),
                path_digest,
            }
            .into());
        }

        Ok(&self.mtree)
    }

    /// Returns the optional [alpm-install-scriptlet] of the [`PackageInput`] as [`Path`] reference.
    ///
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    pub fn install_scriptlet(&self) -> Option<&Path> {
        self.scriptlet.as_deref()
    }

    /// Returns a slice of [`PathBuf`]s representing all data files of the [`PackageInput`].
    pub fn get_data_files(&self) -> &[PathBuf] {
        &self.data_files
    }
}

impl TryFrom<&Path> for PackageInput {
    type Error = crate::Error;

    /// Creates a [`PackageInput`] from [`Path`] reference.
    ///
    /// Delegates to [`PackageInput::from_input_dir`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`PackageInput::from_input_dir`] fails.
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        Self::from_input_dir(value)
    }
}
