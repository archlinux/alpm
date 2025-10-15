//! Facilities for creating a package file from input.

use std::{
    fmt::Display,
    fs::read,
    path::{Path, PathBuf},
};

use alpm_buildinfo::BuildInfo;
use alpm_common::{InputPaths, MetadataFile, relative_files};
use alpm_mtree::{Mtree, mtree::v2::MTREE_PATH_PREFIX};
use alpm_pkginfo::PackageInfo;
use alpm_types::{
    Architecture,
    FullVersion,
    INSTALL_SCRIPTLET_FILE_NAME,
    MetadataFileName,
    Name,
    Packager,
    Sha256Checksum,
};
use log::{debug, trace};

#[cfg(doc)]
use crate::Package;
use crate::scriptlet::check_scriptlet;

/// A single key-value pair from a type of [alpm-package] metadata file.
///
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
#[derive(Clone, Debug)]
pub struct MetadataKeyValue {
    /// The file name of the metadata type.
    pub file_name: MetadataFileName,
    /// The key of one piece of metadata in `file_name`.
    pub key: String,
    /// The value associated with the `key` of one piece of metadata in `file_name`.
    pub value: String,
}

/// A mismatch between metadata of two types of [alpm-package] metadata files.
///
/// Tracks two [`MetadataKeyValue`] instances that describe a mismatch in a key-value pair.
///
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
#[derive(Clone, Debug)]
pub struct MetadataMismatch {
    /// A [`MetadataKeyValue`].
    pub first: MetadataKeyValue,
    /// Another [`MetadataKeyValue`] that differs from the `first`.
    pub second: MetadataKeyValue,
}

impl Display for MetadataMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} => {}\n{}: {} => {}",
            self.first.file_name,
            self.first.key,
            self.first.value,
            self.second.file_name,
            self.second.key,
            self.second.value
        )
    }
}

/// An error that can occur when dealing with package input directories and package files.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The hash digest of a file in an input directory no longer matches.
    #[error(
        "The hash digest {initial_digest} of {path:?} in package input directory {input_dir:?} has changed to {current_digest}"
    )]
    FileHashDigestChanged {
        /// The relative path of a file for which the hash digest does not match.
        path: PathBuf,
        /// The current hash digest of the file.
        current_digest: Sha256Checksum,
        /// The initial hash digest of the file.
        initial_digest: Sha256Checksum,
        /// The path to the package input directory in which the file resides.
        input_dir: PathBuf,
    },

    /// A file is missing in a package input directory.
    #[error("The file {path:?} in package input directory {input_dir:?} is missing")]
    FileIsMissing {
        /// The relative path of the missing file.
        path: PathBuf,
        /// The path to the package input directory.
        input_dir: PathBuf,
    },

    /// Two metadata files have mismatching entries.
    #[error(
        "The following metadata entries are not matching:\n{}",
        mismatches.iter().map(
            |mismatch|
            mismatch.to_string()
        ).collect::<Vec<String>>().join("\n")
    )]
    MetadataMismatch {
        /// A list of mismatches.
        mismatches: Vec<MetadataMismatch>,
    },
}

/// An input directory that is guaranteed to be an absolute directory.
#[derive(Clone, Debug, Eq, PartialEq)]
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

/// Compares the hash digest of a file with the recorded data in an [`Mtree`].
///
/// Takes an `mtree` against which a `file_name` in `input_dir` is checked.
/// Returns the absolute path to the file and a byte buffer that represents the contents of the
/// file.
///
/// # Errors
///
/// Returns an error if
///
/// - the file path (`input_dir` + `file_name`) does not exist,
/// - the file can not be read,
/// - the hash digest of the file does not match that initially recorded in `mtree`,
/// - or the file can not be found in `mtree`.
fn compare_digests(
    mtree: &Mtree,
    input_dir: &InputDir,
    file_name: &str,
) -> Result<(PathBuf, Vec<u8>), crate::Error> {
    let path = input_dir.join(file_name);

    if !path.exists() {
        return Err(Error::FileIsMissing {
            path: PathBuf::from(file_name),
            input_dir: input_dir.to_path_buf(),
        }
        .into());
    }

    // Read the file to a buffer.
    let buf = read(path.as_path()).map_err(|source| crate::Error::IoPath {
        path: path.clone(),
        context: "reading the file",
        source,
    })?;

    // Create a custom file name for searching in ALPM-MTREE entries, as they are prefixed with
    // MTREE_PATH_PREFIX.
    let mtree_file_name = PathBuf::from(MTREE_PATH_PREFIX).join(file_name);

    // Create a SHA-256 hash digest for the file.
    let current_digest = Sha256Checksum::calculate_from(&buf);

    // Check if the initial hash digest of the file - recorded in ALPM-MTREE data - matches.
    if let Some(initial_digest) = match mtree {
        Mtree::V1(paths) => paths.as_slice(),
        Mtree::V2(paths) => paths.as_slice(),
    }
    .iter()
    .find_map(|path| match path {
        alpm_mtree::mtree::v2::Path::File(file) if file.path == mtree_file_name => {
            Some(file.sha256_digest.clone())
        }
        _ => None,
    }) {
        if initial_digest != current_digest {
            return Err(Error::FileHashDigestChanged {
                path: PathBuf::from(file_name),
                current_digest,
                initial_digest,
                input_dir: input_dir.to_path_buf(),
            }
            .into());
        }
    } else {
        return Err(Error::FileIsMissing {
            path: PathBuf::from(file_name),
            input_dir: input_dir.to_path_buf(),
        }
        .into());
    };

    Ok((path, buf))
}

/// Returns whether an [alpm-install-scriptlet] exists in an input directory.
///
/// # Errors
///
/// Returns an error if
///
/// - the file contents cannot be read to a buffer,
/// - the hash digest of the file does not match that initially recorded in `mtree`,
/// - or the file contents do not represent valid [alpm-install-scriptlet] data.
///
/// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
fn get_install_scriptlet(
    input_dir: &InputDir,
    mtree: &Mtree,
) -> Result<Option<PathBuf>, crate::Error> {
    debug!("Check that an alpm-install-scriptlet is valid if it exists in {input_dir:?}.");

    let path = match compare_digests(mtree, input_dir, INSTALL_SCRIPTLET_FILE_NAME) {
        Err(crate::Error::Input(Error::FileIsMissing { .. })) => return Ok(None),
        Err(error) => return Err(error),
        Ok((path, _buf)) => path,
    };

    // Validate the scriptlet.
    check_scriptlet(&path)?;

    Ok(Some(path))
}

/// Returns a [`BuildInfo`] from a BUILDINFO file in an input directory.
///
/// # Errors
///
/// Returns an error if
///
/// - the file does not exist,
/// - the file contents cannot be read to a buffer,
/// - the hash digest of the file does not match that initially recorded in `mtree`,
/// - or the file contents do not represent valid [`BuildInfo`] data.
fn get_build_info(input_dir: &InputDir, mtree: &Mtree) -> Result<BuildInfo, crate::Error> {
    debug!("Check that a valid BUILDINFO file exists in {input_dir:?}.");

    let (_path, buf) = compare_digests(mtree, input_dir, MetadataFileName::BuildInfo.as_ref())?;

    BuildInfo::from_reader(buf.as_slice()).map_err(crate::Error::AlpmBuildInfo)
}

/// Returns a [`PackageInfo`] from a PKGINFO file in an input directory.
///
/// # Errors
///
/// Returns an error if
///
/// - the file does not exist,
/// - the file contents cannot be read to a buffer,
/// - the hash digest of the file does not match that initially recorded in `mtree`,
/// - or the file contents do not represent valid [`PackageInfo`] data.
fn get_package_info(input_dir: &InputDir, mtree: &Mtree) -> Result<PackageInfo, crate::Error> {
    debug!("Check that a valid PKGINFO file exists in {input_dir:?}.");

    let (_path, buf) = compare_digests(mtree, input_dir, MetadataFileName::PackageInfo.as_ref())?;

    PackageInfo::from_reader(buf.as_slice()).map_err(crate::Error::AlpmPackageInfo)
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
fn get_mtree(input_dir: &InputDir) -> Result<(Mtree, Sha256Checksum), crate::Error> {
    debug!("Check that a valid .MTREE file exists in {input_dir:?}.");
    let file_name = PathBuf::from(MetadataFileName::Mtree.as_ref());
    let path = input_dir.join(file_name.as_path());

    if !path.exists() {
        return Err(Error::FileIsMissing {
            path: file_name,
            input_dir: input_dir.to_path_buf(),
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
    let mtree_digest = Sha256Checksum::calculate_from(buf);

    Ok((mtree, mtree_digest))
}

/// The comparison intersection between two different types of metadata files.
///
/// This is used to allow for a basic data comparison between [`BuildInfo`] and [`PackageInfo`].
#[derive(Clone, Debug)]
pub struct MetadataComparison<'a> {
    /// The [alpm-package-name] encoded in the metadata file.
    ///
    /// [alpm-package-name]: https://alpm.archlinux.page/specifications/alpm-package-name.7.html
    pub package_name: &'a Name,
    /// The alpm-package-base encoded in the metadata file.
    pub package_base: &'a Name,
    /// The [alpm-package-version] encoded in the metadata file.
    ///
    /// [alpm-package-version]: https://alpm.archlinux.page/specifications/alpm-package-version.7.html
    pub version: &'a FullVersion,
    /// The [alpm-architecture] encoded in the metadata file.
    ///
    /// [alpm-architecture]: https://alpm.archlinux.page/specifications/alpm-architecture.7.html
    pub architecture: Architecture,
    /// The packager encoded in the metadata file.
    pub packager: &'a Packager,
    /// The date in seconds since the epoch when the package has been built as encoded in the
    /// metadata file.
    pub build_date: i64,
}

impl<'a> From<&'a BuildInfo> for MetadataComparison<'a> {
    /// Creates a [`MetadataComparison`] from a [`BuildInfo`].
    fn from(value: &'a BuildInfo) -> Self {
        match value {
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
}

impl<'a> From<&'a PackageInfo> for MetadataComparison<'a> {
    /// Creates a [`MetadataComparison`] from a [`PackageInfo`].
    fn from(value: &'a PackageInfo) -> Self {
        match value {
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
    let build_info_compare: MetadataComparison<'_> = build_info.into();
    let package_info_compare: MetadataComparison<'_> = package_info.into();
    let mut mismatches = Vec::new();

    let comparisons = [
        (
            (build_info_compare.package_name.to_string(), "pkgname"),
            (package_info_compare.package_name.to_string(), "pkgname"),
        ),
        (
            (build_info_compare.package_base.to_string(), "pkgbase"),
            (package_info_compare.package_base.to_string(), "pkgbase"),
        ),
        (
            (build_info_compare.version.to_string(), "pkgver"),
            (package_info_compare.version.to_string(), "pkgver"),
        ),
        (
            (build_info_compare.architecture.to_string(), "pkgarch"),
            (package_info_compare.architecture.to_string(), "arch"),
        ),
        (
            (build_info_compare.packager.to_string(), "packager"),
            (package_info_compare.packager.to_string(), "packager"),
        ),
        (
            (build_info_compare.build_date.to_string(), "builddate"),
            (package_info_compare.build_date.to_string(), "builddate"),
        ),
    ];
    for comparison in comparisons {
        if comparison.0.0 != comparison.1.0 {
            mismatches.push(MetadataMismatch {
                first: MetadataKeyValue {
                    file_name: MetadataFileName::BuildInfo,
                    key: comparison.0.1.to_string(),
                    value: comparison.0.0,
                },
                second: MetadataKeyValue {
                    file_name: MetadataFileName::PackageInfo,
                    key: comparison.1.1.to_string(),
                    value: comparison.1.0,
                },
            })
        }
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
    package_info: PackageInfo,
    mtree: Mtree,
    mtree_digest: Sha256Checksum,
    input_dir: InputDir,
    scriptlet: Option<PathBuf>,
    relative_paths: Vec<PathBuf>,
}

impl PackageInput {
    /// Returns the input directory of the [`PackageInput`] as [`Path`] reference.
    pub fn input_dir(&self) -> &Path {
        self.input_dir.as_ref()
    }

    /// Returns a reference to the [`BuildInfo`] data of the [`PackageInput`].
    ///
    /// # Note
    ///
    /// The [`BuildInfo`] data relates directly to an on-disk file tracked by the
    /// [`PackageInput`]. This method provides access to the data as present during the creation
    /// of the [`PackageInput`]. While the data can be guaranteed to be correct, the on-disk
    /// file may have changed between creation of the [`PackageInput`] and the call of this method.
    pub fn build_info(&self) -> &BuildInfo {
        &self.build_info
    }

    /// Returns a reference to the [`PackageInfo`] data of the [`PackageInput`].
    ///
    /// # Note
    ///
    /// The [`PackageInfo`] data relates directly to an on-disk file tracked by the
    /// [`PackageInput`]. This method provides access to the data as present during the creation
    /// of the [`PackageInput`]. While the data can be guaranteed to be correct, the on-disk
    /// file may have changed between creation of the [`PackageInput`] and the call of this method.
    pub fn package_info(&self) -> &PackageInfo {
        &self.package_info
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
        let file_name = PathBuf::from(MetadataFileName::Mtree.as_ref());
        let path = self.input_dir.join(file_name.as_path());
        let buf = read(path.as_path()).map_err(|source| crate::Error::IoPath {
            path: path.clone(),
            context: "reading the ALPM-MTREE file",
            source,
        })?;
        let current_digest = Sha256Checksum::calculate_from(buf);
        if current_digest != self.mtree_digest {
            return Err(Error::FileHashDigestChanged {
                path: file_name,
                current_digest,
                initial_digest: self.mtree_digest.clone(),
                input_dir: self.input_dir.to_path_buf(),
            }
            .into());
        }

        Ok(&self.mtree)
    }

    /// Returns the optional [alpm-install-scriptlet] of the [`PackageInput`] as [`Path`] reference.
    ///
    /// # Note
    ///
    /// The [alpm-install-scriptlet] path relates directly to an on-disk file tracked by the
    /// [`PackageInput`]. This method provides access to the data as present during the creation
    /// of the [`PackageInput`]. While the data can be guaranteed to be correct, the on-disk
    /// file may have changed between creation of the [`PackageInput`] and the call of this method.
    ///
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    pub fn install_scriptlet(&self) -> Option<&Path> {
        self.scriptlet.as_deref()
    }

    /// Returns all paths relative to the [`PackageInput`]'s input directory.
    pub fn relative_paths(&self) -> &[PathBuf] {
        &self.relative_paths
    }

    /// Returns an [`InputPaths`] for the input directory and all relative paths contained in it.
    pub fn input_paths(&self) -> Result<InputPaths<'_, '_>, crate::Error> {
        Ok(InputPaths::new(
            self.input_dir.as_path(),
            &self.relative_paths,
        )?)
    }
}

impl TryFrom<InputDir> for PackageInput {
    type Error = crate::Error;

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
    /// - `value` is not a valid [`InputDir`],
    /// - there is no valid [BUILDINFO] file,
    /// - there is no valid [ALPM-MTREE] file,
    /// - there is no valid [PKGINFO] file,
    /// - or one of the files below `dir` does not match the [ALPM-MTREE] data.
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    /// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
    /// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    fn try_from(value: InputDir) -> Result<Self, Self::Error> {
        debug!("Create PackageInput from path {value:?}");

        // Get Mtree data and file digest.
        let (mtree, mtree_digest) = get_mtree(&value)?;

        // Get all relative paths in value.
        let relative_paths = relative_files(&value, &[])?;
        trace!("Relative files:\n{relative_paths:?}");

        // When comparing with ALPM-MTREE data, exclude the ALPM-MTREE file.
        let relative_mtree_paths: Vec<PathBuf> = relative_paths
            .iter()
            .filter(|path| path.as_os_str() != MetadataFileName::Mtree.as_ref())
            .cloned()
            .collect();
        mtree.validate_paths(&InputPaths::new(value.as_ref(), &relative_mtree_paths)?)?;

        // Get PackageInfo data and file digest.
        let package_info = get_package_info(&value, &mtree)?;
        // Get BuildInfo data and file digest.
        let build_info = get_build_info(&value, &mtree)?;

        // Compare overlapping metadata of BuildInfo and PackageInfo data.
        compare_build_info_package_info(&build_info, &package_info)?;

        // Get optional scriptlet file.
        let scriptlet = get_install_scriptlet(&value, &mtree)?;

        Ok(Self {
            build_info,
            package_info,
            mtree,
            mtree_digest,
            input_dir: value,
            scriptlet,
            relative_paths,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, str::FromStr};

    use rstest::rstest;
    use tempfile::tempdir;
    use testresult::TestResult;

    use super::*;

    /// Ensures that a [`MetadataMismatch`] has mismatching values.
    ///
    /// This test is mostly here for coverage improvement.
    #[test]
    fn metadata_mismatch() -> TestResult {
        let mismatch = MetadataMismatch {
            first: MetadataKeyValue {
                file_name: MetadataFileName::BuildInfo,
                key: "pkgname".to_string(),
                value: "example".to_string(),
            },
            second: MetadataKeyValue {
                file_name: MetadataFileName::PackageInfo,
                key: "pkgname".to_string(),
                value: "other-example".to_string(),
            },
        };

        assert_eq!(mismatch.first.key, mismatch.second.key);
        assert_ne!(mismatch.first.value, mismatch.second.value);
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
                    assert_eq!(mismatch.first.key, expected.0);
                    assert_eq!(mismatch.second.key, expected.1);
                }
                _ => return Err("Did not return the correct error variant".into()),
            }
        } else {
            return Err("Should have returned an error but succeeded".into());
        }

        Ok(())
    }
}
