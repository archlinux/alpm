//! Package build pipeline.

use std::path::PathBuf;

#[cfg(doc)]
use crate::package::Package;
use crate::{compression::PackageCompression, input::PackageInput};

/// A pipeline for creating a [`Package`] from a [`PackageInput`].
#[derive(Clone, Debug)]
pub struct PackagePipeline {
    /// The [`PackageCompression`] that is used when creating a [`Package`].
    pub compression: PackageCompression,

    /// The [`PackageInput`] that is used when creating a [`Package`].
    pub package_input: PackageInput,

    /// The directory in which a [`Package`] is created.
    pub output_dir: PathBuf,
}

impl PackagePipeline {
    /// Creates a new [`PackagePipeline`].
    pub fn new(compression: PackageCompression, input: PackageInput, output_dir: PathBuf) -> Self {
        Self {
            compression,
            package_input: input,
            output_dir,
        }
    }
}
