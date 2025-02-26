//! Package creation pipeline.

use std::path::PathBuf;

#[cfg(doc)]
use alpm_pkginfo::PackageInfo;
use alpm_types::PackageFileName;

#[cfg(doc)]
use crate::package::Package;
use crate::{compression::CompressionSettings, input::PackageInput};

/// A pipeline for tracking input and output for an [alpm-package].
///
/// Tracks a [`PackageInput`], optional [`CompressionSettings`] and an `output_dir` in which an
/// [alpm-package] is placed after creation.
///
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
#[derive(Clone, Debug)]
pub struct PackageCreationPipeline {
    /// The [`CompressionSettings`] that are used when creating a [`Package`].
    pub compression: Option<CompressionSettings>,

    /// The [`PackageInput`] that is used when creating a [`Package`].
    pub package_input: PackageInput,

    /// The directory in which a [`Package`] is created.
    pub output_dir: PathBuf,
}

impl TryFrom<&PackageCreationPipeline> for PackageFileName {
    type Error = crate::Error;

    /// Creates a [`PackageFileName`] from a [`PackagePipeline`] reference.
    ///
    /// # Errors
    ///
    /// Returns an error if the [`PackageInfo`] tracked by `value` is no longer valid or present.
    fn try_from(value: &PackageCreationPipeline) -> Result<Self, Self::Error> {
        Ok(Self::new(
            match value.package_input.package_info()? {
                alpm_pkginfo::PackageInfo::V1(package_info) => package_info.pkgname().clone(),
                alpm_pkginfo::PackageInfo::V2(package_info) => package_info.pkgname().clone(),
            },
            match value.package_input.package_info()? {
                alpm_pkginfo::PackageInfo::V1(package_info) => package_info.pkgver().clone(),
                alpm_pkginfo::PackageInfo::V2(package_info) => package_info.pkgver().clone(),
            },
            match value.package_input.package_info()? {
                alpm_pkginfo::PackageInfo::V1(package_info) => *package_info.arch(),
                alpm_pkginfo::PackageInfo::V2(package_info) => *package_info.arch(),
            },
            value.compression.as_ref().map(|settings| settings.into()),
        )?)
    }
}
