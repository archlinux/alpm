//! Handling of files contained in an [alpm-db].
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::{
    fmt::Display,
    fs::{DirEntry, File, OpenOptions},
    io::{Seek, read_to_string},
    path::{Path, PathBuf},
};

use alpm_common::MetadataFile;
use alpm_mtree::Mtree;
use alpm_types::{EntryName, FullVersion, Name};
use fluent_i18n::t;

use crate::{db::Error, desc::DbDescFile, files::DbFiles};

/// The [alpm-db-desc] component of an entry in the [alpm-db].
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
/// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
#[derive(Debug)]
struct DescComponent {
    /// The (read-only) file handle for the component.
    file: File,

    path: PathBuf,
}

impl DescComponent {
    /// The file name of the component in the database entry.
    pub const FILE_NAME: &str = "desc";

    /// Creates a new [`DescComponent`] from a path.
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - the file name of `path` does not match that of [`Self::FILE_NAME`]
    /// - the file at `path` cannot be opened for reading
    pub fn new(path: impl AsRef<Path>) -> Result<Self, crate::Error> {
        let path = path.as_ref();

        let metadata = path
            .symlink_metadata()
            .map_err(|source| crate::Error::IoPath {
                path: path.to_path_buf(),
                context: t!("error-io-database-desc-metadata"),
                source,
            })?;

        if !metadata.is_file() {
            return Err(Error::EntryComponentIsNotAFile {
                path: path.to_path_buf(),
                file_name: Self::FILE_NAME,
            }
            .into());
        }

        let file =
            OpenOptions::new()
                .read(true)
                .open(path)
                .map_err(|source| crate::Error::IoPath {
                    path: path.to_path_buf(),
                    context: t!("error-io-database-desc-open-read-only"),
                    source,
                })?;

        Ok(Self {
            file,
            path: path.to_path_buf(),
        })
    }

    /// Returns the [`DbDescFile`].
    ///
    /// # Note
    ///
    /// Reads from the underlying file handle and afterwards rewinds it.
    ///
    /// # Errors
    ///
    /// Returns an errof if
    ///
    /// - a new [`DbDescFile`] cannot be created from the file handle
    /// - the file handle cannot be rewound
    pub fn to_db_desc(&self) -> Result<DbDescFile, crate::Error> {
        let mut file = &self.file;
        let output = DbDescFile::from_reader(file)?;
        file.rewind().map_err(|source| crate::Error::IoPath {
            path: self.path.clone(),
            context: t!("error-io-database-files-rewind"),
            source,
        })?;

        Ok(output)
    }
}

/// The [alpm-db-desc] component of an entry in the [alpm-db].
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
/// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
#[derive(Debug)]
struct FilesComponent {
    /// The (read-only) file handle for the component.
    file: File,

    path: PathBuf,
}

impl FilesComponent {
    /// The file name of the component in the database entry.
    pub const FILE_NAME: &str = "files";

    /// Creates a new [`FilesComponent`] from a path.
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - the file name of `path` does not match that of [`Self::FILE_NAME`]
    /// - the file at `path` cannot be opened for reading
    pub fn new(path: impl AsRef<Path>) -> Result<Self, crate::Error> {
        let path = path.as_ref();

        let metadata = path
            .symlink_metadata()
            .map_err(|source| crate::Error::IoPath {
                path: path.to_path_buf(),
                context: t!("error-io-database-files-metadata"),
                source,
            })?;

        if !metadata.is_file() {
            return Err(Error::EntryComponentIsNotAFile {
                path: path.to_path_buf(),
                file_name: Self::FILE_NAME,
            }
            .into());
        }

        let file =
            OpenOptions::new()
                .read(true)
                .open(path)
                .map_err(|source| crate::Error::IoPath {
                    path: path.to_path_buf(),
                    context: t!("error-io-database-files-open-read-only"),
                    source,
                })?;

        Ok(Self {
            file,
            path: path.to_path_buf(),
        })
    }

    /// Returns the [`DbFiles`].
    ///
    /// # Note
    ///
    /// Reads from the underlying file handle and afterwards rewinds it.
    ///
    /// # Errors
    ///
    /// Returns an errof if
    ///
    /// - a new [`DbFiles`] cannot be created from the file handle
    /// - the file handle cannot be rewound
    pub fn to_db_files(&self) -> Result<DbFiles, crate::Error> {
        let mut file = &self.file;
        let output = DbFiles::from_reader(file)?;
        file.rewind().map_err(|source| crate::Error::IoPath {
            path: self.path.clone(),
            context: t!("error-io-database-files-rewind"),
            source,
        })?;

        Ok(output)
    }
}

/// The optional [alpm-install-scriptlet] component of an entry in the [alpm-db].
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
/// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriplet.5.html
#[derive(Debug)]
struct InstallComponent {
    /// The (read-only) file handle for the component.
    file: File,

    path: PathBuf,
}

impl InstallComponent {
    /// The file name of the component in the database entry.
    pub const FILE_NAME: &str = "install";

    /// Creates a new [`InstallComponent`] from a path.
    ///
    /// # Note
    ///
    /// Returns `Some(InstallComponent)` if the optional component is found, `None` otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - the file name of `path` does not match that of [`Self::FILE_NAME`]
    /// - the file at `path` cannot be opened for reading
    pub fn new(path: impl AsRef<Path>) -> Result<Option<Self>, crate::Error> {
        let path = path.as_ref();

        // It's okay, if the optional alpm-install-scriptlet does not exist, but we fail if there
        // was an error checking.
        if !path.try_exists().map_err(|source| crate::Error::IoPath {
            path: path.to_path_buf(),
            context: t!("error-io-database-install-check-existence"),
            source,
        })? {
            return Ok(None);
        }

        let metadata = path
            .symlink_metadata()
            .map_err(|source| crate::Error::IoPath {
                path: path.to_path_buf(),
                context: t!("error-io-database-install-metadata"),
                source,
            })?;

        if !metadata.is_file() {
            return Err(Error::EntryComponentIsNotAFile {
                path: path.to_path_buf(),
                file_name: Self::FILE_NAME,
            }
            .into());
        }

        let file =
            OpenOptions::new()
                .read(true)
                .open(path)
                .map_err(|source| crate::Error::IoPath {
                    path: path.to_path_buf(),
                    context: t!("error-io-database-install-open-read-only"),
                    source,
                })?;

        Ok(Some(Self {
            file,
            path: path.to_path_buf(),
        }))
    }

    /// Returns the optional [alpm-install-scriptlet] as [`String`].
    ///
    /// # Note
    ///
    /// Reads from the underlying file handle and afterwards rewinds it.
    ///
    /// # Errors
    ///
    /// Returns an errof if
    ///
    /// - a new [alpm-install-scriptlet] cannot be created from the file handle
    /// - the file handle cannot be rewound
    ///
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriplet.5.html
    pub fn to_install_scriptlet(&self) -> Result<String, crate::Error> {
        let mut file = &self.file;
        let output = read_to_string(file).map_err(|source| crate::Error::IoPath {
            path: self.path.to_path_buf(),
            context: t!("error-io-database-install-rewind"),
            source,
        })?;
        file.rewind().map_err(|source| crate::Error::IoPath {
            path: self.path.clone(),
            context: t!("error-io-database-install-rewind"),
            source,
        })?;

        Ok(output)
    }
}

/// The [ALPM-MTREE] component of an entry in the [alpm-db].
///
/// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
#[derive(Debug)]
struct MtreeComponent {
    /// The (read-only) file handle for the [ALPM-MTREE] file.
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    file: File,

    path: PathBuf,
}

impl MtreeComponent {
    /// The file name in the database entry.
    pub const FILE_NAME: &str = "mtree";

    /// Creates a new [`MtreeComponent`] from a path.
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - the file name of `path` does not match that of [`Self::FILE_NAME`]
    /// - the file at `path` cannot be opened for reading
    pub fn new(path: impl AsRef<Path>) -> Result<Self, crate::Error> {
        let path = path.as_ref();

        let metadata = path
            .symlink_metadata()
            .map_err(|source| crate::Error::IoPath {
                path: path.to_path_buf(),
                context: t!("error-io-database-mtree-metadata"),
                source,
            })?;

        if !metadata.is_file() {
            return Err(Error::EntryComponentIsNotAFile {
                path: path.to_path_buf(),
                file_name: Self::FILE_NAME,
            }
            .into());
        }

        let file =
            OpenOptions::new()
                .read(true)
                .open(path)
                .map_err(|source| crate::Error::IoPath {
                    path: path.to_path_buf(),
                    context: t!("error-io-database-mtree-open-read-only"),
                    source,
                })?;

        Ok(Self {
            file,
            path: path.to_path_buf(),
        })
    }

    /// Returns the [`Mtree`].
    ///
    /// # Note
    ///
    /// Reads from the underlying file handle and afterwards rewinds it.
    ///
    /// # Errors
    ///
    /// Returns an errof if
    ///
    /// - a new [`Mtree`] cannot be created from the file handle
    /// - the file handle cannot be rewound
    pub fn to_mtree(&self) -> Result<Mtree, crate::Error> {
        let mut file = &self.file;
        let output = Mtree::from_reader(file)?;
        file.rewind().map_err(|source| crate::Error::IoPath {
            path: self.path.clone(),
            context: t!("error-io-database-files-rewind"),
            source,
        })?;

        Ok(output)
    }
}

impl TryFrom<MtreeComponent> for Mtree {
    type Error = crate::Error;

    fn try_from(value: MtreeComponent) -> Result<Self, Self::Error> {
        Ok(Self::from_reader(value.file)?)
    }
}

/// The representation of an entry in an [alpm-db].
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
#[derive(Debug)]
pub struct Entry {
    /// The name of the entry.
    name: EntryName,

    alpm_db_desc: DescComponent,

    alpm_db_files: FilesComponent,

    alpm_install_scriptlet: Option<InstallComponent>,

    alpm_mtree: MtreeComponent,
}

impl Entry {
    /// Creates a new [`Entry`] from a path.
    ///
    /// The `path` represents an [alpm-db] entry, which must contain the [alpm-db-desc],
    /// [alpm-db-files] and [ALPM-MTREE] components and optionally the [alpm-install-scriptlet]
    /// component.
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - an [`EntryName`] cannot be created from the file name of `path`
    /// - a file for an [alpm-db-desc], [alpm-db-files] or [ALPM-MTREE] component cannot be found
    /// - the optional [alpm-install-scriptlet] component is not a file
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    /// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    /// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriplet.5.html
    pub fn new(path: impl AsRef<Path>) -> Result<Self, crate::Error> {
        let path = path.as_ref();
        let name = EntryName::try_from(path)?;
        let alpm_db_desc = DescComponent::new(path.join(DescComponent::FILE_NAME))?;
        let alpm_db_files = FilesComponent::new(path.join(FilesComponent::FILE_NAME))?;
        let alpm_mtree = MtreeComponent::new(path.join(MtreeComponent::FILE_NAME))?;
        let alpm_install_scriptlet = InstallComponent::new(path.join(InstallComponent::FILE_NAME))?;

        Ok(Self {
            name,
            alpm_db_desc,
            alpm_db_files,
            alpm_install_scriptlet,
            alpm_mtree,
        })
    }

    /// Returns a reference to the [`Name`] of the entry.
    pub fn name(&self) -> &Name {
        self.name.name()
    }

    /// Returns a reference to the [`FullVersion`] of the entry.
    pub fn version(&self) -> &FullVersion {
        self.name.version()
    }

    /// Returns the [`DbDescFile`].
    ///
    /// # Note
    ///
    /// Reads from the underlying file handle and afterwards rewinds it.
    ///
    /// # Errors
    ///
    /// Returns an errof if
    ///
    /// - a new [`DbDescFile`] cannot be created from the file handle
    /// - the file handle cannot be rewound
    pub fn db_desc(&mut self) -> Result<DbDescFile, crate::Error> {
        self.alpm_db_desc.to_db_desc()
    }

    /// Returns the [`DbFiles`].
    ///
    /// # Note
    ///
    /// Reads from the underlying file handle and afterwards rewinds it.
    ///
    /// # Errors
    ///
    /// Returns an errof if
    ///
    /// - a new [`DbFiles`] cannot be created from the file handle
    /// - the file handle cannot be rewound
    pub fn db_files(&mut self) -> Result<DbFiles, crate::Error> {
        self.alpm_db_files.to_db_files()
    }

    /// Returns the optional [alpm-install-scriptlet] as [`String`].
    ///
    /// # Note
    ///
    /// Reads from the underlying file handle and afterwards rewinds it.
    ///
    /// Returns `Ok(None)`, if the entry has no [alpm-install-scriptlet].
    ///
    /// # Errors
    ///
    /// Returns an errof if
    ///
    /// - a new [alpm-install-scriptlet] cannot be created from the file handle
    /// - the file handle cannot be rewound
    ///
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriplet.5.html
    pub fn install_scriptlet(&mut self) -> Result<Option<String>, crate::Error> {
        let Some(install_scriptlet) = self.alpm_install_scriptlet.as_ref() else {
            return Ok(None);
        };

        Ok(Some(install_scriptlet.to_install_scriptlet()?))
    }

    /// Returns the [`Mtree`].
    ///
    /// # Note
    ///
    /// Reads from the underlying file handle and afterwards rewinds it.
    ///
    /// # Errors
    ///
    /// Returns an errof if
    ///
    /// - a new [`Mtree`] cannot be created from the file handle
    /// - the file handle cannot be rewound
    pub fn mtree(&mut self) -> Result<Mtree, crate::Error> {
        self.alpm_mtree.to_mtree()
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl TryFrom<DirEntry> for Entry {
    type Error = crate::Error;

    /// Creates a new [`Entry`] from a [`DirEntry`].
    fn try_from(value: DirEntry) -> Result<Self, Self::Error> {
        Self::new(value.path())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::create_dir_all;

    use rstest::rstest;
    use tempfile::TempDir;
    use testresult::TestResult;

    use super::*;

    /// Ensures, that [`Entry::new`] succeeds on valid input (but no data).
    #[rstest]
    #[case::with_install_file(true)]
    #[case::without_install_file(false)]
    fn entry_new_succeeds(#[case] with_install: bool) -> TestResult {
        let temp_dir = TempDir::new()?;
        let entry_dir = temp_dir.path().join("example-1.0.0-1");
        create_dir_all(&entry_dir)?;
        let mut file_names: Vec<&str> = vec!["desc", "files", "mtree"];
        if with_install {
            file_names.push("install");
        }

        for file_name in file_names {
            File::create(entry_dir.join(file_name))?;
        }

        Entry::new(&entry_dir)?;

        Ok(())
    }
}
