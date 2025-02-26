use std::path::PathBuf;

#[cfg(doc)]
use crate::package::Package;
use crate::{compression::PackageCompression, input::PackageInput};

/// A pipeline for creating a [`Package`] from a [`PackageInput`].
#[derive(Clone, Debug)]
pub struct PackagePipeline {
    pub compression: PackageCompression,
    pub package_input: PackageInput,
    pub output_dir: PathBuf,
}

impl PackagePipeline {
    pub fn new(compression: PackageCompression, input: PackageInput, output_dir: PathBuf) -> Self {
        Self {
            compression,
            package_input: input,
            output_dir,
        }
    }
}
