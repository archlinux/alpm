//! Facilities for creating a package file from input.

use std::{
    fmt::Display,
    fs::read,
    path::{Path, PathBuf},
};

use alpm_buildinfo::BuildInfo;
use alpm_common::{InputPaths, MetadataFile, relative_files};
use alpm_mtree::Mtree;
use alpm_pkginfo::PackageInfo;
use alpm_types::{
    Architecture,
    Blake2b512Checksum,
    INSTALL_SCRIPTLET_FILE_NAME,
    MetadataFileName,
    Name,
    Packager,
    Version,
};
use log::{debug, trace};

#[cfg(doc)]
use crate::Package;
use crate::scriptlet::check_scriptlet;

#[derive(Debug)]
pub struct MetadataMismatch {
    pub metadata_file_a: MetadataFileName,
    pub key_a: String,
    pub value_a: String,
    pub metadata_file_b: MetadataFileName,
    pub key_b: String,
    pub value_b: String,
}

impl Display for MetadataMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} => {}\n{}: {} => {}",
            self.metadata_file_a,
            self.key_a,
            self.value_a,
            self.metadata_file_b,
            self.key_b,
            self.value_b
        )
    }
}

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

    /// The metadata files have mismatching entries.
    #[error(
        "The following metadata entries are not matching:\n{}",
        mismatches.iter().fold(String::new(), |mut output, mismatch| {output.push_str(&format!("{}\n", mismatch)); output}))]
    MetadataMismatch {
        /// A list of mismatches.
        mismatches: Vec<MetadataMismatch>,
    },
}

/// An [alpm-install-scriptlet].
///
/// Tracks the path and hash digest of a valid [alpm-install-scriptlet] file.
///
/// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
#[derive(Clone, Debug)]
struct Scriptlet {
    pub path: PathBuf,
    pub digest: Blake2b512Checksum,
}

/// Returns an optional [alpm-install-scriptlet] path and its checksum.
///
/// # Errors
///
/// Returns an error if
///
/// - the file does not exist,
/// - the file contents cannot be read to a buffer,
/// - or the file contents do not represent valid [alpm-install-scriptlet] data.
///
/// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
fn get_scriptlet(dir: impl AsRef<Path>) -> Result<Option<Scriptlet>, crate::Error> {
    let dir = dir.as_ref();
    debug!("Check that an alpm-install-scriptlet is valid if it exists in {dir:?}.");
    let path = dir.join(INSTALL_SCRIPTLET_FILE_NAME);

    Ok(if path.exists() {
        // Validate the scriptlet.
        check_scriptlet(path.as_path())?;

        let buf = read(path.as_path()).map_err(|source| crate::Error::IoPath {
            path: path.clone(),
            context: "reading the alpm-install-scriptlet file",
            source,
        })?;

        Some(Scriptlet {
            path: path.clone(),
            digest: Blake2b512Checksum::calculate_from(buf),
        })
    } else {
        None
    })
}

/// Returns a [`BuildInfo`] and its file hash digest.
///
/// # Errors
///
/// Returns an error if
///
/// - the file does not exist,
/// - the file contents cannot be read to a buffer,
/// - or the file contents do not represent valid [`BuildInfo`] data.
fn get_build_info(dir: impl AsRef<Path>) -> Result<(BuildInfo, Blake2b512Checksum), crate::Error> {
    let dir = dir.as_ref();
    debug!("Check that a valid BUILDINFO file exists in {dir:?}.");
    let buildinfo_path = dir.join(MetadataFileName::BuildInfo.as_ref());

    if !buildinfo_path.exists() {
        return Err(Error::MetadataFileMissing {
            metadata_file: MetadataFileName::BuildInfo,
            path: buildinfo_path.to_path_buf(),
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
    let build_info = BuildInfo::from_reader(buf.as_slice()).map_err(crate::Error::AlpmBuildInfo)?;
    // Create a hash digest for the file.
    let build_info_digest = Blake2b512Checksum::calculate_from(buf);

    Ok((build_info, build_info_digest))
}

/// Returns a [`PackageInfo`] and its file hash digest.
///
/// # Errors
///
/// Returns an error if
///
/// - the file does not exist,
/// - the file contents cannot be read to a buffer,
/// - or the file contents do not represent valid [`PackageInfo`] data.
fn get_package_info(
    dir: impl AsRef<Path>,
) -> Result<(PackageInfo, Blake2b512Checksum), crate::Error> {
    let dir = dir.as_ref();
    debug!("Check that a valid PKGINFO file exists in {dir:?}.");
    let path = dir.join(MetadataFileName::PackageInfo.as_ref());

    if !path.exists() {
        return Err(Error::MetadataFileMissing {
            metadata_file: MetadataFileName::PackageInfo,
            path: dir.to_path_buf(),
        }
        .into());
    }
    // Read the file to a buffer.
    let buf = read(path.as_path()).map_err(|source| crate::Error::IoPath {
        path,
        context: "reading the PKGINFO file",
        source,
    })?;
    // Validate the metadata.
    let package_info =
        PackageInfo::from_reader(buf.as_slice()).map_err(crate::Error::AlpmPackageInfo)?;
    // Create a hash digest for the file.
    let package_info_digest = Blake2b512Checksum::calculate_from(buf);

    Ok((package_info, package_info_digest))
}

/// Returns an [`Mtree`] and its file hash digest.
///
/// # Errors
///
/// Returns an error if
///
/// - the file does not exist,
/// - the file contents cannot be read to a buffer,
/// - or the file contents do not represent valid [`Mtree`] data.
fn get_mtree(dir: impl AsRef<Path>) -> Result<(Mtree, Blake2b512Checksum), crate::Error> {
    let dir = dir.as_ref();
    debug!("Check that a valid .MTREE file exists in {dir:?}.");
    let path = dir.join(MetadataFileName::Mtree.as_ref());

    if !path.exists() {
        return Err(Error::MetadataFileMissing {
            metadata_file: MetadataFileName::Mtree,
            path: dir.to_path_buf(),
        }
        .into());
    }
    // Read the file to a buffer.
    let buf = read(path.as_path()).map_err(|source| crate::Error::IoPath {
        path,
        context: "reading the ALPM-MTREE file",
        source,
    })?;
    // Validate the metadata.
    let mtree = Mtree::from_reader(buf.as_slice()).map_err(crate::Error::AlpmMtree)?;
    debug!(".MTREE data:\n{mtree}");
    // Create a hash digest for the file.
    let mtree_digest = Blake2b512Checksum::calculate_from(buf);

    Ok((mtree, mtree_digest))
}

/// The comparison intersection between two different types of metadata file formats.
///
/// This is used to allow comparison between [`BuildInfo`] and [`PackageInfo`] data.
pub struct MetadataComparison<'a, 'b, 'c, 'd, 'e, 'f> {
    pub package_name: &'a Name,
    pub package_base: &'b Name,
    pub version: &'c Version,
    pub architecture: &'d Architecture,
    pub packager: &'e Packager,
    pub build_date: &'f i64,
}

/// Extracts [`MetadataComparison`] from a [`BuildInfo`].
fn metadata_comparison_build_info(build_info: &BuildInfo) -> MetadataComparison {
    match build_info {
        BuildInfo::V1(inner) => MetadataComparison {
            package_name: inner.pkgname(),
            package_base: inner.pkgbase(),
            version: inner.pkgver(),
            architecture: inner.pkgarch(),
            packager: inner.packager(),
            build_date: inner.builddate(),
        },
        BuildInfo::V2(inner) => MetadataComparison {
            package_name: inner.pkgname(),
            package_base: inner.pkgbase(),
            version: inner.pkgver(),
            architecture: inner.pkgarch(),
            packager: inner.packager(),
            build_date: inner.builddate(),
        },
    }
}

/// Extracts [`MetadataComparison`] from a [`PackageInfo`].
fn metadata_comparison_package_info(package_info: &PackageInfo) -> MetadataComparison {
    match package_info {
        PackageInfo::V1(inner) => MetadataComparison {
            package_name: inner.pkgname(),
            package_base: inner.pkgbase(),
            version: inner.pkgver(),
            architecture: inner.arch(),
            packager: inner.packager(),
            build_date: inner.builddate(),
        },
        PackageInfo::V2(inner) => MetadataComparison {
            package_name: inner.pkgname(),
            package_base: inner.pkgbase(),
            version: inner.pkgver(),
            architecture: inner.arch(),
            packager: inner.packager(),
            build_date: inner.builddate(),
        },
    }
}

/// Compares overlapping data of a [`BuildInfo`] and a [`PackageInfo`].
///
/// # Errors
///
/// Returns an error if there are one or more mismatches in the data provided by `build_info`
/// and `package_info`.
fn compare_build_info_package_info(
    build_info: &BuildInfo,
    package_info: &PackageInfo,
) -> Result<(), crate::Error> {
    let build_info_compare = metadata_comparison_build_info(build_info);
    let package_info_compare = metadata_comparison_package_info(package_info);
    let mut mismatches = Vec::new();

    if build_info_compare.package_name != package_info_compare.package_name {
        mismatches.push(MetadataMismatch {
            metadata_file_a: MetadataFileName::BuildInfo,
            key_a: "pkgname".to_string(),
            value_a: build_info_compare.package_name.to_string(),
            metadata_file_b: MetadataFileName::PackageInfo,
            key_b: "pkgname".to_string(),
            value_b: package_info_compare.package_name.to_string(),
        })
    }

    if build_info_compare.package_base != package_info_compare.package_base {
        mismatches.push(MetadataMismatch {
            metadata_file_a: MetadataFileName::BuildInfo,
            key_a: "pkgbase".to_string(),
            value_a: build_info_compare.package_base.to_string(),
            metadata_file_b: MetadataFileName::PackageInfo,
            key_b: "pkgbase".to_string(),
            value_b: package_info_compare.package_base.to_string(),
        })
    }

    if build_info_compare.version != package_info_compare.version {
        mismatches.push(MetadataMismatch {
            metadata_file_a: MetadataFileName::BuildInfo,
            key_a: "pkgver".to_string(),
            value_a: build_info_compare.version.to_string(),
            metadata_file_b: MetadataFileName::PackageInfo,
            key_b: "pkgver".to_string(),
            value_b: package_info_compare.version.to_string(),
        })
    }

    if build_info_compare.architecture != package_info_compare.architecture {
        mismatches.push(MetadataMismatch {
            metadata_file_a: MetadataFileName::BuildInfo,
            key_a: "pkgarch".to_string(),
            value_a: build_info_compare.architecture.to_string(),
            metadata_file_b: MetadataFileName::PackageInfo,
            key_b: "arch".to_string(),
            value_b: package_info_compare.architecture.to_string(),
        })
    }

    if build_info_compare.packager != package_info_compare.packager {
        mismatches.push(MetadataMismatch {
            metadata_file_a: MetadataFileName::BuildInfo,
            key_a: "packager".to_string(),
            value_a: build_info_compare.packager.to_string(),
            metadata_file_b: MetadataFileName::PackageInfo,
            key_b: "packager".to_string(),
            value_b: package_info_compare.packager.to_string(),
        })
    }

    if build_info_compare.build_date != package_info_compare.build_date {
        mismatches.push(MetadataMismatch {
            metadata_file_a: MetadataFileName::BuildInfo,
            key_a: "builddate".to_string(),
            value_a: build_info_compare.build_date.to_string(),
            metadata_file_b: MetadataFileName::PackageInfo,
            key_b: "builddate".to_string(),
            value_b: package_info_compare.build_date.to_string(),
        })
    }

    if !mismatches.is_empty() {
        return Err(Error::MetadataMismatch { mismatches }.into());
    }

    Ok(())
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
    scriptlet: Option<Scriptlet>,
    data_files: Vec<PathBuf>,
}

impl PackageInput {
    /// Creates a [`PackageInput`] from an input directory.
    ///
    /// An input directory must contain
    ///
    /// - a valid [ALPM-MTREE] file,
    /// - a valid [BUILDINFO] file,
    /// - a valid [PKGINFO] file,
    ///
    /// Further, the input directory may contain an [alpm-install-scriptlet] file and zero or more
    /// package data files (see [alpm-package]).
    ///
    /// This function reads [ALPM-MTREE], [BUILDINFO] and [PKGINFO] files, collects the path of an
    /// existing [alpm-install-scriptlet] and validates them.
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
        let (build_info, build_info_digest) = get_build_info(dir)?;

        // Get PackageInfo data and file digest.
        let (package_info, package_info_digest) = get_package_info(dir)?;

        // Compare overlapping metadata of BuildInfo and PackageInfo data.
        compare_build_info_package_info(&build_info, &package_info)?;

        // Get optional scriptlet file.
        let scriptlet = get_scriptlet(dir)?;

        // Get Mtree data and file digest.
        let (mtree, mtree_digest) = get_mtree(dir)?;

        // Get all data files in input_dir, excluding the ALPM-MTREE file, for comparison with
        // ALPM-MTREE data.
        let mut relative_files = relative_files(dir, &[".MTREE"])?;
        trace!("Relative files:\n{relative_files:?}");
        mtree.validate_paths(&InputPaths::new(dir, &relative_files)?)?;

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
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - the file on disk can no longer be read,
    /// - or the file on disk has a changed hash digest.
    ///
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    pub fn install_scriptlet(&self) -> Result<Option<&Path>, crate::Error> {
        if let Some(scriptlet) = self.scriptlet.as_ref() {
            let buf = read(scriptlet.path.as_path()).map_err(|source| crate::Error::IoPath {
                path: scriptlet.path.clone(),
                context: "reading the alpm-install-scriptlet file",
                source,
            })?;
            let path_digest = Blake2b512Checksum::calculate_from(buf);
            if scriptlet.digest != path_digest {
                Err(Error::DigestMismatch {
                    path: scriptlet.path.clone(),
                    digest: self.mtree_digest.clone(),
                    path_digest,
                }
                .into())
            } else {
                Ok(Some(scriptlet.path.as_path()))
            }
        } else {
            Ok(None)
        }
    }

    /// Returns a slice of [`PathBuf`]s representing all data files of the [`PackageInput`].
    pub fn get_data_files(&self) -> &[PathBuf] {
        &self.data_files
    }
}
