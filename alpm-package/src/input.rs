//! Facilities for creating a package file from input.

use std::path::{Path, PathBuf};

use alpm_buildinfo::BuildInfo;
use alpm_common::{MetadataFile, relative_data_files};
use alpm_mtree::Mtree;
use alpm_pkginfo::PackageInfo;
use alpm_types::{INSTALL_SCRIPTLET_FILE_NAME, MetadataFileName};
use log::debug;

#[cfg(doc)]
use crate::Package;
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

    /// A path in the list of files representing package data is invalid.
    #[error("The package data path {path} is invalid, because {context}")]
    PackageDataInvalidPath {
        /// The invalid package data path.
        path: PathBuf,
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "The package data path {path} is invalid,
        /// because ".
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

/// A package input directory.
#[derive(Clone, Debug)]
pub struct PackageInput {
    build_info: BuildInfo,
    package_info: PackageInfo,
    mtree: Mtree,
    base_dir: PathBuf,
    scriptlet: Option<PathBuf>,
    data_files: Vec<PathBuf>,
}

impl PackageInput {
    /// Creates a new [`PackageInput`].
    ///
    /// Takes a `base_dir` in which all data resides.
    /// Further takes `build_info`, `package_info`, and `mtree` data, an optional `scriptlet` (see
    /// [alpm-install-scriptlet]) and `data_files` (a list of file and directory paths, relative to
    /// `base_dir`).
    ///
    /// A [`PackageInput`] is used to create a [`Package`] and thus the provided metadata (i.e.
    /// `build_info`, `package_info`, `mtree`) must match the data contained in the files in
    /// `base_dir`.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - `base_dir` does not exist or is not a directory,
    /// - one of the paths in `data_files` does not exist below `base_dir`,
    /// - the [ALPM-MTREE], [BUILDINFO] or [PKGINFO] metadata files or the [alpm-install-scriptlet]
    ///   file are contained in `data_files`,
    /// - a `scriptlet` is provided but it is not contained in `base_dir` or is invalid,
    /// - no [ALPM-MTREE] file exists in `base_dir` or the contents of that file does not match
    ///   `mtree`,
    /// - a [BUILDINFO] file does not exist in `base_dir`, or is not mentioned in the [ALPM-MTREE]
    ///   data, or does not match `build_info`,
    /// - a [PKGINFO] file does not exist in `base_dir`, or is not mentioned in the [ALPM-MTREE]
    ///   data, or does not match `package_info`,
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    /// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
    /// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    pub fn new(
        base_dir: PathBuf,
        build_info: BuildInfo,
        package_info: PackageInfo,
        mtree: Mtree,
        scriptlet: Option<PathBuf>,
        data_files: Vec<PathBuf>,
    ) -> Result<Self, crate::Error> {
        debug!("Creating PackageInput");

        // Check base_dir exists and is a directory.
        debug!("Check that {base_dir:?} exists and is a directory.");
        if !base_dir.exists() {
            return Err(crate::Error::PathDoesNotExist { path: base_dir });
        }
        if !base_dir.is_dir() {
            return Err(crate::Error::PathNotADir { path: base_dir });
        }

        // Check that all data_files exist and are in base_dir.
        debug!("Check that all data files exist in {base_dir:?}.");
        debug!(
            "Check that neither .BUILDINFO, .MTREE, .PKGINFO nor .INSTALL files are contained in data files."
        );
        for path in data_files.iter() {
            let resolved_path = base_dir.join(path);
            if !resolved_path.exists() {
                return Err(crate::Error::PathDoesNotExist {
                    path: resolved_path,
                });
            }

            // Make sure metadata and script files are not in the data files.
            if path.ends_with(INSTALL_SCRIPTLET_FILE_NAME) {
                return Err(Error::PackageDataInvalidPath {
                    path: path.to_path_buf(),
                    context: "alpm-install-scriptlets are not part of package data files",
                }
                .into());
            }
            if path.ends_with(MetadataFileName::Mtree.as_ref()) {
                return Err(Error::PackageDataInvalidPath {
                    path: path.to_path_buf(),
                    context: "ALPM-MTREE files are not part of package data files",
                }
                .into());
            }
            if path.ends_with(MetadataFileName::BuildInfo.as_ref()) {
                return Err(Error::PackageDataInvalidPath {
                    path: path.to_path_buf(),
                    context: "BUILDINFO files are not part of package data files",
                }
                .into());
            }
            if path.ends_with(MetadataFileName::PackageInfo.as_ref()) {
                return Err(Error::PackageDataInvalidPath {
                    path: path.to_path_buf(),
                    context: "PKGINFO files are not part of package data files",
                }
                .into());
            }
        }

        // Check that the alpm-install-scriptlet is in base_dir, exists and is somewhat valid.
        if let Some(path) = scriptlet.as_deref() {
            debug!("Check if .INSTALL exists in {base_dir:?} and is valid.");
            let required_path = base_dir.join(INSTALL_SCRIPTLET_FILE_NAME);
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
            debug!("Check that .MTREE exists in {base_dir:?} and matches the provided metadata.");
            let path = base_dir.join(MetadataFileName::Mtree.as_ref());

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
            debug!(
                "Check that .BUILDINFO exists in {base_dir:?} and .MTREE and matches the provided metadata."
            );
            let mtree_paths = match &mtree {
                Mtree::V1(mtree) | Mtree::V2(mtree) => mtree.as_slice(),
            };
            let path = PathBuf::from(format!("./{}", MetadataFileName::BuildInfo));
            if !mtree_paths.iter().any(|mtree_path| match mtree_path {
                alpm_mtree::mtree::v2::Path::File(file) => file.path == path,
                _ => false,
            }) {
                return Err(Error::MtreePathMissing {
                    path: path.to_path_buf(),
                }
                .into());
            }

            let path = base_dir.join(MetadataFileName::BuildInfo.as_ref());

            if !path.exists() {
                return Err(crate::Error::PathDoesNotExist {
                    path: path.to_path_buf(),
                });
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
            debug!(
                "Check that .PKGINFO exists in {base_dir:?} and .MTREE and matches the provided metadata."
            );
            let mtree_paths = match &mtree {
                Mtree::V1(mtree) | Mtree::V2(mtree) => mtree.as_slice(),
            };
            let path = PathBuf::from(format!("./{}", MetadataFileName::PackageInfo));
            if !mtree_paths.iter().any(|mtree_path| match mtree_path {
                alpm_mtree::mtree::v2::Path::File(file) => file.path == path,
                _ => false,
            }) {
                return Err(Error::MtreePathMissing {
                    path: path.to_path_buf(),
                }
                .into());
            }

            let path = base_dir.join(MetadataFileName::PackageInfo.as_ref());

            if !path.exists() {
                return Err(crate::Error::PathDoesNotExist {
                    path: path.to_path_buf(),
                });
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

    /// Returns the base directory of the [`PackageInput`] as [`Path`] reference.
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Returns a reference to the [`BuildInfo`] data of the [`PackageInput`].
    pub fn build_info(&self) -> &BuildInfo {
        &self.build_info
    }

    /// Returns a reference to the [`PackageInfo`] data of the [`PackageInput`].
    pub fn package_info(&self) -> &PackageInfo {
        &self.package_info
    }

    /// Returns a reference to the [`Mtree`] data of the [`PackageInput`].
    pub fn mtree(&self) -> &Mtree {
        &self.mtree
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

    /// Creates a [`PackageInput`] from path.
    ///
    /// Extracts [BUILDINFO], [ALPM-MTREE] and [PKGINFO] metadata from the relevant files in
    /// `value`.
    /// Detects [alpm-install-scriptlet] files in `value`.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - no valid [BUILDINFO] file is present in `value`,
    /// - no valid [ALPM-MTREE] file is present in `value`,
    /// - or no valid [PKGINFO] file is present in `value`.
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    /// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
    /// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        debug!("Create PackageInput from path {value:?}");

        // Get BuildInfo data
        debug!("Check that a valid .BUILDINFO exists in {value:?}.");
        let buildinfo_path = value.join(MetadataFileName::BuildInfo.as_ref());
        if !buildinfo_path.exists() {
            return Err(Error::MetadataFileMissing {
                metadata_file: MetadataFileName::BuildInfo,
                path: value.to_path_buf(),
            }
            .into());
        }
        let build_info = BuildInfo::from_file(&buildinfo_path).map_err(crate::Error::BuildInfo)?;

        // Get Mtree data
        debug!("Check that a valid .MTREE file exists in {value:?}.");
        let mtree_path = value.join(MetadataFileName::Mtree.as_ref());
        if !mtree_path.exists() {
            return Err(Error::MetadataFileMissing {
                metadata_file: MetadataFileName::Mtree,
                path: value.to_path_buf(),
            }
            .into());
        }
        let mtree = Mtree::from_file(&mtree_path).map_err(crate::Error::Mtree)?;

        // Get PkgInfo data
        debug!("Check that a valid .PKGINFO file exists in {value:?}.");
        let pkginfo_path = value.join(MetadataFileName::PackageInfo.as_ref());
        if !pkginfo_path.exists() {
            return Err(Error::MetadataFileMissing {
                metadata_file: MetadataFileName::PackageInfo,
                path: value.to_path_buf(),
            }
            .into());
        }
        let package_info =
            PackageInfo::from_file(&pkginfo_path).map_err(crate::Error::PackageInfo)?;

        let scriptlet_path = value.join(INSTALL_SCRIPTLET_FILE_NAME);
        let scriptlet = if scriptlet_path.exists() {
            Some(scriptlet_path)
        } else {
            None
        };

        let data_files = relative_data_files(value)?;

        Self::new(
            value.to_path_buf(),
            build_info,
            package_info,
            mtree,
            scriptlet,
            data_files,
        )
    }
}

#[cfg(test)]
mod test {
    use std::{
        fs::{File, create_dir_all},
        io::Write,
        os::unix::fs::symlink,
    };

    use alpm_common::relative_files;
    use rstest::rstest;
    use tempfile::tempdir;
    use testresult::TestResult;

    use super::*;

    pub const VALID_BUILDINFO_V2_DATA: &str = r#"
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
buildenv = envfoo
buildenv = envbar
format = 2
installed = bar-1.2.3-1-any
installed = beh-2.2.3-4-any
options = some_option
options = !other_option
packager = Foobar McFooface <foobar@mcfooface.org>
pkgarch = any
pkgbase = example
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = example
pkgver = 1:1.0.0-1
"#;

    pub const VALID_PKGINFO_V2_DATA: &str = r#"
pkgname = example
pkgbase = example
xdata = pkgtype=pkg
pkgver = 1:1.0.0-1
pkgdesc = A project that does something
url = https://example.org/
builddate = 1729181726
packager = John Doe <john@example.org>
size = 181849963
arch = any
license = GPL-3.0-or-later
replaces = other-package>0.9.0-3
group = package-group
conflict = conflicting-package<1.0.0
provides = some-component
backup = etc/example/config.toml
depend = glibc
optdepend = python: for special-python-script.py
makedepend = cmake
checkdepend = extra-test-tool
"#;

    fn create_data_files(path: impl AsRef<Path>) -> TestResult {
        let path = path.as_ref();
        // Create dummy directory structure
        create_dir_all(path.join("usr/share/foo/bar/baz"))?;
        // Create dummy text file
        let mut output = File::create(path.join("usr/share/foo/beh.txt"))?;
        write!(output, "test")?;
        // Create relative symlink to actual text file
        symlink("../../beh.txt", path.join("usr/share/foo/bar/baz/beh.txt"))?;
        Ok(())
    }

    fn create_metadata_files(path: impl AsRef<Path>) -> TestResult {
        let path = path.as_ref();
        let mut output = File::create(path.join(MetadataFileName::BuildInfo.as_ref()))?;
        write!(output, "{VALID_BUILDINFO_V2_DATA}")?;
        let mut output = File::create(path.join(MetadataFileName::PackageInfo.as_ref()))?;
        write!(output, "{VALID_PKGINFO_V2_DATA}")?;
        Ok(())
    }

    /// Tests the successful collection of relative files in a test directory.
    #[rstest]
    #[case(false)]
    #[case(true)]
    fn relative_files_are_collected_successfully_without_filter(
        #[case] create_metadata: bool,
    ) -> TestResult {
        let tempdir = tempdir()?;
        let mut expected_paths: Vec<PathBuf> = Vec::new();

        create_data_files(tempdir.path())?;

        if create_metadata {
            create_metadata_files(tempdir.path())?;
            expected_paths.push(PathBuf::from(MetadataFileName::BuildInfo.as_ref()));
            expected_paths.push(PathBuf::from(MetadataFileName::PackageInfo.as_ref()));
        }

        for path in [
            PathBuf::from("usr"),
            PathBuf::from("usr/share"),
            PathBuf::from("usr/share/foo"),
            PathBuf::from("usr/share/foo/bar"),
            PathBuf::from("usr/share/foo/bar/baz"),
            PathBuf::from("usr/share/foo/bar/baz/beh.txt"),
            PathBuf::from("usr/share/foo/beh.txt"),
        ] {
            expected_paths.push(path)
        }

        // Collect all files
        let collected_files = relative_files(tempdir, &[])?;
        println!("{:?}", collected_files);

        assert_eq!(expected_paths.as_slice(), collected_files.as_slice());

        Ok(())
    }
}
