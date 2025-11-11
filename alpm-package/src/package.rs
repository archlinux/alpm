//! Representation of [alpm-package] files.
//!
//! [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html

use std::{
    fmt::{self, Debug},
    fs::{File, create_dir_all},
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_buildinfo::BuildInfo;
use alpm_common::{InputPaths, MetadataFile};
use alpm_compress::tarball::{TarballBuilder, TarballEntries, TarballEntry, TarballReader};
use alpm_mtree::Mtree;
use alpm_pkginfo::PackageInfo;
use alpm_types::{INSTALL_SCRIPTLET_FILE_NAME, MetadataFileName, PackageError, PackageFileName};
use log::debug;

use crate::{OutputDir, PackageCreationConfig};

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

/// Appends relative files from an input directory to a [`TarballBuilder`].
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
fn append_relative_files<'c>(
    mut builder: TarballBuilder<'c>,
    mtree: &Mtree,
    input_paths: &InputPaths,
) -> Result<TarballBuilder<'c>, crate::Error> {
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
            .inner_mut()
            .append_path_with_name(from_path.as_path(), relative_file.as_path())
            .map_err(|source| Error::AppendFileToArchive {
                from_path,
                to_path: relative_file.clone(),
                source,
            })?
    }

    Ok(builder)
}

/// An entry in a package archive.
///
/// This can be either a metadata file (such as [PKGINFO], [BUILDINFO], or [ALPM-MTREE]) or an
/// [alpm-install-scriptlet] file.
///
/// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
/// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
/// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
/// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
#[derive(Clone, Debug)]
pub enum PackageEntry {
    /// A metadata entry in the package archive.
    ///
    /// See [`MetadataEntry`] for the different types of metadata entries.
    ///
    /// This variant is boxed to avoid large allocations
    Metadata(Box<MetadataEntry>),

    /// An [alpm-install-scriptlet] file in the package.
    ///
    /// [alpm-install-scriptlet]:
    /// https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    InstallScriptlet(String),
}

/// Metadata entry contained in an [alpm-package] file.
///
/// This is used e.g. in [`PackageReader::metadata_entries`] when iterating over available
/// metadata files.
///
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
#[derive(Clone, Debug)]
pub enum MetadataEntry {
    /// The [PKGINFO] data.
    ///
    /// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
    PackageInfo(PackageInfo),

    /// The [BUILDINFO] data.
    ///
    /// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
    BuildInfo(BuildInfo),

    /// The [ALPM-MTREE] data.
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    Mtree(Mtree),
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
}

/// An iterator over each [`PackageEntry`] of a package.
///
/// Stops early once all package entry files have been found.
///
/// # Note
///
/// Uses two lifetimes for the underlying [`TarballEntries`]
pub struct PackageEntryIterator<'a, 'c> {
    /// The archive entries iterator that contains all of the archive's entries.
    entries: TarballEntries<'a, 'c>,
    /// Whether a `.BUILDINFO` file has been found.
    found_buildinfo: bool,
    /// Whether a `.MTREE` file has been found.
    found_mtree: bool,
    /// Whether a `.PKGINFO` file has been found.
    found_pkginfo: bool,
    /// Whether a `.INSTALL` scriptlet has been found or skipped.
    checked_install_scriptlet: bool,
}

impl Debug for PackageEntryIterator<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PackageEntryIterator")
            .field("entries", &"TarballEntries")
            .field("found_buildinfo", &self.found_buildinfo)
            .field("found_mtree", &self.found_mtree)
            .field("found_pkginfo", &self.found_pkginfo)
            .field("checked_install_scriptlet", &self.checked_install_scriptlet)
            .finish()
    }
}

impl<'a, 'c> PackageEntryIterator<'a, 'c> {
    /// Creates a new [`PackageEntryIterator`] from [`TarballEntries`].
    pub fn new(entries: TarballEntries<'a, 'c>) -> Self {
        Self {
            entries,
            found_buildinfo: false,
            found_mtree: false,
            found_pkginfo: false,
            checked_install_scriptlet: false,
        }
    }

    /// Return the inner [`TarballEntries`] iterator at the current iteration position.
    pub fn into_inner(self) -> TarballEntries<'a, 'c> {
        self.entries
    }

    /// Checks whether all variants of [`PackageEntry`] have been found.
    ///
    /// Returns `true` if all variants of [`PackageEntry`] have been found, `false` otherwise.
    fn all_entries_found(&self) -> bool {
        self.checked_install_scriptlet
            && self.found_pkginfo
            && self.found_mtree
            && self.found_buildinfo
    }

    /// A helper function that returns an optional [`PackageEntry`] from a [`TarballEntry`].
    ///
    /// Based on the path of `entry` either returns:
    ///
    /// - `Ok(Some(PackageEntry))` when a valid [`PackageEntry`] is detected,
    /// - `Ok(None)` for any other files.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - no path can be retrieved from `entry`,
    /// - the path of `entry` indicates a [BUILDINFO] file, but a [`BuildInfo`] cannot be created
    ///   from it,
    /// - the path of `entry` indicates an [ALPM-MTREE] file, but an [`Mtree`] cannot be created
    ///   from it,
    /// - the path of `entry` indicates a [PKGINFO] file, but a [`PackageInfo`] cannot be created
    ///   from it,
    /// - or the path of `entry` indicates an [alpm-install-script] file, but it cannot be read to a
    ///   string.
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    /// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
    /// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    fn get_package_entry(mut entry: TarballEntry) -> Result<Option<PackageEntry>, crate::Error> {
        let path = entry.path().to_string_lossy();
        match path.as_ref() {
            p if p == MetadataFileName::PackageInfo.as_ref() => {
                let info = PackageInfo::from_reader(&mut entry)?;
                Ok(Some(PackageEntry::Metadata(Box::new(
                    MetadataEntry::PackageInfo(info),
                ))))
            }
            p if p == MetadataFileName::BuildInfo.as_ref() => {
                let info = BuildInfo::from_reader(&mut entry)?;
                Ok(Some(PackageEntry::Metadata(Box::new(
                    MetadataEntry::BuildInfo(info),
                ))))
            }
            p if p == MetadataFileName::Mtree.as_ref() => {
                let info = Mtree::from_reader(&mut entry)?;
                Ok(Some(PackageEntry::Metadata(Box::new(
                    MetadataEntry::Mtree(info),
                ))))
            }
            INSTALL_SCRIPTLET_FILE_NAME => {
                let mut scriptlet = String::new();
                entry
                    .read_to_string(&mut scriptlet)
                    .map_err(|source| crate::Error::IoRead {
                        context: "reading install scriptlet",
                        source,
                    })?;
                Ok(Some(PackageEntry::InstallScriptlet(scriptlet)))
            }
            _ => Ok(None),
        }
    }
}

impl Iterator for PackageEntryIterator<'_, '_> {
    type Item = Result<PackageEntry, crate::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // Return early if we already found all entries.
        // In that case we don't need to continue iteration.
        if self.all_entries_found() {
            return None;
        }

        for entry_result in &mut self.entries {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(e) => return Some(Err(e.into())),
            };

            // Get the package entry and convert `Result<Option<PackageEntry>>` to a
            // `Option<Result<PackageEntry>>`.
            let entry = Self::get_package_entry(entry).transpose();

            // Now, if the entry is either an error or a valid PackageEntry, return it.
            // Otherwise, we look at the next entry.
            match entry {
                Some(Ok(ref package_entry)) => {
                    // Remember each file we found.
                    // Once all files are found, the iterator can short-circuit and stop early.
                    match &package_entry {
                        PackageEntry::Metadata(metadata_entry) => match **metadata_entry {
                            MetadataEntry::PackageInfo(_) => self.found_pkginfo = true,
                            MetadataEntry::BuildInfo(_) => self.found_buildinfo = true,
                            MetadataEntry::Mtree(_) => self.found_mtree = true,
                        },
                        PackageEntry::InstallScriptlet(_) => self.checked_install_scriptlet = true,
                    }
                    return entry;
                }
                Some(Err(e)) => return Some(Err(e)),
                _ if self.found_buildinfo && self.found_mtree && self.found_pkginfo => {
                    // Found three required metadata files and hit the first non-metadata file.
                    // This means that install scriptlet does not exist in the package and we
                    // can stop iterating.
                    //
                    // This logic relies on the ordering of files, where the `.INSTALL` file is
                    // placed in between `.PKGINFO` and `.MTREE`.
                    self.checked_install_scriptlet = true;
                    break;
                }
                _ => (),
            }
        }

        None
    }
}

/// An iterator over each [`MetadataEntry`] of a package.
///
/// Stops early once all metadata files have been found.
///
/// # Notes
///
/// Uses two lifetimes for the underlying [`TarballEntries`] of [`PackageEntryIterator`]
/// in the `entries` field.
pub struct MetadataEntryIterator<'a, 'c> {
    /// The archive entries iterator that contains all archive's entries.
    entries: PackageEntryIterator<'a, 'c>,
    /// Whether a `.BUILDINFO` file has been found.
    found_buildinfo: bool,
    /// Whether a `.MTREE` file has been found.
    found_mtree: bool,
    /// Whether a `.PKGINFO` file has been found.
    found_pkginfo: bool,
}

impl Debug for MetadataEntryIterator<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MetadataEntryIterator")
            .field("entries", &self.entries)
            .field("found_buildinfo", &self.found_buildinfo)
            .field("found_mtree", &self.found_mtree)
            .field("found_pkginfo", &self.found_pkginfo)
            .finish()
    }
}

impl<'a, 'c> MetadataEntryIterator<'a, 'c> {
    /// Creates a new [`MetadataEntryIterator`] from a [`PackageEntryIterator`].
    pub fn new(entries: PackageEntryIterator<'a, 'c>) -> Self {
        Self {
            entries,
            found_buildinfo: false,
            found_mtree: false,
            found_pkginfo: false,
        }
    }

    /// Return the inner [`PackageEntryIterator`] iterator at the current iteration position.
    pub fn into_inner(self) -> PackageEntryIterator<'a, 'c> {
        self.entries
    }

    /// Checks whether all variants of [`MetadataEntry`] have been found.
    ///
    /// Returns `true` if all known types of [`MetadataEntry`] have been found, `false` otherwise.
    fn all_entries_found(&self) -> bool {
        self.found_pkginfo && self.found_mtree && self.found_buildinfo
    }
}

impl Iterator for MetadataEntryIterator<'_, '_> {
    type Item = Result<MetadataEntry, crate::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // Return early if we already found all entries.
        // In that case we don't need to continue iteration.
        if self.all_entries_found() {
            return None;
        }

        // Now check whether we have any entries left.
        for entry_result in &mut self.entries {
            let metadata = match entry_result {
                Ok(PackageEntry::Metadata(metadata)) => metadata,
                Ok(PackageEntry::InstallScriptlet(_)) => continue,
                Err(e) => return Some(Err(e)),
            };

            match *metadata {
                MetadataEntry::PackageInfo(_) => self.found_pkginfo = true,
                MetadataEntry::BuildInfo(_) => self.found_buildinfo = true,
                MetadataEntry::Mtree(_) => self.found_mtree = true,
            }
            return Some(Ok(*metadata));
        }

        None
    }
}

/// A reader for [`Package`] files.
///
/// A [`PackageReader`] can be created from a [`Package`] using the
/// [`Package::into_reader`] or [`PackageReader::try_from`] methods.
///
/// # Examples
///
/// ```
/// # use std::fs::{File, Permissions, create_dir_all};
/// # use std::io::Write;
/// # use std::os::unix::fs::PermissionsExt;
/// use std::path::Path;
///
/// # use alpm_mtree::create_mtree_v2_from_input_dir;
/// use alpm_package::{MetadataEntry, Package, PackageReader};
/// # use alpm_package::{
/// #     InputDir,
/// #     OutputDir,
/// #     PackageCreationConfig,
/// #     PackageInput,
/// # };
/// # use alpm_compress::compression::CompressionSettings;
/// use alpm_types::MetadataFileName;
///
/// # fn main() -> testresult::TestResult {
/// // A directory for the package file.
/// let temp_dir = tempfile::tempdir()?;
/// let path = temp_dir.path();
/// # let input_dir = path.join("input");
/// # create_dir_all(&input_dir)?;
/// # let input_dir = InputDir::new(input_dir)?;
/// # let output_dir = OutputDir::new(path.join("output"))?;
/// #
/// # // Create a valid, but minimal BUILDINFOv2 file.
/// # let mut file = File::create(&input_dir.join(MetadataFileName::BuildInfo.as_ref()))?;
/// # write!(file, r#"
/// # format = 2
/// # builddate = 1
/// # builddir = /build
/// # startdir = /startdir/
/// # buildtool = devtools
/// # buildtoolver = 1:1.2.1-1-any
/// # installed = other-example-1.2.3-1-any
/// # packager = John Doe <john@example.org>
/// # pkgarch = any
/// # pkgbase = example
/// # pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
/// # pkgname = example
/// # pkgver = 1.0.0-1
/// # "#)?;
/// #
/// # // Create a valid, but minimal PKGINFOv2 file.
/// # let mut file = File::create(&input_dir.join(MetadataFileName::PackageInfo.as_ref()))?;
/// # write!(file, r#"
/// # pkgname = example
/// # pkgbase = example
/// # xdata = pkgtype=pkg
/// # pkgver = 1.0.0-1
/// # pkgdesc = A project that returns true
/// # url = https://example.org/
/// # builddate = 1
/// # packager = John Doe <john@example.org>
/// # size = 181849963
/// # arch = any
/// # license = GPL-3.0-or-later
/// # depend = bash
/// # "#)?;
/// #
/// # // Create a dummy script as package data.
/// # create_dir_all(&input_dir.join("usr/bin"))?;
/// # let mut file = File::create(&input_dir.join("usr/bin/example"))?;
/// # write!(file, r#"!/bin/bash
/// # true
/// # "#)?;
/// # file.set_permissions(Permissions::from_mode(0o755))?;
/// #
/// # // Create a valid ALPM-MTREEv2 file from the input directory.
/// # create_mtree_v2_from_input_dir(&input_dir)?;
/// #
/// # // Create PackageInput and PackageCreationConfig.
/// # let package_input: PackageInput = input_dir.try_into()?;
/// # let config = PackageCreationConfig::new(
/// #     package_input,
/// #     output_dir,
/// #     CompressionSettings::default(),
/// # )?;
///
/// # // Create package file.
/// # let package = Package::try_from(&config)?;
/// // Assume that the package is created
/// let package_path = path.join("output/example-1.0.0-1-any.pkg.tar.zst");
///
/// // Create a reader for the package.
/// let mut reader = package.clone().into_reader()?;
///
/// // Read all the metadata from the package archive.
/// let metadata = reader.metadata()?;
/// let pkginfo = metadata.pkginfo;
/// let buildinfo = metadata.buildinfo;
/// let mtree = metadata.mtree;
///
/// // Or you can iterate over the metadata entries:
/// let mut reader = package.clone().into_reader()?;
/// for entry in reader.metadata_entries()? {
///     let entry = entry?;
///     match entry {
///         MetadataEntry::PackageInfo(pkginfo) => {}
///         MetadataEntry::BuildInfo(buildinfo) => {}
///         MetadataEntry::Mtree(mtree) => {}
///         _ => {}
///     }
/// }
///
/// // You can also read specific metadata files directly:
/// let mut reader = package.clone().into_reader()?;
/// let pkginfo = reader.read_metadata_file(MetadataFileName::PackageInfo)?;
/// // let buildinfo = reader.read_metadata_file(MetadataFileName::BuildInfo)?;
/// // let mtree = reader.read_metadata_file(MetadataFileName::Mtree)?;
///
/// // Read the install scriptlet, if present:
/// let mut reader = package.clone().into_reader()?;
/// let install_scriptlet = reader.read_install_scriptlet()?;
///
/// // Iterate over the data entries in the package archive.
/// let mut reader = package.clone().into_reader()?;
/// for data_entry in reader.data_entries()? {
///     let mut data_entry = data_entry?;
///     let content = data_entry.content()?;
///     // Note: data_entry also implements `Read`, so you can read from it directly.
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Notes
///
/// This API is designed with **streaming** and **single-pass iteration** in mind.
///
/// Calling [`Package::into_reader`] creates a new [`PackageReader`] each time,
/// which consumes the underlying archive in a forward-only manner. This allows
/// efficient access to package contents without needing to load the entire archive
/// into memory.
///
/// If you need to perform multiple operations on a package, you can call
/// [`Package::into_reader`] multiple times — each reader starts fresh and ensures
/// predictable, deterministic access to the archive's contents.
///
/// Please note that convenience methods on [`Package`] itself, such as
/// [`Package::read_pkginfo`], are also provided for better ergonomics
/// and ease of use.
///
/// The lifetimes `'c` is for the [`TarballReader`]
#[derive(Debug)]
pub struct PackageReader<'c>(TarballReader<'c>);

impl<'c> PackageReader<'c> {
    /// Creates a new [`PackageReader`] from an [`TarballReader`].
    pub fn new(tarball_reader: TarballReader<'c>) -> Self {
        Self(tarball_reader)
    }

    fn is_scriplet_file(entry: &TarballEntry) -> bool {
        let path = entry.path().to_string_lossy();
        path.as_ref() == INSTALL_SCRIPTLET_FILE_NAME
    }

    fn is_metadata_file(entry: &TarballEntry) -> bool {
        let metadata_file_names = [
            MetadataFileName::PackageInfo.as_ref(),
            MetadataFileName::BuildInfo.as_ref(),
            MetadataFileName::Mtree.as_ref(),
        ];
        let path = entry.path().to_string_lossy();
        metadata_file_names.contains(&path.as_ref())
    }

    fn is_data_file(entry: &TarballEntry) -> bool {
        !Self::is_scriplet_file(entry) && !Self::is_metadata_file(entry)
    }

    /// Returns an iterator over the raw entries of the package's tar archive.
    ///
    /// The returned [`TarballEntries`] implements an iterator over each [`TarballEntry`],
    /// which provides direct data access to all entries of the package's tar archive.
    ///
    /// # Errors
    ///
    /// Returns an error if the [`TarballEntries`] cannot be read from the package's tar archive.
    pub fn raw_entries<'a>(&'a mut self) -> Result<TarballEntries<'a, 'c>, crate::Error> {
        Ok(self.0.entries()?)
    }

    /// Returns an iterator over the known files in the [alpm-package] file.
    ///
    /// This iterator yields a set of [`PackageEntry`] variants, which may only contain data
    /// from metadata files (i.e. [ALPM-MTREE], [BUILDINFO] or [PKGINFO]) or an install scriptlet
    /// (i.e. [alpm-install-scriplet]).
    ///
    /// # Note
    ///
    /// The file names of metadata file formats (i.e. [ALPM-MTREE], [BUILDINFO], [PKGINFO])
    /// and install scriptlets (i.e. [alpm-install-scriptlet]) are prefixed with a dot (`.`)
    /// in [alpm-package] files.
    ///
    /// As [alpm-package] files are assumed to contain a sorted list of entries, these files are
    /// considered first. The iterator stops as soon as it encounters an entry that does not
    /// match any known metadata file or install scriptlet file name.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - reading the package archive entries fails,
    /// - reading a package archive entry fails,
    /// - reading the contents of a package archive entry fails,
    /// - or retrieving the path of a package archive entry fails.
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    /// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
    /// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    /// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
    pub fn entries<'a>(&'a mut self) -> Result<PackageEntryIterator<'a, 'c>, crate::Error> {
        let entries = self.raw_entries()?;
        Ok(PackageEntryIterator::new(entries))
    }

    /// Returns an iterator over the metadata entries in the package archive.
    ///
    /// This iterator yields [`MetadataEntry`]s, which can be either [PKGINFO], [BUILDINFO],
    /// or [ALPM-MTREE].
    ///
    /// The iterator stops when it encounters an entry that does not match any
    /// known package files.
    ///
    /// It is a wrapper around [`PackageReader::entries`] that filters out
    /// the install scriptlet.
    ///
    /// # Errors
    ///
    /// Returns an error if [`PackageReader::entries`] fails to read the entries.
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    /// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
    /// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
    pub fn metadata_entries<'a>(
        &'a mut self,
    ) -> Result<MetadataEntryIterator<'a, 'c>, crate::Error> {
        let entries = self.entries()?;
        Ok(MetadataEntryIterator::new(entries))
    }

    /// Returns an iterator over the data files of the [alpm-package] archive.
    ///
    /// This iterator yields the path and content of each data file of a package archive in the form
    /// of a [`TarballEntry`].
    ///
    /// # Notes
    ///
    /// This iterator filters out the known metadata files [PKGINFO], [BUILDINFO] and [ALPM-MTREE].
    /// and the [alpm-install-scriplet] file.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - reading the package archive entries fails,
    /// - reading a package archive entry fails,
    /// - reading the contents of a package archive entry fails,
    /// - or retrieving the path of a package archive entry fails.
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    /// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
    /// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    /// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
    pub fn data_entries<'a>(
        &'a mut self,
    ) -> Result<impl Iterator<Item = Result<TarballEntry<'a, 'c>, crate::Error>>, crate::Error>
    {
        let entries = self.raw_entries()?;
        Ok(entries.filter_map(move |entry| {
            let filter = (|| {
                let entry = entry?;
                // Filter out known metadata files
                if !Self::is_data_file(&entry) {
                    return Ok(None);
                }
                Ok(Some(entry))
            })();
            filter.transpose()
        }))
    }

    /// Reads all metadata from an [alpm-package] file.
    ///
    /// This method reads all the metadata entries in the package file and returns a
    /// [`Metadata`] struct containing the processed data.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - reading the metadata entries fails,
    /// - parsing a metadata entry fails,
    /// - or if any of the required metadata files are not found in the package.
    ///
    /// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
    pub fn metadata(&mut self) -> Result<Metadata, crate::Error> {
        let mut pkginfo = None;
        let mut buildinfo = None;
        let mut mtree = None;
        for entry in self.metadata_entries()? {
            match entry? {
                MetadataEntry::PackageInfo(m) => pkginfo = Some(m),
                MetadataEntry::BuildInfo(m) => buildinfo = Some(m),
                MetadataEntry::Mtree(m) => mtree = Some(m),
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
        })
    }

    /// Reads the data of a specific metadata file from the [alpm-package] file.
    ///
    /// This method searches for a metadata file that matches the provided
    /// [`MetadataFileName`] and returns the corresponding [`MetadataEntry`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - [`PackageReader::metadata_entries`] fails to retrieve the metadata entries,
    /// - or a [`MetadataEntry`] is not valid.
    ///
    /// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
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

    /// Reads the content of the [alpm-install-scriptlet] from the package archive, if it exists.
    ///
    /// # Errors
    ///
    /// Returns an error if [`PackageReader::entries`] fails to read the entries.
    ///
    /// [alpm-install-scriplet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    pub fn read_install_scriptlet(&mut self) -> Result<Option<String>, crate::Error> {
        for entry in self.entries()? {
            let entry = entry?;
            if let PackageEntry::InstallScriptlet(scriptlet) = entry {
                return Ok(Some(scriptlet));
            }
        }
        Ok(None)
    }

    /// Reads a [`TarballEntry`] matching a specific path name from the package archive.
    ///
    /// Returns [`None`] if no [`TarballEntry`] is found in the package archive that matches `path`.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - [`PackageReader::data_entries`] fails to retrieve the data entries,
    /// - or retrieving the details of a data entry fails.
    pub fn read_data_entry<'a, P: AsRef<Path>>(
        &'a mut self,
        path: P,
    ) -> Result<Option<TarballEntry<'a, 'c>>, crate::Error> {
        for entry in self.data_entries()? {
            let entry = entry?;
            if entry.path() == path.as_ref() {
                return Ok(Some(entry));
            }
        }
        Ok(None)
    }
}

impl TryFrom<Package> for PackageReader<'_> {
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
        Ok(Self::new(TarballReader::try_from(path)?))
    }
}

impl TryFrom<&Path> for PackageReader<'_> {
    type Error = crate::Error;

    /// Creates a [`PackageReader`] from a [`Path`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - [`Package::try_from`] fails to create a [`Package`] from `path`,
    /// - or [`PackageReader::try_from`] fails to create a [`PackageReader`] from the package.
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let package = Package::try_from(path)?;
        PackageReader::try_from(package)
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
    /// This is a convenience wrapper around [`PackageReader::read_metadata_file`].
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
    /// This is a convenience wrapper around [`PackageReader::read_metadata_file`].
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
    /// This is a convenience wrapper around [`PackageReader::read_metadata_file`].
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

    /// Returns the contents of the optional [alpm-install-scriptlet] of the package.
    ///
    /// Returns [`None`] if the package does not contain an [alpm-install-scriptlet] file.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - a [`PackageReader`] cannot be created for the package,
    /// - or reading the entries using [`PackageReader::metadata_entries`].
    ///
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    pub fn read_install_scriptlet(&self) -> Result<Option<String>, crate::Error> {
        let mut reader = PackageReader::try_from(self.clone())?;
        reader.read_install_scriptlet()
    }

    /// Creates a [`PackageReader`] for the package.
    ///
    /// Convenience wrapper for [`PackageReader::try_from`].
    ///
    /// # Errors
    ///
    /// Returns an error if `self` cannot be converted into a [`PackageReader`].
    pub fn into_reader<'c>(self) -> Result<PackageReader<'c>, crate::Error> {
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
    /// - creating a [`TarballBuilder`] fails,
    /// - creating a compressed or uncompressed package file fails,
    /// - validating any of the paths using ALPM-MTREE data (available through `value`) fails,
    /// - appending files to a compressed or uncompressed package file fails,
    /// - finishing a compressed or uncompressed package file fails,
    /// - or creating a [`Package`] fails.
    fn try_from(value: &PackageCreationConfig) -> Result<Self, Self::Error> {
        let filename = PackageFileName::from(value);
        let parent_dir: ExistingAbsoluteDir = value.output_dir().into();
        let output_path = value.output_dir().join(filename.to_path_buf());

        // Create the output file.
        let file = File::create(output_path.as_path()).map_err(|source| crate::Error::IoPath {
            path: output_path.clone(),
            context: "creating a package file",
            source,
        })?;

        let mut builder = TarballBuilder::new(file, value.compression())?;
        builder.inner_mut().follow_symlinks(false);
        builder = append_relative_files(
            builder,
            value.package_input().mtree()?,
            &value.package_input().input_paths()?,
        )?;
        builder.finish()?;

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

    /// Ensure that the Debug implementation of [`PackageEntryIterator`] and
    /// [`MetadataEntryIterator`] works as expected.
    #[test]
    fn package_entry_iterators_debug() -> TestResult {
        init_logger();

        let temp_dir = TempDir::new()?;
        let path = temp_dir.path();
        let absolute_dir = ExistingAbsoluteDir::new(path.to_path_buf())?;
        let package_name = "example-1.0.0-1-x86_64.pkg.tar.zst";
        File::create(absolute_dir.join(package_name))?;
        let package = Package::new(package_name.parse()?, absolute_dir.clone())?;

        // Create iterators
        let mut reader = PackageReader::try_from(package.clone())?;
        let entry_iter = reader.entries()?;

        let mut reader = PackageReader::try_from(package.clone())?;
        let metadata_iter = reader.metadata_entries()?;

        assert_eq!(
            format!("{entry_iter:?}"),
            "PackageEntryIterator { entries: \"TarballEntries\", found_buildinfo: false, \
                found_mtree: false, found_pkginfo: false, checked_install_scriptlet: false }"
        );
        assert_eq!(
            format!("{metadata_iter:?}"),
            "MetadataEntryIterator { entries: PackageEntryIterator { entries: \"TarballEntries\", \
                found_buildinfo: false, found_mtree: false, found_pkginfo: false, checked_install_scriptlet: false }, \
                found_buildinfo: false, found_mtree: false, found_pkginfo: false }"
        );

        Ok(())
    }
}
