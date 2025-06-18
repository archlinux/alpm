//! Representation of [alpm-package] files.
//!
//! [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html

use std::{
    fs::{File, create_dir_all},
    io::{Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_buildinfo::BuildInfo;
use alpm_common::{InputPaths, MetadataFile};
use alpm_mtree::Mtree;
use alpm_pkginfo::PackageInfo;
use alpm_types::{
    CompressionAlgorithmFileExtension,
    MetadataFileName,
    PackageError,
    PackageFileName,
};
use log::debug;
use tar::{Archive, Builder};

use crate::{
    CompressionAlgorithm,
    CompressionEncoder,
    OutputDir,
    PackageCreationConfig,
    compression::CompressionDecoder,
};

/// An error that can occur when handling [alpm-package] files.
///
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
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
    #[error("Error while finishing the creation of uncompressed package {package_path}:\n{source}")]
    FinishArchive {
        /// The path of the package file that is being written to
        package_path: PathBuf,
        /// The source error.
        source: std::io::Error,
    },
}

/// A path that is guaranteed to be an existing absolute directory.
#[derive(Clone, Debug)]
pub struct ExistingAbsoluteDir(PathBuf);

impl ExistingAbsoluteDir {
    /// Creates a new [`ExistingAbsoluteDir`] from `path`.
    ///
    /// Creates a directory at `path` if it does not exist yet.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - `path` is not absolute,
    /// - `path` does not exist and cannot be created,
    /// - the metadata of `path` cannot be retrieved,
    /// - or `path` is not a directory.
    pub fn new(path: PathBuf) -> Result<Self, crate::Error> {
        if !path.is_absolute() {
            return Err(alpm_common::Error::NonAbsolutePaths {
                paths: vec![path.clone()],
            }
            .into());
        }

        if !path.exists() {
            create_dir_all(&path).map_err(|source| crate::Error::IoPath {
                path: path.clone(),
                context: "creating absolute directory",
                source,
            })?;
        }

        let metadata = path.metadata().map_err(|source| crate::Error::IoPath {
            path: path.clone(),
            context: "retrieving metadata",
            source,
        })?;

        if !metadata.is_dir() {
            return Err(alpm_common::Error::NotADirectory { path: path.clone() }.into());
        }

        Ok(Self(path))
    }

    /// Coerces to a Path slice.
    ///
    /// Delegates to [`PathBuf::as_path`].
    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }

    /// Converts a Path to an owned PathBuf.
    ///
    /// Delegates to [`Path::to_path_buf`].
    pub fn to_path_buf(&self) -> PathBuf {
        self.0.to_path_buf()
    }

    /// Creates an owned PathBuf with path adjoined to self.
    ///
    /// Delegates to [`Path::join`].
    pub fn join(&self, path: impl AsRef<Path>) -> PathBuf {
        self.0.join(path)
    }
}

impl AsRef<Path> for ExistingAbsoluteDir {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl From<&OutputDir> for ExistingAbsoluteDir {
    /// Creates an [`ExistingAbsoluteDir`] from an [`OutputDir`].
    ///
    /// As [`OutputDir`] provides a more strict set of requirements, this can be infallible.
    fn from(value: &OutputDir) -> Self {
        Self(value.to_path_buf())
    }
}

impl TryFrom<&Path> for ExistingAbsoluteDir {
    type Error = crate::Error;

    /// Creates an [`ExistingAbsoluteDir`] from a [`Path`] reference.
    ///
    /// Delegates to [`ExistingAbsoluteDir::new`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`ExistingAbsoluteDir::new`] fails.
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        Self::new(value.to_path_buf())
    }
}

/// Appends relative files from an input directory to a [`Builder`].
///
/// Before appending any files, all provided `input_paths` are validated against `mtree` (ALPM-MTREE
/// data).
///
/// # Errors
///
/// Returns an error if
///
/// - validating any path in `input_paths` using `mtree` fails,
/// - retrieving files relative to `input_dir` fails,
/// - or adding one of the relative paths to the `builder` fails.
fn append_relative_files<W>(
    mut builder: Builder<W>,
    mtree: &Mtree,
    input_paths: &InputPaths,
) -> Result<Builder<W>, crate::Error>
where
    W: Write,
{
    // Validate all paths using the ALPM-MTREE data before appending them to the builder.
    let mtree_path = PathBuf::from(MetadataFileName::Mtree.as_ref());
    let check_paths = {
        let all_paths = input_paths.paths();
        // If there is an ALPM-MTREE file, exclude it from the validation, as the ALPM-MTREE data
        // does not cover it.
        if let Some(mtree_position) = all_paths.iter().position(|path| path == &mtree_path) {
            let before = &all_paths[..mtree_position];
            let after = if all_paths.len() > mtree_position {
                &all_paths[mtree_position + 1..]
            } else {
                &[]
            };
            &[before, after].concat()
        } else {
            all_paths
        }
    };
    mtree.validate_paths(&InputPaths::new(input_paths.base_dir(), check_paths)?)?;

    // Append all files/directories to the archive.
    for relative_file in input_paths.paths() {
        let from_path = input_paths.base_dir().join(relative_file.as_path());
        builder
            .append_path_with_name(from_path.as_path(), relative_file.as_path())
            .map_err(|source| Error::AppendFileToArchive {
                from_path,
                to_path: relative_file.clone(),
                source,
            })?
    }

    Ok(builder)
}

/// A reader for [`Package`] files.
///
/// It implements the [`Read`] trait, allowing reading the contents of a package file
/// as a stream of bytes.
///
/// A [`PackageReader`] can be created from a [`Package`] using the
/// [`Package::into_reader`] or [`PackageReader::try_from`] methods.
#[derive(Debug)]
pub struct PackageReader<'a> {
    decoder: CompressionDecoder<'a>,
}

impl<'a> PackageReader<'a> {
    /// Creates a new [`PackageReader`] from a [`CompressionDecoder`].
    ///
    /// # Errors
    ///
    /// Returns an error if the `decoder` cannot be created from the file and its extension.
    pub fn new(decoder: CompressionDecoder<'a>) -> Result<Self, crate::Error> {
        Ok(Self { decoder })
    }

    /// Returns a [`Archive`] from the [`PackageReader`].
    pub fn into_tar(self) -> Archive<CompressionDecoder<'a>> {
        Archive::new(self.decoder)
    }

    /// Reads a metadata file from the package archive.
    ///
    /// Returns the contents of the metadata file as a `Vec<u8>`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the metadata does not exist in the package archive,
    /// - reading the package archive entries fails,
    /// - reading a package archive entry fails,
    /// - reading the contents of a package archive entry fails,
    /// - or the path of the entry cannot be retrieved.
    pub fn read_metadata_file(mut self, name: MetadataFileName) -> Result<Vec<u8>, crate::Error> {
        let mut archive = Archive::new(&mut self.decoder);
        let entries = archive.entries().map_err(|source| crate::Error::IoRead {
            context: "reading package archive entries",
            source,
        })?;
        for entry in entries {
            let mut entry = entry.map_err(|source| crate::Error::IoRead {
                context: "reading package archive entry",
                source,
            })?;
            if let Ok(path) = entry.path() {
                if path.to_string_lossy() == name.to_string() {
                    let path = path.to_path_buf();
                    let mut buffer = Vec::new();
                    entry
                        .read_to_end(&mut buffer)
                        .map_err(|source| crate::Error::IoPath {
                            path,
                            context: "reading package archive entries",
                            source,
                        })?;
                    return Ok(buffer);
                }
            }
        }
        Err(crate::Error::MetadataFileNotFound { name })
    }
}

impl<'a> TryFrom<Package> for PackageReader<'a> {
    type Error = crate::Error;

    /// Creates a [`PackageReader`] from a [`Package`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the package file cannot be opened,
    /// - the package file extension cannot be determined,
    /// - or the compression decoder cannot be created from the file and its extension.
    fn try_from(package: Package) -> Result<Self, Self::Error> {
        let path = package.to_path_buf();
        let file = File::open(&path).map_err(|source| crate::Error::IoPath {
            path: path.clone(),
            context: "opening package file",
            source,
        })?;
        let extension = CompressionAlgorithmFileExtension::try_from(path.as_path())?;
        let algorithm = CompressionAlgorithm::try_from(extension)?;
        let decoder = CompressionDecoder::new(file, algorithm)?;
        Ok(Self { decoder })
    }
}

impl<'a> Read for PackageReader<'a> {
    /// Reads the contents of the package file into `buf`.
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.decoder.read(buf)
    }
}

/// An [alpm-package] file.
///
/// Tracks the [`PackageFileName`] of the [alpm-package] as well as its absolute `parent_dir`.
///
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
#[derive(Clone, Debug)]
pub struct Package {
    file_name: PackageFileName,
    parent_dir: ExistingAbsoluteDir,
}

impl Package {
    /// Creates a new [`Package`].
    ///
    /// # Errors
    ///
    /// Returns an error if no file exists at the path defined by `parent_dir` and `filename`.
    pub fn new(
        file_name: PackageFileName,
        parent_dir: ExistingAbsoluteDir,
    ) -> Result<Self, crate::Error> {
        let file_path = parent_dir.to_path_buf().join(file_name.to_path_buf());
        if !file_path.exists() {
            return Err(crate::Error::PathDoesNotExist { path: file_path });
        }
        if !file_path.is_file() {
            return Err(crate::Error::PathIsNotAFile { path: file_path });
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

    /// Creates a [`PackageInfo`] from the contents of the [PKGINFO] file in the package.
    ///
    /// This is a convenience wrapper around [`PackageReader::read_metadata_file`]
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - a [`PackageReader`] cannot be created for the package,
    /// - the package does not contain a [PKGINFO] file,
    /// - or the [PKGINFO] file in the package is not valid.
    ///
    /// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
    pub fn read_pkginfo(&self) -> Result<PackageInfo, crate::Error> {
        let reader = PackageReader::try_from(self.clone())?;
        let data = reader.read_metadata_file(MetadataFileName::PackageInfo)?;
        let pkginfo = PackageInfo::from_reader(&*data)?;
        Ok(pkginfo)
    }

    /// Creates a [`BuildInfo`] from the contents of the [BUILDINFO] file in the package.
    ///
    /// This is a convenience wrapper around [`PackageReader::read_metadata_file`]
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - a [`PackageReader`] cannot be created for the package,
    /// - the package does not contain a [BUILDINFO] file,
    /// - or the [BUILDINFO] file in the package is not valid.
    ///
    /// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
    pub fn read_buildinfo(&self) -> Result<BuildInfo, crate::Error> {
        let reader = PackageReader::try_from(self.clone())?;
        let data = reader.read_metadata_file(MetadataFileName::BuildInfo)?;
        let buildinfo = BuildInfo::from_reader(&*data)?;
        Ok(buildinfo)
    }

    /// Creates an [`Mtree`] from the contents of the [ALPM-MTREE] file in the package.
    ///
    /// This is a convenience wrapper around [`PackageReader::read_metadata_file`]
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - a [`PackageReader`] cannot be created for the package,
    /// - the package does not contain a [ALPM-MTREE] file,
    /// - or the [ALPM-MTREE] file in the package is not valid.
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    pub fn read_mtree(&self) -> Result<Mtree, crate::Error> {
        let reader = PackageReader::try_from(self.clone())?;
        let data = reader.read_metadata_file(MetadataFileName::Mtree)?;
        let mtree = Mtree::from_reader(&*data)?;
        Ok(mtree)
    }

    /// Creates a [`PackageReader`] for the package.
    ///
    /// Convenience wrapper for [`PackageReader::try_from`].
    ///
    /// # Errors
    ///
    /// Returns an error if `self` cannot be converted into a [`PackageReader`].
    pub fn into_reader<'a>(self) -> Result<PackageReader<'a>, crate::Error> {
        PackageReader::try_from(self)
    }
}

impl TryFrom<&Path> for Package {
    type Error = crate::Error;

    /// Creates a [`Package`] from a [`Path`] reference.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - no file name can be retrieved from `path`,
    /// - `value` has no parent directory,
    /// - or [`Package::new`] fails.
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        debug!("Attempt to create a package representation from path {value:?}");
        let Some(parent_dir) = value.parent() else {
            return Err(crate::Error::PathHasNoParent {
                path: value.to_path_buf(),
            });
        };
        let Some(filename) = value.file_name().and_then(|name| name.to_str()) else {
            return Err(PackageError::InvalidPackageFileNamePath {
                path: value.to_path_buf(),
            }
            .into());
        };

        Self::new(PackageFileName::from_str(filename)?, parent_dir.try_into()?)
    }
}

impl TryFrom<&PackageCreationConfig> for Package {
    type Error = crate::Error;

    /// Creates a new [`Package`] from a [`PackageCreationConfig`].
    ///
    /// Before creating a [`Package`], guarantees the on-disk file consistency with the
    /// help of available [`Mtree`] data.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - creating a [`PackageFileName`] from `value` fails,
    /// - creating a [`CompressionEncoder`] fails,
    /// - creating a compressed or uncompressed package file fails,
    /// - validating any of the paths using ALPM-MTREE data (available through `value`) fails,
    /// - appending files to a compressed or uncompressed package file fails,
    /// - finishing a compressed or uncompressed package file fails,
    /// - or creating a [`Package`] fails.
    fn try_from(value: &PackageCreationConfig) -> Result<Self, Self::Error> {
        let filename = PackageFileName::try_from(value)?;
        let parent_dir: ExistingAbsoluteDir = value.output_dir().into();
        let output_path = value.output_dir().join(filename.to_path_buf());

        // Create the output file.
        let file = File::create(output_path.as_path()).map_err(|source| crate::Error::IoPath {
            path: output_path.clone(),
            context: "creating a package file",
            source,
        })?;

        // If compression is requested, create a dedicated compression encoder streaming to a file
        // and a tar builder that streams to the compression encoder.
        // Append all files and directories to it, then finish the tar builder and the compression
        // encoder streams.
        if let Some(compression) = value.compression() {
            let encoder = CompressionEncoder::new(file, compression)?;
            let mut builder = Builder::new(encoder);
            // We do not want to follow symlinks but instead archive symlinks!
            builder.follow_symlinks(false);
            let builder = append_relative_files(
                builder,
                value.package_input().mtree()?,
                &value.package_input().input_paths()?,
            )?;
            let encoder = builder
                .into_inner()
                .map_err(|source| Error::FinishArchive {
                    package_path: output_path.clone(),
                    source,
                })?;
            encoder.finish()?;
        // If no compression is requested, only create a tar builder.
        // Append all files and directories to it, then finish the tar builder stream.
        } else {
            let mut builder = Builder::new(file);
            // We do not want to follow symlinks but instead archive symlinks!
            builder.follow_symlinks(false);
            let mut builder = append_relative_files(
                builder,
                value.package_input().mtree()?,
                &value.package_input().input_paths()?,
            )?;
            builder.finish().map_err(|source| Error::FinishArchive {
                package_path: output_path.clone(),
                source,
            })?;
        }

        Self::new(filename, parent_dir)
    }
}

impl Read for Package {
    /// Reads the contents of the package file into `buf`.
    ///
    /// # Note
    ///
    /// It is recommended to use [`PackageReader`] instead of this method, as this method
    /// creates a new [`PackageReader`] each time it is called, which is inefficient.
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut reader = self.clone().into_reader().map_err(|source| {
            std::io::Error::other(format!("Failed to create package reader: {source}"))
        })?;
        reader.read(buf)
    }
}

#[cfg(test)]
mod tests {

    use std::fs::create_dir;

    use log::{LevelFilter, debug};
    use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
    use tempfile::{NamedTempFile, TempDir};
    use testresult::TestResult;

    use super::*;

    /// Initializes a global [`TermLogger`].
    fn init_logger() {
        if TermLogger::init(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        )
        .is_err()
        {
            debug!("Not initializing another logger, as one is initialized already.");
        }
    }

    /// Ensures that [`ExistingAbsoluteDir::new`] creates non-existing, absolute paths.
    #[test]
    fn absolute_dir_new_creates_dir() -> TestResult {
        init_logger();

        let temp_dir = TempDir::new()?;
        let path = temp_dir.path().join("additional");

        if let Err(error) = ExistingAbsoluteDir::new(path) {
            return Err(format!("Failed although it should have succeeded: {error}").into());
        }

        Ok(())
    }

    /// Ensures that [`ExistingAbsoluteDir::new`] fails on non-absolute paths and those representing
    /// a file.
    #[test]
    fn absolute_dir_new_fails() -> TestResult {
        init_logger();

        if let Err(error) = ExistingAbsoluteDir::new(PathBuf::from("test")) {
            assert!(matches!(
                error,
                crate::Error::AlpmCommon(alpm_common::Error::NonAbsolutePaths { paths: _ })
            ));
        } else {
            return Err("Succeeded although it should have failed".into());
        }

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();
        if let Err(error) = ExistingAbsoluteDir::new(path.to_path_buf()) {
            assert!(matches!(
                error,
                crate::Error::AlpmCommon(alpm_common::Error::NotADirectory { path: _ })
            ));
        } else {
            return Err("Succeeded although it should have failed".into());
        }

        Ok(())
    }

    /// Ensures that utility methods of [`ExistingAbsoluteDir`] are functional.
    #[test]
    fn absolute_dir_utilities() -> TestResult {
        let temp_dir = TempDir::new()?;
        let path = temp_dir.path();

        // Create from &Path
        let absolute_dir: ExistingAbsoluteDir = path.try_into()?;

        assert_eq!(absolute_dir.as_path(), path);
        assert_eq!(absolute_dir.as_ref(), path);

        Ok(())
    }

    /// Ensure that [`Package::new`] can succeeds.
    #[test]
    fn package_new() -> TestResult {
        let temp_dir = TempDir::new()?;
        let path = temp_dir.path();
        let absolute_dir = ExistingAbsoluteDir::new(path.to_path_buf())?;
        let package_name = "example-1.0.0-1-x86_64.pkg.tar.zst";
        File::create(absolute_dir.join(package_name))?;

        let Ok(_package) = Package::new(package_name.parse()?, absolute_dir.clone()) else {
            return Err("Failed although it should have succeeded".into());
        };

        Ok(())
    }

    /// Ensure that [`Package::new`] fails on a non-existent file and on paths that are not a file.
    #[test]
    fn package_new_fails() -> TestResult {
        let temp_dir = TempDir::new()?;
        let path = temp_dir.path();
        let absolute_dir = ExistingAbsoluteDir::new(path.to_path_buf())?;
        let package_name = "example-1.0.0-1-x86_64.pkg.tar.zst";

        // The file does not exist.
        if let Err(error) = Package::new(package_name.parse()?, absolute_dir.clone()) {
            assert!(matches!(error, crate::Error::PathDoesNotExist { path: _ }))
        } else {
            return Err("Succeeded although it should have failed".into());
        }

        // The file is a directory.
        create_dir(absolute_dir.join(package_name))?;
        if let Err(error) = Package::new(package_name.parse()?, absolute_dir.clone()) {
            assert!(matches!(error, crate::Error::PathIsNotAFile { path: _ }))
        } else {
            return Err("Succeeded although it should have failed".into());
        }

        Ok(())
    }

    /// Ensure that [`Package::try_from`] fails on paths not providing a file name and paths not
    /// providing a parent directory.
    #[test]
    fn package_try_from_path_fails() -> TestResult {
        init_logger();

        // Fail on trying to use a directory without a file name as a package.
        assert!(Package::try_from(PathBuf::from("/").as_path()).is_err());

        // Fail on trying to use a file without a parent
        assert!(
            Package::try_from(
                PathBuf::from("/something_very_unlikely_to_ever_exist_in_a_filesystem").as_path()
            )
            .is_err()
        );

        Ok(())
    }
}
