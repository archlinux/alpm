use std::{
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

use tar::Builder;

/// An error that can occur when dealing with alpm-package.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error occurred while adding files from an input directory to a package.
    #[error(
        "Error while adding files from input directory {input_dir} to package {package_path}:\n{source}"
    )]
    AddFilesToArchive {
        input_dir: PathBuf,
        package_path: PathBuf,
        source: std::io::Error,
    },
}

use crate::{
    compression::PackageCompression,
    filename::PackageFileName,
    pipeline::PackagePipeline,
};

/// An [alpm-package] file.
///
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
#[derive(Clone, Debug)]
pub struct Package {
    filename: PackageFileName,
    parent_path: PathBuf,
}

impl Package {
    /// Creates a new [`Package`].
    pub fn new(filename: PackageFileName, parent_path: PathBuf) -> Result<Self, crate::Error> {
        let file_path = parent_path.to_path_buf().join(filename.to_path_buf());
        if !file_path.exists() {
            return Err(crate::Error::PathDoesNotExist { path: file_path });
        }

        Ok(Self {
            filename,
            parent_path,
        })
    }

    pub fn to_path_buf(&self) -> PathBuf {
        self.parent_path.join(self.filename.to_path_buf())
    }
}

impl TryFrom<&Path> for Package {
    type Error = crate::Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        Self::try_from(value.to_path_buf())
    }
}

impl TryFrom<PathBuf> for Package {
    type Error = crate::Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let Some(Some(filename)) = value.file_name().map(|name_os| name_os.to_str()) else {
            return Err(crate::filename::Error::InvalidPath { path: value }.into());
        };
        let Some(parent_dir) = value.parent() else {
            return Err(crate::Error::PathNoParent { path: value });
        };

        Self::new(
            PackageFileName::from_str(filename)?,
            parent_dir.to_path_buf(),
        )
    }
}

impl TryFrom<PackagePipeline> for Package {
    type Error = crate::Error;

    fn try_from(value: PackagePipeline) -> Result<Self, Self::Error> {
        let filename = PackageFileName::from(&value);
        let full_path = value.output_dir.join(filename.to_path_buf());

        // Add files to uncompressed tar archive
        let file = File::create(full_path.as_path()).map_err(|source| crate::Error::IoPath {
            path: full_path,
            context: "creating an uncompressed package file",
            source,
        })?;
        let mut builder = Builder::new(file);
        builder.follow_symlinks(false);
        builder
            .append_dir_all(".", value.package_input.get_base_dir())
            .map_err(|source| Error::AddFilesToArchive {
                input_dir: value.package_input.get_base_dir().to_path_buf(),
                package_path: filename.to_path_buf(),
                source,
            })?;

        match value.compression {
            PackageCompression::None => Package::new(filename, value.output_dir.to_path_buf()),
            _ => unimplemented!(),
        }
    }
}
