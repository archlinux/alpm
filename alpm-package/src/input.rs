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

/// A single key-value pair from a type of [alpm-package] metadata file.
///
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
#[derive(Debug, Clone)]
pub struct MetadataKeyValue {
    /// The file name of the metadata type.
    pub file_type: MetadataFileName,
    /// The key of one piece of metadata in `file_type`.
    pub key: String,
    /// The value associated with the `key` of one piece of metadata in `file_type`.
    pub value: String,
}

/// A mismatch between metadata of two types of [alpm-package] metadata files.
///
/// Tracks two [`MetadataKeyValue`] instances that describe a mismatch in a key-value pair.
///
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
#[derive(Debug, Clone)]
pub struct MetadataMismatch {
    /// One [`MetadataKeyValue`].
    pub one: MetadataKeyValue,
    /// Another [`MetadataKeyValue`] that differs from `one`.
    pub other: MetadataKeyValue,
}

impl Display for MetadataMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} => {}\n{}: {} => {}",
            self.one.file_type,
            self.one.key,
            self.one.value,
            self.other.file_type,
            self.other.key,
            self.other.value
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

    /// A metadata file is missing in a package input directory.
    #[error("There is no {metadata_file} file in package input path {path}")]
    MetadataFileMissing {
        /// The type of the metadata file.
        metadata_file: MetadataFileName,
        /// The path to the package input directory.
        path: PathBuf,
    },

    /// Two metadata files have mismatching entries.
    #[error(
        "The following metadata entries are not matching:\n{}",
        mismatches.iter().fold(String::new(), |mut output, mismatch| {output.push_str(&format!("{}\n", mismatch)); output})
    )]
    MetadataMismatch {
        /// A list of mismatches.
        mismatches: Vec<MetadataMismatch>,
    },
}

/// An input directory that is guaranteed to be an absolute directory.
#[derive(Debug, Clone)]
pub struct InputDir(PathBuf);

impl InputDir {
    /// Creates a new [`InputDir`] from `path`.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - `path` is not absolute,
    /// - `path` does not exist,
    /// - the metadata of `path` cannot be retrieved,
    /// - or `path` is not a directory.
    pub fn new(path: PathBuf) -> Result<Self, crate::Error> {
        if !path.is_absolute() {
            return Err(alpm_common::Error::NonAbsolutePaths {
                paths: vec![path.clone()],
            }
            .into());
        }

        if !path.exists() {
            return Err(crate::Error::PathDoesNotExist { path: path.clone() });
        }

        if !path.is_dir() {
            return Err(alpm_common::Error::NotADirectory { path: path.clone() }.into());
        }

        Ok(Self(path))
    }

    /// Coerces to a Path slice.
    ///
    /// Delegates to [`PathBuf::as_path`].
    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }

    /// Converts a Path to an owned PathBuf.
    ///
    /// Delegates to [`Path::to_path_buf`].
    pub fn to_path_buf(&self) -> PathBuf {
        self.0.to_path_buf()
    }

    /// Creates an owned PathBuf with path adjoined to self.
    ///
    /// Delegates to [`Path::join`].
    pub fn join(&self, path: impl AsRef<Path>) -> PathBuf {
        self.0.join(path)
    }
}

impl AsRef<Path> for InputDir {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

/// Returns an optional hash digest of an [alpm-install-scriptlet].
///
/// # Errors
///
/// Returns an error if
///
/// - the file contents cannot be read to a buffer,
/// - or the file contents do not represent valid [alpm-install-scriptlet] data.
///
/// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
fn get_scriptlet_digest(
    input_dir: impl AsRef<Path>,
) -> Result<Option<Blake2b512Checksum>, crate::Error> {
    let dir = input_dir.as_ref();
    debug!("Check that an alpm-install-scriptlet is valid if it exists in {dir:?}.");
    let path = dir.join(INSTALL_SCRIPTLET_FILE_NAME);

    Ok(if path.as_path().exists() {
        // Validate the scriptlet.
        check_scriptlet(&path)?;

        let buf = read(&path).map_err(|source| crate::Error::IoPath {
            path: path.clone(),
            context: "reading the alpm-install-scriptlet file",
            source,
        })?;

        Some(Blake2b512Checksum::calculate_from(buf))
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
fn get_build_info(
    input_dir: impl AsRef<Path>,
) -> Result<(BuildInfo, Blake2b512Checksum), crate::Error> {
    let dir = input_dir.as_ref();
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
    input_dir: impl AsRef<Path>,
) -> Result<(PackageInfo, Blake2b512Checksum), crate::Error> {
    let dir = input_dir.as_ref();
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
fn get_mtree(input_dir: impl AsRef<Path>) -> Result<(Mtree, Blake2b512Checksum), crate::Error> {
    let dir = input_dir.as_ref();
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

/// The comparison intersection between two different types of metadata files.
///
/// This is used to allow for a basic data comparison between [`BuildInfo`] and [`PackageInfo`].
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
            one: MetadataKeyValue {
                file_type: MetadataFileName::BuildInfo,
                key: "pkgname".to_string(),
                value: build_info_compare.package_name.to_string(),
            },
            other: MetadataKeyValue {
                file_type: MetadataFileName::PackageInfo,
                key: "pkgname".to_string(),
                value: package_info_compare.package_name.to_string(),
            },
        })
    }

    if build_info_compare.package_base != package_info_compare.package_base {
        mismatches.push(MetadataMismatch {
            one: MetadataKeyValue {
                file_type: MetadataFileName::BuildInfo,
                key: "pkgbase".to_string(),
                value: build_info_compare.package_base.to_string(),
            },
            other: MetadataKeyValue {
                file_type: MetadataFileName::PackageInfo,
                key: "pkgbase".to_string(),
                value: package_info_compare.package_base.to_string(),
            },
        })
    }

    if build_info_compare.version != package_info_compare.version {
        mismatches.push(MetadataMismatch {
            one: MetadataKeyValue {
                file_type: MetadataFileName::BuildInfo,
                key: "pkgver".to_string(),
                value: build_info_compare.version.to_string(),
            },
            other: MetadataKeyValue {
                file_type: MetadataFileName::PackageInfo,
                key: "pkgver".to_string(),
                value: package_info_compare.version.to_string(),
            },
        })
    }

    if build_info_compare.architecture != package_info_compare.architecture {
        mismatches.push(MetadataMismatch {
            one: MetadataKeyValue {
                file_type: MetadataFileName::BuildInfo,
                key: "pkgarch".to_string(),
                value: build_info_compare.architecture.to_string(),
            },
            other: MetadataKeyValue {
                file_type: MetadataFileName::PackageInfo,
                key: "arch".to_string(),
                value: package_info_compare.architecture.to_string(),
            },
        })
    }

    if build_info_compare.packager != package_info_compare.packager {
        mismatches.push(MetadataMismatch {
            one: MetadataKeyValue {
                file_type: MetadataFileName::BuildInfo,
                key: "packager".to_string(),
                value: build_info_compare.packager.to_string(),
            },
            other: MetadataKeyValue {
                file_type: MetadataFileName::PackageInfo,
                key: "packager".to_string(),
                value: package_info_compare.packager.to_string(),
            },
        })
    }

    if build_info_compare.build_date != package_info_compare.build_date {
        mismatches.push(MetadataMismatch {
            one: MetadataKeyValue {
                file_type: MetadataFileName::BuildInfo,
                key: "builddate".to_string(),
                value: build_info_compare.build_date.to_string(),
            },
            other: MetadataKeyValue {
                file_type: MetadataFileName::PackageInfo,
                key: "builddate".to_string(),
                value: package_info_compare.build_date.to_string(),
            },
        })
    }

    if !mismatches.is_empty() {
        return Err(Error::MetadataMismatch { mismatches }.into());
    }

    Ok(())
}

/// A package input directory.
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
/// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
/// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
/// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
/// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
#[derive(Clone, Debug)]
pub struct PackageInput {
    build_info: BuildInfo,
    build_info_digest: Blake2b512Checksum,
    package_info: PackageInfo,
    package_info_digest: Blake2b512Checksum,
    mtree: Mtree,
    mtree_digest: Blake2b512Checksum,
    input_dir: InputDir,
    scriptlet_digest: Option<Blake2b512Checksum>,
    relative_paths: Vec<PathBuf>,
}

impl PackageInput {
    /// Creates a [`PackageInput`] from input directory `path`.
    ///
    /// This function reads [ALPM-MTREE], [BUILDINFO] and [PKGINFO] files in `path`, collects the
    /// path of an existing [alpm-install-scriptlet] and validates them.
    /// All data files below `path` are then checked against the [ALPM-MTREE] data.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - `path` is not a valid [`InputDir`],
    /// - there is no valid [BUILDINFO] file,
    /// - there is no valid [ALPM-MTREE] file,
    /// - there is no valid [PKGINFO] file,
    /// - or one of the files below `dir` does not match the [ALPM-MTREE] data.
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    /// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
    /// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    pub fn from_input_dir(input_dir: InputDir) -> Result<Self, crate::Error> {
        debug!("Create PackageInput from path {input_dir:?}");

        // Get BuildInfo data and file digest.
        let (build_info, build_info_digest) = get_build_info(&input_dir)?;

        // Get PackageInfo data and file digest.
        let (package_info, package_info_digest) = get_package_info(&input_dir)?;

        // Compare overlapping metadata of BuildInfo and PackageInfo data.
        compare_build_info_package_info(&build_info, &package_info)?;

        // Get optional scriptlet file.
        let scriptlet_digest = get_scriptlet_digest(&input_dir)?;

        // Get Mtree data and file digest.
        let (mtree, mtree_digest) = get_mtree(&input_dir)?;

        // Get all relative paths in input_dir.
        let relative_paths = relative_files(&input_dir, &[])?;
        trace!("Relative files:\n{relative_paths:?}");

        // When comparing with ALPM-MTREE data, exclud the ALPM-MTREE file.
        let relative_mtree_paths: Vec<PathBuf> = relative_paths
            .iter()
            .filter(|path| path.as_os_str() != MetadataFileName::Mtree.as_ref())
            .cloned()
            .collect();
        mtree.validate_paths(&InputPaths::new(input_dir.as_ref(), &relative_mtree_paths)?)?;

        Ok(Self {
            build_info,
            build_info_digest,
            package_info,
            package_info_digest,
            mtree,
            mtree_digest,
            input_dir,
            scriptlet_digest,
            relative_paths,
        })
    }

    /// Returns the input directory of the [`PackageInput`] as [`Path`] reference.
    pub fn input_dir(&self) -> &Path {
        self.input_dir.as_ref()
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
        let path = self.input_dir.join(MetadataFileName::BuildInfo.as_ref());
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
        let path = self.input_dir.join(MetadataFileName::PackageInfo.as_ref());
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
        let path = self.input_dir.join(MetadataFileName::Mtree.as_ref());
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
    pub fn install_scriptlet(&self) -> Result<Option<PathBuf>, crate::Error> {
        if let Some(scriptlet_digest) = self.scriptlet_digest.as_ref() {
            let scriptlet = self.input_dir.join(INSTALL_SCRIPTLET_FILE_NAME);
            let buf = read(&scriptlet).map_err(|source| crate::Error::IoPath {
                path: scriptlet.clone(),
                context: "reading the alpm-install-scriptlet file",
                source,
            })?;
            let path_digest = Blake2b512Checksum::calculate_from(buf);
            if scriptlet_digest != &path_digest {
                Err(Error::DigestMismatch {
                    path: scriptlet.clone(),
                    digest: scriptlet_digest.clone(),
                    path_digest,
                }
                .into())
            } else {
                Ok(Some(scriptlet.clone()))
            }
        } else {
            Ok(None)
        }
    }

    /// Returns all paths relative to the [`PackageInput`]'s input directory.
    pub fn relative_paths(&self) -> &[PathBuf] {
        &self.relative_paths
    }

    /// Returns an [`InputPaths`] for the input directory and all relative paths contained in it.
    pub fn input_paths(&self) -> Result<InputPaths, crate::Error> {
        Ok(InputPaths::new(
            self.input_dir.as_path(),
            &self.relative_paths,
        )?)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, str::FromStr};

    use rstest::rstest;
    use tempfile::tempdir;
    use testresult::TestResult;

    use super::*;

    #[test]
    fn metadata_mismatch() -> TestResult {
        let mismatch = MetadataMismatch {
            one: MetadataKeyValue {
                file_type: MetadataFileName::BuildInfo,
                key: "pkgname".to_string(),
                value: "example".to_string(),
            },
            other: MetadataKeyValue {
                file_type: MetadataFileName::PackageInfo,
                key: "pkgname".to_string(),
                value: "other-example".to_string(),
            },
        };

        println!("{mismatch}");
        Ok(())
    }

    /// Ensures that [`InputDir::new`] fails on relative paths, non-existing paths and non-directory
    /// paths.
    #[test]
    fn input_dir_new_fails() -> TestResult {
        assert!(matches!(
            InputDir::new(PathBuf::from("test")),
            Err(crate::Error::AlpmCommon(
                alpm_common::Error::NonAbsolutePaths { paths: _ }
            ))
        ));

        let temp_dir = tempdir()?;
        let non_existing_path = temp_dir.path().join("non-existing");
        assert!(matches!(
            InputDir::new(non_existing_path),
            Err(crate::Error::PathDoesNotExist { path: _ })
        ));

        let file_path = temp_dir.path().join("non-existing");
        let _file = File::create(&file_path)?;
        assert!(matches!(
            InputDir::new(file_path),
            Err(crate::Error::AlpmCommon(
                alpm_common::Error::NotADirectory { path: _ }
            ))
        ));

        Ok(())
    }

    /// Ensures that [`InputDir::to_path_buf`] works.
    #[test]
    fn input_dir_to_path_buf() -> TestResult {
        let temp_dir = tempdir()?;
        let dir = temp_dir.path();
        let input_dir = InputDir::new(dir.to_path_buf())?;

        assert_eq!(input_dir.to_path_buf(), dir.to_path_buf());

        Ok(())
    }

    const PKGNAME_MISMATCH: &[&str; 2] = &[
        r#"
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
format = 2
packager = John Doe <john@example.org>
pkgarch = any
pkgbase = example
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = example
pkgver = 1:1.0.0-1
"#,
        r#"
pkgname = example-different
pkgbase = example
xdata = pkgtype=pkg
pkgver = 1:1.0.0-1
pkgdesc = A project that does something
url = https://example.org/
builddate = 1
packager = John Doe <john@example.org>
size = 181849963
arch = any
"#,
    ];

    const PKGBASE_MISMATCH: &[&str; 2] = &[
        r#"
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
format = 2
packager = John Doe <john@example.org>
pkgarch = any
pkgbase = example
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = example
pkgver = 1:1.0.0-1
"#,
        r#"
pkgname = example
pkgbase = example-different
xdata = pkgtype=pkg
pkgver = 1:1.0.0-1
pkgdesc = A project that does something
url = https://example.org/
builddate = 1
packager = John Doe <john@example.org>
size = 181849963
arch = any
"#,
    ];

    const VERSION_MISMATCH: &[&str; 2] = &[
        r#"
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
format = 2
packager = John Doe <john@example.org>
pkgarch = any
pkgbase = example
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = example
pkgver = 1:1.0.0-1
"#,
        r#"
pkgname = example
pkgbase = example
xdata = pkgtype=pkg
pkgver = 1.0.0-1
pkgdesc = A project that does something
url = https://example.org/
builddate = 1
packager = John Doe <john@example.org>
size = 181849963
arch = any
"#,
    ];

    const ARCHITECTURE_MISMATCH: &[&str; 2] = &[
        r#"
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
format = 2
packager = John Doe <john@example.org>
pkgarch = any
pkgbase = example
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = example
pkgver = 1:1.0.0-1
"#,
        r#"
pkgname = example
pkgbase = example
xdata = pkgtype=pkg
pkgver = 1:1.0.0-1
pkgdesc = A project that does something
url = https://example.org/
builddate = 1
packager = John Doe <john@example.org>
size = 181849963
arch = x86_64
"#,
    ];

    const PACKAGER_MISMATCH: &[&str; 2] = &[
        r#"
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
format = 2
packager = John Doe <john@example.org>
pkgarch = any
pkgbase = example
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = example
pkgver = 1:1.0.0-1
"#,
        r#"
pkgname = example
pkgbase = example
xdata = pkgtype=pkg
pkgver = 1:1.0.0-1
pkgdesc = A project that does something
url = https://example.org/
builddate = 1
packager = Jane Doe <jane@example.org>
size = 181849963
arch = any
"#,
    ];

    const BUILD_DATE_MISMATCH: &[&str; 2] = &[
        r#"
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
format = 2
packager = John Doe <john@example.org>
pkgarch = any
pkgbase = example
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = example
pkgver = 1:1.0.0-1
"#,
        r#"
pkgname = example
pkgbase = example
xdata = pkgtype=pkg
pkgver = 1:1.0.0-1
pkgdesc = A project that does something
url = https://example.org/
builddate = 2
packager = John Doe <john@example.org>
size = 181849963
arch = any
"#,
    ];

    /// Ensures that [`compare_build_info_package_info`] fails on mismatches in [`BuildInfo`] and
    /// [`PackageInfo`].
    #[rstest]
    #[case::pkgname_mismatch(PKGNAME_MISMATCH, ("pkgname", "pkgname"))]
    #[case::pkgbase_mismatch(PKGBASE_MISMATCH, ("pkgbase", "pkgbase"))]
    #[case::version_mismatch(VERSION_MISMATCH, ("pkgver", "pkgver"))]
    #[case::architecture_mismatch(ARCHITECTURE_MISMATCH, ("pkgarch", "arch"))]
    #[case::packager_mismatch(PACKAGER_MISMATCH, ("packager", "packager"))]
    #[case::build_date_mismatch(BUILD_DATE_MISMATCH, ("builddate", "builddate"))]
    fn test_compare_build_info_package_info_fails(
        #[case] metadata: &[&str; 2],
        #[case] expected: (&str, &str),
    ) -> TestResult {
        let build_info = BuildInfo::from_str(metadata[0])?;
        let package_info = PackageInfo::from_str(metadata[1])?;

        if let Err(error) = compare_build_info_package_info(&build_info, &package_info) {
            match error {
                crate::Error::Input(crate::input::Error::MetadataMismatch { mismatches }) => {
                    if mismatches.len() != 1 {
                        return Err("There should be exactly one metadata mismatch".into());
                    }
                    let Some(mismatch) = mismatches.first() else {
                        return Err("There should be at least one metadata mismatch".into());
                    };
                    assert_eq!(mismatch.one.key, expected.0);
                    assert_eq!(mismatch.other.key, expected.1);
                }
                _ => return Err("Did not return the correct error variant".into()),
            }
        } else {
            return Err("Should have returned an error but succeeded".into());
        }

        Ok(())
    }
}
