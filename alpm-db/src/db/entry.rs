//! Handling of files contained in an [alpm-db].
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::{
    fmt::Display,
    fs::{File, OpenOptions},
    io::{Seek, read_to_string},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use alpm_common::MetadataFile;
use alpm_mtree::Mtree;
use alpm_types::{EntryName, FullVersion, Name};
use fluent_i18n::t;

use crate::{db::Error, desc::DbDescFile, files::DbFiles};

/// The default permissions for a component of an [alpm-db] entry.
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
pub(crate) const ALPM_DB_ENTRY_COMPONENT_FILE_MODE: u32 = 0o100644;

/// The file name of the [alpm-db-desc] component in a database entry.
///
/// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
pub(crate) const ALPM_DB_DESC_FILE_NAME: &str = "desc";

/// The file name of the [alpm-db-files] component in a database entry.
///
/// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
pub(crate) const ALPM_DB_FILES_FILE_NAME: &str = "files";

/// The file name of the optional [alpm-install-scriptlet] component in a database entry.
///
/// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
pub(crate) const ALPM_INSTALL_SCRIPTLET_FILE_NAME: &str = "install";

/// The file name of the [ALPM-MTREE] component in a database entry.
///
/// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
pub(crate) const ALPM_MTREE_FILE_NAME: &str = "mtree";

/// Returns a read-only [`File`] handle for a component file.
///
/// # Errors
///
/// Returns an error, if
///
/// - the file name of `path` does not match `name`
/// - the metadata for `path` cannot be retrieved
/// - `path` is not a file
/// - the file permissions of `path` do not match that of [`ALPM_DB_ENTRY_COMPONENT_FILE_MODE`]
/// - the file at `path` cannot be opened for reading
fn component_file(path: &Path, name: &'static str) -> Result<File, crate::Error> {
    if path.file_name().is_none() || path.file_name().is_some_and(|file_name| file_name != name) {
        return Err(Error::EntryComponentWrongFileName {
            path: path.to_path_buf(),
            expected: name,
        }
        .into());
    }

    let metadata = path
        .symlink_metadata()
        .map_err(|source| crate::Error::IoPath {
            path: path.to_path_buf(),
            context: t!("error-io-metadata"),
            source,
        })?;

    if !metadata.is_file() {
        return Err(Error::EntryComponentIsNotAFile {
            path: path.to_path_buf(),
        }
        .into());
    }

    if metadata.permissions().mode() != ALPM_DB_ENTRY_COMPONENT_FILE_MODE {
        return Err(Error::EntryComponentWrongPermissions {
            path: path.to_path_buf(),
            permissions: metadata.permissions().mode(),
        }
        .into());
    }

    OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|source| crate::Error::IoPath {
            path: path.to_path_buf(),
            context: t!("error-io-path-open-read-only"),
            source,
        })
}

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
    /// Creates a new [`DescComponent`] from a path.
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - the file name of `path` does not match that of the component
    /// - the file at `path` cannot be opened for reading
    pub fn new(path: impl AsRef<Path>) -> Result<Self, crate::Error> {
        let path = path.as_ref();

        Ok(Self {
            file: component_file(path, ALPM_DB_DESC_FILE_NAME)?,
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
            context: t!("error-io-path-rewind"),
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
    /// Creates a new [`FilesComponent`] from a path.
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - the file name of `path` does not match that of the component
    /// - the file at `path` cannot be opened for reading
    pub fn new(path: impl AsRef<Path>) -> Result<Self, crate::Error> {
        let path = path.as_ref();

        Ok(Self {
            file: component_file(path, ALPM_DB_FILES_FILE_NAME)?,
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
            context: t!("error-io-path-rewind"),
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
    /// - the file name of `path` does not match that of the component
    /// - the file at `path` cannot be opened for reading
    pub fn new(path: impl AsRef<Path>) -> Result<Option<Self>, crate::Error> {
        let path = path.as_ref();

        // It's okay, if the optional alpm-install-scriptlet does not exist, but we fail if there
        // was an error checking.
        if !path.try_exists().map_err(|source| crate::Error::IoPath {
            path: path.to_path_buf(),
            context: t!("error-io-path-check-existence"),
            source,
        })? {
            return Ok(None);
        }

        Ok(Some(Self {
            file: component_file(path, ALPM_INSTALL_SCRIPTLET_FILE_NAME)?,
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
            context: t!("error-io-path-read-file-to-string"),
            source,
        })?;
        file.rewind().map_err(|source| crate::Error::IoPath {
            path: self.path.clone(),
            context: t!("error-io-path-rewind"),
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
    /// The (read-only) file handle for the component.
    file: File,

    path: PathBuf,
}

impl MtreeComponent {
    /// Creates a new [`MtreeComponent`] from a path.
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - the file name of `path` does not match that of the component
    /// - the file at `path` cannot be opened for reading
    pub fn new(path: impl AsRef<Path>) -> Result<Self, crate::Error> {
        let path = path.as_ref();

        Ok(Self {
            file: component_file(path, ALPM_MTREE_FILE_NAME)?,
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
            context: t!("error-io-path-rewind"),
            source,
        })?;

        Ok(output)
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
    /// The file mode of an [`Entry`] directory on disk.
    pub(crate) const ENTRY_DIR_MODE: u32 = 0o040755;

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
    /// - `path` cannot be canonicalized
    /// - metadata for `path` cannot be retrieved
    /// - `path` is not a directory
    /// - `path` has non-default permissions
    /// - an [`EntryName`] cannot be created from the file name of `path`
    /// - a file for an [alpm-db-desc], [alpm-db-files] or [ALPM-MTREE] component cannot be found
    /// - the optional [alpm-install-scriptlet] component is not a file
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    /// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    /// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriplet.5.html
    pub fn new(path: impl AsRef<Path>) -> Result<Self, crate::Error> {
        let path = {
            let path = path.as_ref();
            let path = path.canonicalize().map_err(|source| crate::Error::IoPath {
                path: path.to_path_buf(),
                context: t!("error-io-path-canonicalize-entry-dir"),
                source,
            })?;
            let metadata = path.metadata().map_err(|source| crate::Error::IoPath {
                path: path.clone(),
                context: t!("error-io-path-metadata"),
                source,
            })?;
            if !metadata.is_dir() {
                return Err(Error::EntryIsNotADirectory { path }.into());
            }
            let permissions = metadata.permissions().mode();
            if permissions != Self::ENTRY_DIR_MODE {
                return Err(Error::EntryDirWrongPermissions { path, permissions }.into());
            }

            path
        };

        let name = EntryName::try_from(path.as_path())?;
        let alpm_db_desc = DescComponent::new(path.join(ALPM_DB_DESC_FILE_NAME))?;
        let alpm_db_files = FilesComponent::new(path.join(ALPM_DB_FILES_FILE_NAME))?;
        let alpm_mtree = MtreeComponent::new(path.join(ALPM_MTREE_FILE_NAME))?;
        let alpm_install_scriptlet =
            InstallComponent::new(path.join(ALPM_INSTALL_SCRIPTLET_FILE_NAME))?;

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
    /// - the [`EntryName`] of the entry directory and that of the [`DbDescFile`] component do not
    ///   match
    pub fn db_desc(&mut self) -> Result<DbDescFile, crate::Error> {
        let output = self.alpm_db_desc.to_db_desc()?;
        if output.name() != self.name.name() || output.version() != self.name.version() {
            return Err(Error::EntryComponentNameMismatch {
                entry_name: self.name.to_string(),
                component_entry_name: EntryName::new(
                    output.name().clone(),
                    output.version().clone(),
                )
                .to_string(),
            }
            .into());
        }

        Ok(output)
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

#[cfg(test)]
mod tests {
    use std::{
        fs::{Permissions, create_dir, create_dir_all, set_permissions},
        io::Write,
        os::unix::fs::OpenOptionsExt,
    };

    use alpm_types::{
        Architecture,
        ExtraData,
        ExtraDataEntry,
        PackageInstallReason,
        PackageValidation,
        RelativeFilePath,
        SystemArchitecture,
    };
    use log::{LevelFilter, debug};
    use rstest::{fixture, rstest};
    use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
    use tempfile::TempDir;
    use testresult::TestResult;

    use super::*;
    use crate::{
        desc::DbDescFileV2,
        files::{BackupEntry, DbFilesV1},
    };

    /// Initialize a logger that shows trace messages on stderr.
    fn init_logger() {
        if TermLogger::init(
            LevelFilter::Trace,
            Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        )
        .is_err()
        {
            debug!("Not initializing another logger, as one is initialized already.");
        }
    }

    /// Ensures, that [`Entry::new`] succeeds on valid input (but no data).
    #[rstest]
    #[case::with_install_file(true)]
    #[case::without_install_file(false)]
    fn entry_new_succeeds(#[case] with_install: bool) -> TestResult {
        init_logger();

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

    /// Ensures, that [`Entry::new`] fails on path not being a directory.
    #[rstest]
    fn entry_new_fails_on_path_not_a_dir() -> TestResult {
        init_logger();

        let temp_dir = TempDir::new()?;
        let entry = temp_dir.path().join("example-1.0.0-1");
        File::create(&entry)?;

        let Err(error) = Entry::new(&entry) else {
            panic!("Expected to fail on Error::EntryIsNotADirectory");
        };

        debug!("{error}");
        assert!(matches!(
            error,
            crate::Error::Database(Error::EntryIsNotADirectory { .. })
        ));

        Ok(())
    }

    /// Ensures, that [`Entry::new`] fails on path not being a directory.
    #[rstest]
    fn entry_new_fails_on_wrong_permissions() -> TestResult {
        init_logger();

        let temp_dir = TempDir::new()?;
        let entry = temp_dir.path().join("example-1.0.0-1");
        create_dir_all(&entry)?;
        set_permissions(&entry, Permissions::from_mode(0o040777))?;

        let Err(error) = Entry::new(&entry) else {
            panic!("Expected to fail on Error::EntryDirWrongPermissions");
        };

        debug!("{error}");
        assert!(matches!(
            error,
            crate::Error::Database(Error::EntryDirWrongPermissions { .. })
        ));

        Ok(())
    }

    /// Ensures, that [`Entry::new`] fails, if the desc, files or mtree component is missing.
    #[rstest]
    #[case::no_desc(vec!["files", "mtree"])]
    #[case::no_files(vec!["desc", "mtree"])]
    #[case::no_mtree(vec!["desc", "files"])]
    #[case::no_files_no_mtree(vec!["desc"])]
    #[case::no_desc_no_mtree(vec!["files"])]
    #[case::no_desc_no_files(vec!["mtree"])]
    #[case::none(Vec::new())]
    fn entry_new_fails_on_missing_component(#[case] components: Vec<&str>) -> TestResult {
        init_logger();

        let temp_dir = TempDir::new()?;
        let entry_dir = temp_dir.path().join("example-1.0.0-1");
        create_dir_all(&entry_dir)?;

        for component in components {
            File::create(entry_dir.join(component))?;
        }

        let Err(error) = Entry::new(&entry_dir) else {
            panic!("Expected to fail with Error::EntryComponentIsNotAFile");
        };

        debug!("{error}");
        assert!(matches!(error, crate::Error::IoPath { .. }));

        Ok(())
    }

    /// Ensures, that [`Entry::new`] fails, if the desc, files or mtree component is not a file.
    #[rstest]
    #[case::no_desc(vec!["files", "mtree"], vec!["desc"])]
    #[case::no_files(vec!["desc", "mtree"], vec!["files"])]
    #[case::no_mtree(vec!["desc", "files"], vec!["mtree"])]
    #[case::no_files_no_mtree(vec!["desc"], vec!["files", "mtree"])]
    #[case::no_desc_no_mtree(vec!["files"], vec!["desc", "mtree"])]
    #[case::no_desc_no_files(vec!["mtree"], vec!["desc", "files"])]
    #[case::none(Vec::new(), vec!["desc", "files", "mtree"])]
    fn entry_new_fails_on_component_is_not_a_file(
        #[case] file_components: Vec<&str>,
        #[case] dir_components: Vec<&str>,
    ) -> TestResult {
        init_logger();

        let temp_dir = TempDir::new()?;
        let entry_dir = temp_dir.path().join("example-1.0.0-1");
        create_dir_all(&entry_dir)?;

        for component in file_components {
            File::create(entry_dir.join(component))?;
        }
        for component in dir_components {
            create_dir(entry_dir.join(component))?;
        }

        let Err(error) = Entry::new(&entry_dir) else {
            panic!("Expected to fail with Error::EntryComponentIsNotAFile");
        };

        debug!("{error}");
        assert!(matches!(
            error,
            crate::Error::Database(Error::EntryComponentIsNotAFile { .. })
        ));

        Ok(())
    }

    /// Ensures, that [`Entry::new`] fails, if the desc, files or mtree component have wrong
    /// permissions.
    #[rstest]
    #[case::no_desc(vec!["files", "mtree"], vec!["desc"])]
    #[case::no_files(vec!["desc", "mtree"], vec!["files"])]
    #[case::no_mtree(vec!["desc", "files"], vec!["mtree"])]
    #[case::no_files_no_mtree(vec!["desc"], vec!["files", "mtree"])]
    #[case::no_desc_no_mtree(vec!["files"], vec!["desc", "mtree"])]
    #[case::no_desc_no_files(vec!["mtree"], vec!["desc", "files"])]
    #[case::none(Vec::new(), vec!["desc", "files", "mtree"])]
    fn entry_new_fails_on_component_has_wrong_permission(
        #[case] valid_perms: Vec<&str>,
        #[case] invalid_perms: Vec<&str>,
    ) -> TestResult {
        init_logger();

        let temp_dir = TempDir::new()?;
        let entry_dir = temp_dir.path().join("example-1.0.0-1");
        create_dir_all(&entry_dir)?;

        for component in valid_perms {
            File::create(entry_dir.join(component))?;
        }
        for component in invalid_perms {
            OpenOptions::new()
                .create_new(true)
                .write(true)
                .mode(0o600)
                .open(entry_dir.join(component))?;
        }

        let Err(error) = Entry::new(&entry_dir) else {
            panic!("Expected to fail with Error::EntryComponentWrongPermissions");
        };

        debug!("{error}");
        assert!(matches!(
            error,
            crate::Error::Database(Error::EntryComponentWrongPermissions { .. })
        ));

        Ok(())
    }

    #[fixture]
    fn db_desc() -> Result<DbDescFile, crate::Error> {
        Ok(DbDescFile::V2(DbDescFileV2 {
            name: "example".parse()?,
            version: "1.0.0-1".parse()?,
            base: "example".parse()?,
            description: "An example package".into(),
            url: Some("https://example.org".parse()?),
            arch: Architecture::Some(SystemArchitecture::X86_64),
            builddate: 1,
            installdate: 2,
            packager: "Foobar McFooface <foobar@mcfooface.org>".parse()?,
            size: 10,
            groups: Vec::new(),
            reason: PackageInstallReason::Explicit,
            license: vec!["0BSD".parse()?],
            validation: vec![PackageValidation::None],
            replaces: Vec::new(),
            depends: Vec::new(),
            optdepends: Vec::new(),
            conflicts: Vec::new(),
            provides: Vec::new(),
            xdata: ExtraData::try_from(ExtraDataEntry::new(
                "pkgtype".to_string(),
                "pkg".to_string(),
            ))?,
        }))
    }

    #[fixture]
    fn db_files() -> Result<DbFiles, crate::Error> {
        Ok(DbFiles::V1(DbFilesV1::try_from((
            vec![
                PathBuf::from("etc/"),
                PathBuf::from("etc/example"),
                PathBuf::from("etc/example/conf"),
                PathBuf::from("usr"),
                PathBuf::from("usr/bin"),
                PathBuf::from("usr/bin/foo"),
            ],
            vec![BackupEntry {
                path: RelativeFilePath::new(PathBuf::from("etc/example/conf"))?,
                md5: "d3b07384d113edec49eaa6238ad5ff00".parse()?,
            }],
        ))?))
    }

    const INSTALL_SCRIPTLET: &str = r#"!/usr/bin/env bash

post_install() {
    printf 'YOLO\n'
}
"#;

    const MTREE_DATA: &str = r#"#mtree
/set mode=644 uid=0 gid=0 type=file
./etc mode=755 type=dir time=1700000000.0
./etc/example mode=755 type=dir time=1700000000.0
./etc/example/conf time=1700000000.0 size=1337 sha256digest=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
./some_dir type=dir time=1700000000.0
./usr mode=755 type=dir time=1700000000.0
./usr/bin mode=755 type=dir time=1700000000.0
./usr/bin/foo time=1700000000.0 size=1337 sha256digest=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
"#;

    /// Ensures, that [`Entry`] can successfully return component data.
    ///
    /// This tests [`Entry::db_desc`], [`Entry::db_files`], [`Entry::install_scriptlet`],
    /// [`Entry::mtree`].
    #[rstest]
    fn entry_return_component_data(
        db_desc: Result<DbDescFile, crate::Error>,
        db_files: Result<DbFiles, crate::Error>,
    ) -> TestResult {
        init_logger();

        let temp_dir = TempDir::new()?;
        let entry_dir = temp_dir.path().join("example-1.0.0-1");
        create_dir_all(&entry_dir)?;

        let db_desc = db_desc?.to_string();
        let db_files = db_files?.to_string();
        let install_script = INSTALL_SCRIPTLET.to_string();
        let mtree_data = MTREE_DATA.to_string();

        for (file_name, data) in [
            ("desc", db_desc),
            ("files", db_files),
            ("install", install_script),
            ("mtree", mtree_data),
        ] {
            let mut file = File::create(entry_dir.join(file_name))?;
            file.write_all(&data.into_bytes())?;
        }

        Entry::new(&entry_dir)?;

        Ok(())
    }

    /// Ensures, that [`Entry::db_desc`] fails on mismatching [`EntryName`].
    #[rstest]
    fn entry_db_desc_fails_on_mismatching_entry_name(
        db_desc: Result<DbDescFile, crate::Error>,
        db_files: Result<DbFiles, crate::Error>,
    ) -> TestResult {
        init_logger();

        let temp_dir = TempDir::new()?;
        let entry_dir = temp_dir.path().join("example-foo-1.0.0-1");
        create_dir_all(&entry_dir)?;

        let db_desc = db_desc?.to_string();
        let db_files = db_files?.to_string();
        let install_script = INSTALL_SCRIPTLET.to_string();
        let mtree_data = MTREE_DATA.to_string();

        for (file_name, data) in [
            ("desc", db_desc),
            ("files", db_files),
            ("install", install_script),
            ("mtree", mtree_data),
        ] {
            let mut file = File::create(entry_dir.join(file_name))?;
            file.write_all(&data.into_bytes())?;
        }

        let mut entry = Entry::new(&entry_dir)?;

        let Err(error) = entry.db_desc() else {
            panic!("Expected to return Error::EntryComponentNameMismatch");
        };

        debug!("{error}");
        assert!(matches!(
            error,
            crate::Error::Database(Error::EntryComponentNameMismatch { .. })
        ));

        Ok(())
    }
}
