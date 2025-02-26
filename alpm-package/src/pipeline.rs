//! Package build pipeline.

use std::path::PathBuf;

use alpm_types::PackageFileName;

#[cfg(doc)]
use crate::package::Package;
use crate::{compression::CompressionSettings, input::PackageInput};

/// A pipeline for creating a [`Package`] from a [`PackageInput`].
#[derive(Clone, Debug)]
pub struct PackagePipeline {
    /// The [`CompressionSettings`] that is used when creating a [`Package`].
    pub compression: CompressionSettings,

    /// The [`PackageInput`] that is used when creating a [`Package`].
    pub package_input: PackageInput,

    /// The directory in which a [`Package`] is created.
    pub output_dir: PathBuf,
}

impl PackagePipeline {
    /// Creates a new [`PackagePipeline`].
    pub fn new(compression: CompressionSettings, input: PackageInput, output_dir: PathBuf) -> Self {
        Self {
            compression,
            package_input: input,
            output_dir,
        }
    }
}

impl From<&PackagePipeline> for PackageFileName {
    /// Creates a [`PackageFileName`] from a [`PackagePipeline`] reference.
    fn from(value: &PackagePipeline) -> Self {
        Self::new(
            match value.package_input.package_info() {
                alpm_pkginfo::PackageInfo::V1(package_info) => package_info.pkgname().clone(),
                alpm_pkginfo::PackageInfo::V2(package_info) => package_info.pkgname().clone(),
            },
            match value.package_input.package_info() {
                alpm_pkginfo::PackageInfo::V1(package_info) => package_info.pkgver().clone(),
                alpm_pkginfo::PackageInfo::V2(package_info) => package_info.pkgver().clone(),
            },
            match value.package_input.package_info() {
                alpm_pkginfo::PackageInfo::V1(package_info) => *package_info.arch(),
                alpm_pkginfo::PackageInfo::V2(package_info) => *package_info.arch(),
            },
            value.compression.clone().into(),
        )
    }
}
