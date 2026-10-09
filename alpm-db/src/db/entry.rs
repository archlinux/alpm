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
use log::warn;
use strum::AsRefStr;

use crate::{
    db::{Error, permissions_world_writable},
    desc::DbDescFile,
    files::DbFiles,
};

/// The type of an entry component.
#[derive(AsRefStr, Clone, Copy, Debug, strum::Display, Eq, Ord, PartialEq, PartialOrd)]
pub enum EntryComponentType {
    /// The [alpm-db-desc] component in a database entry.
    ///
    /// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    #[strum(serialize = "desc")]
    DbDesc,

    /// The [alpm-db-files] component in a database entry.
    ///
    /// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
    #[strum(serialize = "files")]
    DbFiles,

    /// The optional [alpm-install-scriptlet] component in a database entry.
    ///
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    #[strum(serialize = "install")]
    InstallScriptlet,

    /// The [ALPM-MTREE] component in a database entry.
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    #[strum(serialize = "mtree")]
    Mtree,
}

impl EntryComponentType {
    /// Returns the name of the file type.
    pub fn file_type_name(&self) -> &str {
        match self {
            Self::DbDesc => "alpm-desc-desc",
            Self::DbFiles => "alpm-db-files",
            Self::Mtree => "ALPM-MTREE",
            Self::InstallScriptlet => "alpm-install-scriptlet",
        }
    }
}

/// A component file in an [`Entry`].
///
/// Each component file is tracked using a read-only [`File`] handle and the path available upon
/// first access.
#[derive(Debug)]
pub(crate) struct EntryComponent {
    /// The (read-only) file handle for the component.
    file: File,

    /// The path to the component during creation.
    path: PathBuf,
}

impl EntryComponent {
    /// The default permissions for a component of an [alpm-db] entry.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    pub const FILE_MODE: u32 = 0o100644;

    /// Creates a new entry component.
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - the metadata for `path` cannot be retrieved
    /// - `path` is not a file
    /// - the file permissions of `path` do not match that of [`Self::FILE_MODE`]
    /// - the file at `path` cannot be opened for reading
    pub fn new(
        entry_dir: impl AsRef<Path>,
        name: EntryComponentType,
    ) -> Result<Self, crate::Error> {
        let path = entry_dir.as_ref().join(name.as_ref());

        let metadata = path
            .symlink_metadata()
            .map_err(|source| crate::Error::IoPath {
                path: path.clone(),
                context: t!("error-io-path-metadata"),
                source,
            })?;

        if !metadata.is_file() {
            return Err(Error::EntryComponentIsNotAFile { path: path.clone() }.into());
        }

        // Emit a warning if the file is world-writable.
        if permissions_world_writable(metadata.permissions().mode()) {
            warn!(
                "{}",
                t!("warn-database-entry-component-world-writable", { "path" => path.display().to_string() })
            );
        }

        let file = OpenOptions::new()
            .read(true)
            .open(&path)
            .map_err(|source| crate::Error::IoPath {
                path: path.clone(),
                context: t!("error-io-path-open-read-only"),
                source,
            })?;

        Ok(Self {
            file,
            path: path.to_path_buf(),
        })
    }
}

/// The representation of an entry in an [alpm-db].
///
/// Each entry is represented by a directory, containing a set of file components.
/// File components are distinguished between mandatory ones ([ALPM-MTREE], [alpm-db-desc] and
/// [alpm-db-files]) and optional ones ([alpm-install-scriptlet]).
///
/// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
/// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
/// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
/// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
#[derive(Debug)]
pub struct Entry {
    /// The name of the entry.
    name: EntryName,

    // Access to the alpm-db-desc component.
    alpm_db_desc: EntryComponent,

    // Access to the alpm-db-files component.
    alpm_db_files: EntryComponent,

    // Access to the alpm-install-scriptlet component.
    alpm_install_scriptlet: Option<EntryComponent>,

    // Access to the ALPM-MTREE component.
    alpm_mtree: EntryComponent,
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

            // Emit a warning if the directory is world-writable.
            if permissions_world_writable(metadata.permissions().mode()) {
                warn!(
                    "{}",
                    t!("warn-database-entry-world-writable", { "path" => path.display().to_string() })
                );
            }

            path
        };

        let name = EntryName::try_from(path.as_path())?;
        let alpm_db_desc = EntryComponent::new(&path, EntryComponentType::DbDesc)?;
        let alpm_db_files = EntryComponent::new(&path, EntryComponentType::DbFiles)?;
        let alpm_mtree = EntryComponent::new(&path, EntryComponentType::Mtree)?;

        // It's okay, if the optional alpm-install-scriptlet does not exist, but we fail if there
        // was an error checking.
        let alpm_install_scriptlet = if path
            .join(EntryComponentType::InstallScriptlet.as_ref())
            .try_exists()
            .map_err(|source| crate::Error::IoPath {
                path: path.to_path_buf(),
                context: t!("error-io-path-check-existence"),
                source,
            })? {
            Some(EntryComponent::new(
                &path,
                EntryComponentType::InstallScriptlet,
            )?)
        } else {
            None
        };

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
        let mut file = &self.alpm_db_desc.file;
        let output = DbDescFile::from_reader(file)?;
        file.rewind().map_err(|source| crate::Error::IoPath {
            path: self.alpm_db_desc.path.clone(),
            context: t!("error-io-path-rewind"),
            source,
        })?;

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
        let mut file = &self.alpm_db_files.file;
        let output = DbFiles::from_reader(file)?;
        file.rewind().map_err(|source| crate::Error::IoPath {
            path: self.alpm_db_files.path.clone(),
            context: t!("error-io-path-rewind"),
            source,
        })?;

        Ok(output)
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
        let Some(install_scriptlet) = self.alpm_install_scriptlet.take() else {
            return Ok(None);
        };
        let mut file = install_scriptlet.file;
        let output = read_to_string(&file).map_err(|source| crate::Error::IoPath {
            path: install_scriptlet.path.to_path_buf(),
            context: t!("error-io-path-read-file-to-string"),
            source,
        })?;
        file.rewind().map_err(|source| crate::Error::IoPath {
            path: install_scriptlet.path.clone(),
            context: t!("error-io-path-rewind"),
            source,
        })?;

        Ok(Some(output))
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
        let mut file = &self.alpm_mtree.file;
        let output = Mtree::from_reader(file)?;
        file.rewind().map_err(|source| crate::Error::IoPath {
            path: self.alpm_mtree.path.clone(),
            context: t!("error-io-path-rewind"),
            source,
        })?;

        Ok(output)
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
        fs::{create_dir, create_dir_all},
        io::Write,
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

        let mut entry = Entry::new(&entry_dir)?;

        let db_desc = entry.db_desc()?;
        debug!("alpm-db-desc:\n{db_desc}");

        let db_files = entry.db_files()?;
        debug!("alpm-db-files:\n{db_files}");

        let mtree = entry.mtree()?;
        debug!("ALPM-MTREE:\n{mtree}");

        if let Some(install_scriptlet) = entry.install_scriptlet()? {
            debug!("alpm-install-scriptlet:\n{install_scriptlet}");
        }

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
