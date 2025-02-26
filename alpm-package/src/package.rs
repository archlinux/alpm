//! ALPM package representation.

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_common::relative_files;
use alpm_types::{PackageError, PackageFileName};
use tar::Builder;

use crate::{CompressionEncoder, PackageCreationPipeline};

/// An error that can occur when dealing with alpm-package.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error occurred while adding files from an input directory to a package.
    #[error("Error while appending file {from_path} to package archive as {to_path}:\n{source}")]
    AppendFileToArchive {
        /// The path to the file that is appended to the archive as `to_path`.
        from_path: PathBuf,
        /// The path in the archive that `from_path` is appended as.
        to_path: PathBuf,
        /// The source error.
        source: std::io::Error,
    },

    /// An error occurred while finishing an uncompressed package.
    #[error("Error while finishing uncompressed package {package_path}:\n{source}")]
    FinishArchive {
        /// The path of the package file that is being written to
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
    file_name: PackageFileName,
    parent_dir: PathBuf,
}

impl Package {
    /// Creates a new [`Package`].
    ///
    /// # Errors
    ///
    /// Returns an error if no file exists at the path defined by `parent_dir` and `filename`.
    pub fn new(file_name: PackageFileName, parent_dir: PathBuf) -> Result<Self, crate::Error> {
        let file_path = parent_dir.to_path_buf().join(file_name.to_path_buf());
        if !file_path.exists() {
            return Err(crate::Error::PathDoesNotExist { path: file_path });
        }

        Ok(Self {
            file_name,
            parent_dir,
        })
    }

    /// Returns the absolute path of the [`Package`].
    pub fn to_path_buf(&self) -> PathBuf {
        self.parent_dir.join(self.file_name.to_path_buf())
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

/// Appends relative files from an input directory to a [`Builder`].
///
/// # Errors
///
/// Returns an error if
///
/// - retrieving files relative to `input_dir` fails,
/// - or adding one of the relative paths to the `builder` fails.
fn append_relative_files<W>(
    mut builder: Builder<W>,
    input_dir: impl AsRef<Path>,
) -> Result<Builder<W>, crate::Error>
where
    W: Write,
{
    let input_dir = input_dir.as_ref();
    // Get all files relative to the input directory.
    let relative_files = relative_files(input_dir, &[])?;
    // Append all files/directories to the archive.
    for relative_file in relative_files {
        let from_path = input_dir.join(relative_file.as_path());
        let to_path = relative_file;
        builder
            .append_path_with_name(from_path.as_path(), to_path.as_path())
            .map_err(|source| Error::AppendFileToArchive {
                from_path,
                to_path,
                source,
            })?
    }

    Ok(builder)
}

impl TryFrom<PackageCreationPipeline> for Package {
    type Error = crate::Error;

    /// Creates a [`Package`] from a [`PackagePipeline`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - creating a [`PackageFileName`] from `value` fails,
    /// - creating an uncompressed package archive fails,
    /// - appending files to an uncompressed package archive fails,
    /// - or creating a [`Package`] fails.
    fn try_from(value: PackageCreationPipeline) -> Result<Self, Self::Error> {
        let filename = PackageFileName::try_from(&value)?;
        let output_path = value.output_dir.join(filename.to_path_buf());

        // Create the output file.
        let file = File::create(output_path.as_path()).map_err(|source| crate::Error::IoPath {
            path: output_path.clone(),
            context: "creating a package file",
            source,
        })?;

        // If there is compression, create a dedicated compression encoder.
        if let Some(compression) = value.compression {
            // Create an encoder for compression, streaming to a file.
            let encoder = CompressionEncoder::new(file, &compression)?;
            // Create a builder for tar files, streaming to the compression encoder.
            let mut builder = Builder::new(encoder);
            // Do not follow symlinks.
            builder.follow_symlinks(false);
            // Append all files/directories to the archive.
            let builder = append_relative_files(builder, value.package_input.base_dir())?;
            // Finish writing the archive and return the wrapped compression encoder.
            let encoder = builder
                .into_inner()
                .map_err(|source| Error::FinishArchive {
                    package_path: output_path.clone(),
                    source,
                })?;
            // Finish the compression encoder stream.
            encoder.finish()?;
        // If there is no compression, only create a tar file.
        } else {
            // Create a builder for a tar file, streaming to a file directly.
            let mut builder = Builder::new(file);
            // Do not follow symlinks.
            builder.follow_symlinks(false);
            // Append all files/directories to the archive.
            let mut builder = append_relative_files(builder, value.package_input.base_dir())?;
            // Finish writing the archive.
            builder.finish().map_err(|source| Error::FinishArchive {
                package_path: output_path.clone(),
                source,
            })?;
        }

        // Return the package
        Self::new(filename, value.output_dir.to_path_buf())
    }
}
