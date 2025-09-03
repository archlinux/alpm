//! Types to gather, represent and provide data for linting.

use std::{fs::metadata, path::Path};

use alpm_buildinfo::BuildInfo;
use alpm_common::MetadataFile;
use alpm_pkginfo::PackageInfo;
use alpm_srcinfo::{SourceInfo, SourceInfoV1};
use alpm_types::{MetadataFileName, PKGBUILD_FILE_NAME, SRCINFO_FILE_NAME};

use crate::{Error, LintScope};

/// The resources used by lints during a single lint run.
// We allow the large enum variant, as we usually only have a single one or at most **very** few
// of these in memory. Not boxing everything simply makes it more ergonomic to work with.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum Resources {
    /// All resources of a package source repository.
    SourceRepository {
        /// The [SRCINFO] file generated from the [PKGBUILD].
        ///
        /// We cannot lint the [PKGBUILD] directly, hence we have to convert it into a
        /// [`SourceInfo`] representation first.
        ///
        /// [PKGBUILD]: https://man.archlinux.org/man/PKGBUILD.5
        /// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
        package_build_source_info: SourceInfo,
        /// The parsed [SRCINFO] file from the package source repository.
        ///
        /// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
        source_info: SourceInfo,
    },
    /// All resources of a single package.
    Package {
        /// The parsed [PKGINFO] file.
        ///
        /// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
        package_info: PackageInfo,
        /// The parsed [BUILDINFO] file.
        ///
        /// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
        build_info: BuildInfo,
    },
    /// A singular [BUILDINFO] file.
    ///
    /// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
    BuildInfo(BuildInfo),
    /// A singular [PKGINFO] file.
    ///
    /// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
    PackageInfo(PackageInfo),
    /// A singular [PKGBUILD] file.
    ///
    /// We cannot lint the [PKGBUILD] directly, hence we have to convert it into a [`SourceInfo`]
    /// representation first.
    ///
    /// [PKGBUILD]: https://man.archlinux.org/man/PKGBUILD.5
    /// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
    PackageBuild(SourceInfo),
    /// A singular [SRCINFO] file.
    ///
    /// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
    SourceInfo(SourceInfo),
}

impl Resources {
    /// Returns the [`LintScope`] for the [`Resources`].
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

    /// Creates a [`Resources`] from a file path and a [`LintScope`].
    ///
    /// Gathers all files and other resources in a `path` in the context of a `scope`.
    /// All ALPM related files are detected by their well-known file names.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - files that are required for a scope don't exist,
    /// - files cannot be opened or read,
    /// - or files contain invalid data and/or cannot be parsed successfully.
    pub fn gather(path: &Path, scope: LintScope) -> Result<Self, Error> {
        if scope.is_single_file() {
            return Self::gather_file(path, scope);
        }

        // `metadata` automatically follows symlinks, so we get the target's metadata
        let metadata = metadata(path).map_err(|source| Error::IoPath {
            path: path.to_owned(),
            context: "getting metadata of path",
            source,
        })?;

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

    /// Creates a [`Resources`] from a single file at a path and a [`LintScope`].
    ///
    /// Gathers a single file at `path` in the context of a `scope`.
    /// Since the path is direct, the filename is not important for this function.
    /// The type of metadata file is pre-determined by the [`LintScope`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - a scope that requires more than a single file is provided,
    /// - the metadata of the file at `path` cannot be retrieved,
    /// - `path` represents a directory,
    /// - the file cannot be opened or read,
    /// - or the file contains invalid data and/or cannot be parsed.
    pub fn gather_file(path: &Path, scope: LintScope) -> Result<Self, Error> {
        // `metadata` automatically follows symlinks, so we get the target's metadata
        let metadata = metadata(path).map_err(|source| Error::IoPath {
            path: path.to_owned(),
            context: "getting metadata of path",
            source,
        })?;

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
