//! Representation of [alpm-package] files.
//!
//! [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html

use std::{
    fmt::{self, Debug},
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
    INSTALL_SCRIPTLET_FILE_NAME,
    MetadataFileName,
    PackageError,
    PackageFileName,
};
use log::debug;
use tar::{Archive, Builder, Entries, Entry};

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

/// Metadata entry contained in a package archive.
///
/// This is being used in [`PackageReader::metadata_entries`] to iterate over available
/// metadata files.
#[derive(Debug)]
pub enum MetadataEntry {
    /// The [PKGINFO] file in the package.
    ///
    /// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
    PackageInfo(PackageInfo),
    /// The [BUILDINFO] file in the package.
    ///
    /// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
    BuildInfo(BuildInfo),
    /// The [ALPM-MTREE] file in the package.
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    Mtree(Mtree),
    /// An [alpm-install-scriptlet] file in the package.
    ///
    /// [alpm-install-scriptlet]:
    /// https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    InstallScriptlet(String),
}

/// All the metadata contained in an [alpm-package] file.
///
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
#[derive(Clone, Debug)]
pub struct Metadata {
    /// The [PKGINFO] file in the package.
    ///
    /// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
    pub pkginfo: PackageInfo,
    /// The [BUILDINFO] file in the package.
    ///
    /// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
    pub buildinfo: BuildInfo,
    /// The [ALPM-MTREE] file in the package.
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    pub mtree: Mtree,
    /// The [alpm-install-scriptlet] file in the package, if present.
    ///
    /// [alpm-install-scriptlet]:
    /// https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    pub install_scriptlet: Option<String>,
}

/// Data entry contained in a package archive.
///
/// This is being used in [`PackageReader::metadata_entries`] to iterate over available
/// data files in a package archive.
///
/// It implements [`Read`] to allow reading the contents of the data entry directly.
pub struct DataEntry<'r, 'a> {
    /// The path of the data entry in the package archive.
    path: PathBuf,
    /// The contents of the data entry.
    entry: Entry<'r, CompressionDecoder<'a>>,
}

impl Debug for DataEntry<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataEntry")
            .field("path", &self.path)
            .field("entry", &"tar::Entry<CompressionDecoder>")
            .finish()
    }
}

impl<'r, 'a> DataEntry<'r, 'a> {
    /// Returns the path of the data entry in the package archive.
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    /// Returns the contents of the data entry.
    ///
    /// # Errors
    ///
    /// Returns an error if [`tar::Entry::read_to_end`] fails.
    pub fn contents(&mut self) -> Result<Vec<u8>, crate::Error> {
        let mut buffer = Vec::new();
        self.entry
            .read_to_end(&mut buffer)
            .map_err(|source| crate::Error::IoRead {
                context: "reading package archive entry contents",
                source,
            })?;
        Ok(buffer)
    }

    /// Returns the raw [`tar::Entry`] of the data entry.
    ///
    /// This is useful for accessing metadata of the entry, such as its header or path.
    pub fn entry(&'r self) -> &'r Entry<'r, CompressionDecoder<'a>> {
        &self.entry
    }
}

impl Read for DataEntry<'_, '_> {
    /// Reads data from the entry into the provided buffer.
    ///
    /// Delegates to [`tar::Entry::read`].
    ///
    /// # Errors
    ///
    /// Returns an error if reading from the entry fails.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        self.entry.read(buf)
    }
}

/// A reader for [`Package`] files.
///
/// A [`PackageReader`] can be created from a [`Package`] using the
/// [`Package::into_reader`] or [`PackageReader::try_from`] methods.
pub struct PackageReader<'a> {
    archive: Archive<CompressionDecoder<'a>>,
}

impl<'a> Debug for PackageReader<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PackageReader")
            .field("archive", &"Archive<CompressionDecoder>")
            .finish()
    }
}

impl<'a> PackageReader<'a> {
    /// Creates a new [`PackageReader`] from a [`Archive<CompressionDecoder>`].
    pub fn new(archive: Archive<CompressionDecoder<'a>>) -> Self {
        Self { archive }
    }

    /// Returns an iterator over the entries in the package archive.
    ///
    /// This iterator yields [`tar::Entry`]s, which can be used to read the contents of the
    /// entries.
    ///
    /// # Errors
    ///
    /// Returns an error if the entries cannot be read from the archive.
    pub fn entries(&mut self) -> Result<Entries<'_, CompressionDecoder<'a>>, crate::Error> {
        self.archive
            .entries()
            .map_err(|source| crate::Error::IoRead {
                context: "reading package archive entries",
                source,
            })
    }

    /// Returns an iterator over the data entries in the package archive.
    ///
    /// This iterator yields [`DataEntry`]s, which contain the path and contents of the data files
    ///
    /// # Notes
    ///
    /// This iterator filters out:
    ///
    /// - directories,
    /// - known metadata files such as `.PKGINFO`, `BUILDINFO`, `ALPM-MTREE`, and the install
    ///   scriptlet file,
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - reading the package archive entries fails,
    /// - reading a package archive entry fails,
    /// - reading the contents of a package archive entry fails,
    /// - or retrieving the path of a package archive entry fails.
    pub fn data_entries<'r>(
        &'r mut self,
    ) -> Result<impl Iterator<Item = Result<DataEntry<'r, 'a>, crate::Error>>, crate::Error> {
        let non_data_file_names = [
            MetadataFileName::PackageInfo.as_ref(),
            MetadataFileName::BuildInfo.as_ref(),
            MetadataFileName::Mtree.as_ref(),
            INSTALL_SCRIPTLET_FILE_NAME,
        ];
        let entries = self.entries()?;
        Ok(entries.filter_map(move |entry| {
            let filter = (|| {
                let entry = entry.map_err(|source| crate::Error::IoRead {
                    context: "reading package archive entry",
                    source,
                })?;
                // Filter out directories
                if entry.header().entry_type() == tar::EntryType::Directory {
                    return Ok(None);
                }
                let path = entry.path().map_err(|source| crate::Error::IoRead {
                    context: "retrieving path of package archive entry",
                    source,
                })?;
                // Filter out known metadata files
                if non_data_file_names.contains(&path.to_string_lossy().as_ref()) {
                    return Ok(None);
                }
                Ok(Some(DataEntry {
                    path: path.to_path_buf(),
                    entry,
                }))
            })();
            match filter {
                Ok(Some(entry)) => Some(Ok(entry)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            }
        }))
    }

    /// Returns an iterator over the metadata entries in the package archive.
    ///
    /// This iterator yields [`MetadataEntry`]s, which can be either a [`PackageInfo`],
    /// [`BuildInfo`], or [`Mtree`].
    ///
    /// The iterator stops when it encounters an entry that does not match any
    /// known metadata file names.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - reading the package archive entries fails,
    /// - reading a package archive entry fails,
    /// - reading the contents of a package archive entry fails,
    /// - retrieving the path of a package archive entry fails,
    /// - or if the path of a package archive entry does not match any known metadata file names.
    pub fn metadata_entries(
        &mut self,
    ) -> Result<impl Iterator<Item = Result<MetadataEntry, crate::Error>>, crate::Error> {
        let entries = self.entries()?;
        Ok(entries.map(|entry| {
            let mut entry = entry.map_err(|source| crate::Error::IoRead {
                context: "reading package archive entry",
                source,
            })?;
            let mut buffer = Vec::new();
            entry
                .read_to_end(&mut buffer)
                .map_err(|source| crate::Error::IoRead {
                    context: "reading package archive entry contents",
                    source,
                })?;
            let path = entry.path().map_err(|source| crate::Error::IoRead {
                context: "retrieving path of package archive entry",
                source,
            })?;
            let path = path.to_string_lossy();
            if path == MetadataFileName::PackageInfo.as_ref() {
                PackageInfo::from_reader(&*buffer)
                    .map(MetadataEntry::PackageInfo)
                    .map_err(crate::Error::from)
            } else if path == MetadataFileName::BuildInfo.as_ref() {
                BuildInfo::from_reader(&*buffer)
                    .map(MetadataEntry::BuildInfo)
                    .map_err(crate::Error::from)
            } else if path == MetadataFileName::Mtree.as_ref() {
                Mtree::from_reader(&*buffer)
                    .map(MetadataEntry::Mtree)
                    .map_err(crate::Error::from)
            } else if path == INSTALL_SCRIPTLET_FILE_NAME {
                let scriptlet =
                    String::from_utf8(buffer).map_err(|source| crate::Error::InvalidUTF8 {
                        context: "reading install scriptlet",
                        source,
                    })?;
                Ok(MetadataEntry::InstallScriptlet(scriptlet))
            } else {
                Err(crate::Error::MetadataFileEndOfFiles)
            }
        }))
    }

    //// Reads the metadata from the package archive.
    ///
    /// This method reads all the metadata entries in the package archive and returns a
    /// [`Metadata`] struct containing the parsed metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - reading the metadata entries fails,
    /// - parsing a metadata entry fails,
    /// - or if any of the required metadata files are not found in the package.
    pub fn metadata(&mut self) -> Result<Metadata, crate::Error> {
        let mut pkginfo = None;
        let mut buildinfo = None;
        let mut mtree = None;
        let mut scriptlet = None;
        for entry in self.metadata_entries()? {
            match entry? {
                MetadataEntry::PackageInfo(m) => pkginfo = Some(m),
                MetadataEntry::BuildInfo(m) => buildinfo = Some(m),
                MetadataEntry::Mtree(m) => mtree = Some(m),
                MetadataEntry::InstallScriptlet(s) => {
                    scriptlet = Some(s);
                }
            }
        }
        Ok(Metadata {
            pkginfo: pkginfo.ok_or(crate::Error::MetadataFileNotFound {
                name: MetadataFileName::PackageInfo,
            })?,
            buildinfo: buildinfo.ok_or(crate::Error::MetadataFileNotFound {
                name: MetadataFileName::BuildInfo,
            })?,
            mtree: mtree.ok_or(crate::Error::MetadataFileNotFound {
                name: MetadataFileName::Mtree,
            })?,
            install_scriptlet: scriptlet,
        })
    }

    /// Reads a specific metadata file from the package.
    ///
    /// This method searches for a metadata file that matches the provided
    /// [`MetadataFileName`] and returns the corresponding [`MetadataEntry`].
    pub fn read_metadata_file(
        &mut self,
        file_name: MetadataFileName,
    ) -> Result<MetadataEntry, crate::Error> {
        for entry in self.metadata_entries()? {
            let entry = entry?;
            match (&entry, &file_name) {
                (MetadataEntry::PackageInfo(_), MetadataFileName::PackageInfo)
                | (MetadataEntry::BuildInfo(_), MetadataFileName::BuildInfo)
                | (MetadataEntry::Mtree(_), MetadataFileName::Mtree) => return Ok(entry),
                _ => continue,
            }
        }
        Err(crate::Error::MetadataFileNotFound { name: file_name })
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
        let archive = Archive::new(decoder);
        Ok(Self { archive })
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

    /// Returns the [`PackageInfo`] of the package.
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
        let mut reader = PackageReader::try_from(self.clone())?;
        let metadata = reader.read_metadata_file(MetadataFileName::PackageInfo)?;
        match metadata {
            MetadataEntry::PackageInfo(pkginfo) => Ok(pkginfo),
            _ => Err(crate::Error::MetadataFileNotFound {
                name: MetadataFileName::PackageInfo,
            }),
        }
    }

    /// Returns the [`BuildInfo`] of the package.
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
        let mut reader = PackageReader::try_from(self.clone())?;
        let metadata = reader.read_metadata_file(MetadataFileName::BuildInfo)?;
        match metadata {
            MetadataEntry::BuildInfo(buildinfo) => Ok(buildinfo),
            _ => Err(crate::Error::MetadataFileNotFound {
                name: MetadataFileName::BuildInfo,
            }),
        }
    }

    /// Returns the [`Mtree`] of the package.
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
        let mut reader = PackageReader::try_from(self.clone())?;
        let metadata = reader.read_metadata_file(MetadataFileName::Mtree)?;
        match metadata {
            MetadataEntry::Mtree(mtree) => Ok(mtree),
            _ => Err(crate::Error::MetadataFileNotFound {
                name: MetadataFileName::Mtree,
            }),
        }
    }

    /// Reads the install scriptlet from the package, if present.
    ///
    /// Returns `None` if the package does not contain an install scriptlet.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - a [`PackageReader`] cannot be created for the package,
    /// - reading the entries via [`PackageReader::metadata_entries`] fails,
    /// - or reading the install scriptlet fails.
    pub fn read_install_scriptlet(&self) -> Result<Option<String>, crate::Error> {
        let mut reader = PackageReader::try_from(self.clone())?;
        for entry in reader.metadata_entries()?.flatten() {
            if let MetadataEntry::InstallScriptlet(scriptlet) = entry {
                return Ok(Some(scriptlet));
            }
        }
        Ok(None)
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
