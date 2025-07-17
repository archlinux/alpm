//! Structs and enums to gather, represent and provide data for linting.

use std::{fs, path::Path};

use alpm_buildinfo::BuildInfo;
use alpm_common::MetadataFile;
use alpm_pkginfo::PackageInfo;
use alpm_srcinfo::{SourceInfo, SourceInfoV1};
use alpm_types::{MetadataFileName, PKGBUILD_FILE_NAME, SRCINFO_FILE_NAME};

use crate::{Error, LintScope};

/// The resources used by lints during a single lint run.
// We allow the large enum variant, as we usually only have a single one or at least very few of
// these in memory. Not boxing everything simply makes it more ergonomic to work with.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum Resources {
    /// All resources of a package source repository.
    SourceRepository {
        /// The source info file generated from the PKGBUILD.
        ///
        /// We cannot lint the PKGBUILD directly, hence we have to convert it into a SourceInfo
        /// representation first.
        package_build_source_info: SourceInfo,
        /// The parsed `.SRCINFO` file from the package source repository.
        source_info: SourceInfo,
    },
    /// All resources of a single package.
    Package {
        /// The parsed `.PKGINFO` file.
        package_info: PackageInfo,
        /// The parsed `.BUILDINFO` file.
        build_info: BuildInfo,
    },
    /// A singular BuildInfo file.
    BuildInfo(BuildInfo),
    /// A singular `.PKGINFO`. file.
    PackageInfo(PackageInfo),
    /// A singular PKGBUILD file.
    PackageBuild(SourceInfo),
    /// A singular `.SRCINFO` file.
    SourceInfo(SourceInfo),
}

impl Resources {
    /// Return the respective [`LintScope`] for these [`Resources`].
    pub fn scope(&self) -> LintScope {
        match self {
            Resources::SourceRepository { .. } => LintScope::SourceRepository,
            Resources::Package { .. } => LintScope::Package,
            Resources::BuildInfo(_) => LintScope::BuildInfo,
            Resources::PackageInfo(_) => LintScope::PackageInfo,
            Resources::PackageBuild(_) => LintScope::PackageBuild,
            Resources::SourceInfo(_) => LintScope::SourceInfo,
        }
    }

    /// Gather all required files and other resources from a given path based on the specified
    /// [`LintScope`].
    ///
    /// Metadata or ALPM files are detected by their well-known filenames.
    ///
    /// # Errors
    ///
    /// - Files that are required for a scope don't exist.
    /// - Files cannot be opened or read.
    /// - Files contain invalid data and/or cannot be parsed.
    pub fn gather(path: &Path, scope: LintScope) -> Result<Self, Error> {
        if scope.is_single_file() {
            return Self::gather_file(path, scope);
        }

        let mut metadata = fs::metadata(path).map_err(|source| Error::IoPath {
            path: path.to_owned(),
            context: "getting metadata of path",
            source,
        })?;

        // If the destiny is a symlink, follow it.
        if metadata.is_symlink() {
            metadata = fs::symlink_metadata(path).map_err(|source| Error::IoPath {
                path: path.to_owned(),
                context: "getting symlink metadata of path",
                source,
            })?;
        }

        // Early check that we're indeed working with a directory
        if !metadata.is_dir() {
            return Err(Error::InvalidPathForLintScope {
                path: path.to_owned(),
                scope,
                expected: "file",
            });
        }

        let resource = match scope {
            LintScope::BuildInfo
            | LintScope::PackageBuild
            | LintScope::PackageInfo
            | LintScope::SourceInfo => {
                return Err(Error::InvalidLintScope {
                    scope,
                    function: "Resource::gather_file",
                    expected: "single file lint scope",
                });
            }
            LintScope::SourceRepository => Resources::SourceRepository {
                package_build_source_info: SourceInfo::V1(SourceInfoV1::from_pkgbuild(
                    &path.join(PKGBUILD_FILE_NAME),
                )?),
                source_info: SourceInfo::from_file_with_schema(path.join(SRCINFO_FILE_NAME), None)?,
            },
            LintScope::Package => Resources::Package {
                package_info: PackageInfo::from_file_with_schema(
                    path.join(MetadataFileName::PackageInfo.to_string()),
                    None,
                )?,
                build_info: BuildInfo::from_file_with_schema(
                    path.join(MetadataFileName::BuildInfo.to_string()),
                    None,
                )?,
            },
        };

        Ok(resource)
    }

    /// Gather a single file from a direct path and a [`LintScope`].
    ///
    /// Since the path is direct, the filename is not important for this function.
    /// The type of metadata file is determined by the
    ///
    /// # Errors
    ///
    /// - A scope that requires more than a single file is provided.
    /// - Files that are required for a scope don't exist.
    /// - Files cannot be opened or read.
    /// - Files contain invalid data and/or cannot be parsed.
    pub fn gather_file(path: &Path, scope: LintScope) -> Result<Self, Error> {
        let mut metadata = fs::metadata(path).map_err(|source| Error::IoPath {
            path: path.to_owned(),
            context: "getting metadata of path",
            source,
        })?;

        // If the destiny is a symlink, follow it.
        if metadata.is_symlink() {
            metadata = fs::symlink_metadata(path).map_err(|source| Error::IoPath {
                path: path.to_owned(),
                context: "getting symlink metadata of path",
                source,
            })?;
        }

        // Check that we're indeed working with a file.
        // If we're in a directory, append the expected filename.
        let path = if metadata.is_dir() {
            let filename = match scope {
                LintScope::SourceRepository | LintScope::Package => {
                    return Err(Error::InvalidLintScope {
                        scope,
                        function: "Resource::gather_file",
                        expected: "single file lint scope",
                    });
                }
                LintScope::BuildInfo => MetadataFileName::BuildInfo.to_string(),
                LintScope::PackageBuild => PKGBUILD_FILE_NAME.to_string(),
                LintScope::PackageInfo => MetadataFileName::PackageInfo.to_string(),
                LintScope::SourceInfo => SRCINFO_FILE_NAME.to_string(),
            };

            path.join(filename)
        } else {
            path.to_owned()
        };

        let resource = match scope {
            LintScope::SourceRepository | LintScope::Package => {
                return Err(Error::InvalidLintScope {
                    scope,
                    function: "Resource::gather_file",
                    expected: "single file lint scope",
                });
            }
            LintScope::BuildInfo => Self::BuildInfo(BuildInfo::from_file_with_schema(path, None)?),
            LintScope::PackageBuild => {
                Self::PackageBuild(SourceInfo::V1(SourceInfoV1::from_pkgbuild(&path)?))
            }
            LintScope::PackageInfo => {
                Self::PackageInfo(PackageInfo::from_file_with_schema(path, None)?)
            }
            LintScope::SourceInfo => {
                Self::SourceInfo(SourceInfo::from_file_with_schema(path, None)?)
            }
        };

        Ok(resource)
    }
}
