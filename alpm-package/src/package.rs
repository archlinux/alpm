//! ALPM package representation.

use std::{
    fs::File,
    io::{BufReader, Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_types::{PackageError, PackageFileName};
use bzip2::write::BzEncoder;
use flate2::write::GzEncoder;
use liblzma::write::XzEncoder;
use tar::Builder;
use zstd::Encoder;

use crate::{compression::CompressionSettings, pipeline::PackagePipeline};

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

    #[error("Error while creating a zstandard encoder:\n{source}")]
    CreateEncoder {
        /// The path of the file for which finishing the encoding failed.
        path: PathBuf,
        /// The compression chosen for the encoder.
        compression: CompressionSettings,
        /// The source error.
        source: std::io::Error,
    },

    /// An error occurred when compressing an uncompressed package file.
    #[error(
        "Error while compressing uncompressed package file {uncompressed_path} as {compressed_path} using {compression}:\n{source}"
    )]
    CompressPackage {
        /// The path of the uncompressed package file.
        uncompressed_path: PathBuf,
        /// The path of the compressed package file.
        compressed_path: PathBuf,
        /// The used compression type.
        compression: CompressionSettings,
        /// The source error.
        source: std::io::Error,
    },

    /// An error occurred while finishing an uncompressed package.
    #[error("Error while finishing uncompressed package {package_path}:\n{source}")]
    FinishArchive {
        package_path: PathBuf,
        source: std::io::Error,
    },

    /// An error occurred while finishing a compression encoder.
    #[error("Error while finishing compression encoder {compression} on file {path}:\n{source}")]
    FinishEncoder {
        /// The path of the file for which finishing the encoding failed.
        path: PathBuf,
        /// The compression chosen for the encoder.
        compression: CompressionSettings,
        /// The source error
        source: std::io::Error,
    },

    /// An error occurred while reading an uncompressed package.
    #[error("Error while reading the uncompressed package {package_path}:\n{source}")]
    ReadPackageArchive {
        /// The uncompressed package file that is read.
        package_path: PathBuf,
        /// The source error.
        source: std::io::Error,
    },
}

/// An [alpm-package] file.
///
/// Tracks the [`PackageFileName`] of the [alpm-package] as well as its `parent_dir`.
///
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
#[derive(Clone, Debug)]
pub struct Package {
    filename: PackageFileName,
    parent_dir: PathBuf,
}

impl Package {
    /// Creates a new [`Package`].
    ///
    /// # Errors
    ///
    /// Returns an error if no file exists at the path defined by `parent_path` plus `filename`.
    pub fn new(filename: PackageFileName, parent_dir: PathBuf) -> Result<Self, crate::Error> {
        let file_path = parent_dir.to_path_buf().join(filename.to_path_buf());
        if !file_path.exists() {
            return Err(crate::Error::PathDoesNotExist { path: file_path });
        }

        Ok(Self {
            filename,
            parent_dir,
        })
    }

    /// Returns the absolute path of the [`Package`].
    pub fn to_path_buf(&self) -> PathBuf {
        self.parent_dir.join(self.filename.to_path_buf())
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
            return Err(PackageError::InvalidPackageFileNamePath { path: value }.into());
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

    /// Creates a [`Package`] from a [`PackagePipeline`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - creating an uncompressed package archive fails,
    /// - appending files to an uncompressed package archive fails,
    /// - or creating a [`Package`] fails.
    fn try_from(value: PackagePipeline) -> Result<Self, Self::Error> {
        let filename = PackageFileName::from(&value);

        let uncompressed_filename = filename.to_uncompressed();
        let uncompressed_path = value.output_dir.join(uncompressed_filename.to_path_buf());

        let uncompressed_file = {
            // Add files to uncompressed tar archive
            let uncompressed_file =
                File::create(uncompressed_path.as_path()).map_err(|source| {
                    crate::Error::IoPath {
                        path: uncompressed_path.clone(),
                        context: "creating an uncompressed package file",
                        source,
                    }
                })?;
            let mut builder = Builder::new(uncompressed_file);
            builder.follow_symlinks(false);
            builder
                .append_dir_all(".", value.package_input.base_dir())
                .map_err(|source| Error::AddFilesToArchive {
                    input_dir: value.package_input.base_dir().to_path_buf(),
                    package_path: filename.to_path_buf(),
                    source,
                })?;

            builder.finish().map_err(|source| Error::FinishArchive {
                package_path: uncompressed_path.clone(),
                source,
            })?;

            File::open(uncompressed_path.as_path()).map_err(|source| crate::Error::IoPath {
                path: uncompressed_path.clone(),
                context: "opening an uncompressed package file",
                source,
            })?
        };

        let mut uncompressed_reader = BufReader::new(uncompressed_file);

        let compressed_path = value.output_dir.join(filename.to_path_buf());
        let file =
            File::create(compressed_path.as_path()).map_err(|source| crate::Error::IoPath {
                path: compressed_path.clone(),
                context: "creating a compressed package file",
                source,
            })?;

        match value.compression {
            CompressionSettings::Bzip2 { compression_level } => {
                let mut writer = BzEncoder::new(file, compression_level.clone().into());
                let mut buffer = [0; 1024];

                loop {
                    let count = uncompressed_reader.read(&mut buffer).map_err(|source| {
                        Error::ReadPackageArchive {
                            package_path: uncompressed_path.clone(),
                            source,
                        }
                    })?;
                    if count == 0 {
                        break;
                    }
                    writer
                        .write(&buffer)
                        .map_err(|source| Error::CompressPackage {
                            uncompressed_path: uncompressed_path.clone(),
                            compressed_path: compressed_path.clone(),
                            compression: CompressionSettings::Bzip2 {
                                compression_level: compression_level.clone(),
                            },
                            source,
                        })?;
                }
                writer.finish().map_err(|source| Error::FinishEncoder {
                    path: filename.to_path_buf(),
                    compression: CompressionSettings::Bzip2 {
                        compression_level: compression_level.clone(),
                    },
                    source,
                })?;
                Package::new(filename, value.output_dir.to_path_buf())
            }
            CompressionSettings::Gzip { compression_level } => {
                let mut writer = GzEncoder::new(file, compression_level.clone().into());
                let mut buffer = [0; 1024];

                loop {
                    let count = uncompressed_reader.read(&mut buffer).map_err(|source| {
                        Error::ReadPackageArchive {
                            package_path: uncompressed_path.clone(),
                            source,
                        }
                    })?;
                    if count == 0 {
                        break;
                    }
                    writer
                        .write(&buffer)
                        .map_err(|source| Error::CompressPackage {
                            uncompressed_path: uncompressed_path.clone(),
                            compressed_path: compressed_path.clone(),
                            compression: CompressionSettings::Gzip {
                                compression_level: compression_level.clone(),
                            },
                            source,
                        })?;
                }
                writer.finish().map_err(|source| Error::FinishEncoder {
                    path: filename.to_path_buf(),
                    compression: CompressionSettings::Gzip {
                        compression_level: compression_level.clone(),
                    },
                    source,
                })?;
                Package::new(filename, value.output_dir.to_path_buf())
            }
            CompressionSettings::Xz { compression_level } => {
                let mut writer = XzEncoder::new(file, compression_level.clone().into());
                let mut buffer = [0; 1024];

                loop {
                    let count = uncompressed_reader.read(&mut buffer).map_err(|source| {
                        Error::ReadPackageArchive {
                            package_path: uncompressed_path.clone(),
                            source,
                        }
                    })?;
                    if count == 0 {
                        break;
                    }
                    writer
                        .write(&buffer)
                        .map_err(|source| Error::CompressPackage {
                            uncompressed_path: uncompressed_path.clone(),
                            compressed_path: compressed_path.clone(),
                            compression: CompressionSettings::Xz {
                                compression_level: compression_level.clone(),
                            },
                            source,
                        })?;
                }
                writer.finish().map_err(|source| Error::FinishEncoder {
                    path: filename.to_path_buf(),
                    compression: CompressionSettings::Xz {
                        compression_level: compression_level.clone(),
                    },
                    source,
                })?;
                Package::new(filename, value.output_dir.to_path_buf())
            }
            CompressionSettings::Zstandard { compression_level } => {
                let mut writer =
                    Encoder::new(file, compression_level.clone().into()).map_err(|source| {
                        Error::CreateEncoder {
                            path: filename.to_path_buf(),
                            compression: CompressionSettings::Zstandard {
                                compression_level: compression_level.clone(),
                            },
                            source,
                        }
                    })?;
                let mut buffer = [0; 1024];

                loop {
                    let count = uncompressed_reader.read(&mut buffer).map_err(|source| {
                        Error::ReadPackageArchive {
                            package_path: uncompressed_path.clone(),
                            source,
                        }
                    })?;
                    if count == 0 {
                        break;
                    }
                    writer
                        .write(&buffer)
                        .map_err(|source| Error::CompressPackage {
                            uncompressed_path: uncompressed_path.clone(),
                            compressed_path: compressed_path.clone(),
                            compression: CompressionSettings::Zstandard {
                                compression_level: compression_level.clone(),
                            },
                            source,
                        })?;
                }

                writer.finish().map_err(|source| Error::FinishEncoder {
                    path: filename.to_path_buf(),
                    compression: CompressionSettings::Zstandard {
                        compression_level: compression_level.clone(),
                    },
                    source,
                })?;

                Package::new(filename, value.output_dir.to_path_buf())
            }
            CompressionSettings::None => Package::new(filename, value.output_dir.to_path_buf()),
        }
    }
}
