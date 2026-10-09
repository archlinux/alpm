//! Traits relevant for interfacing with an [alpm-db].
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::{
    fmt::Debug,
    fs::{File, OpenOptions, Permissions, create_dir, set_permissions},
    io::Write,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::Path,
};

use alpm_mtree::mtree::MtreeTransfer;
use alpm_types::EntryName;
use fluent_i18n::t;
use log::debug;

use crate::{
    db::{
        Entry,
        Error,
        entry::{
            ALPM_DB_DESC_FILE_NAME,
            ALPM_DB_FILES_FILE_NAME,
            ALPM_INSTALL_SCRIPTLET_FILE_NAME,
            ALPM_MTREE_FILE_NAME,
        },
    },
    desc::DbDescFile,
    files::DbFiles,
};

/// Creates a new file handle for writing with file mode compatible to alpm-db entry components.
fn create_entry_component(path: impl AsRef<Path>) -> Result<File, crate::Error> {
    let path = path.as_ref();
    debug!("Creating alpm-db entry component {path:?}");

    OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o644)
        .open(path)
        .map_err(|source| crate::Error::IoPath {
            path: path.to_path_buf(),
            context: t!("error-io-path-db-create-component"),
            source,
        })
}

/// An interface for writing an entry to an [alpm-db].
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
pub trait WritableEntry: Debug {
    /// Returns a reference to the [`EntryName`] of the implementation.
    fn entry_name(&self) -> &EntryName;

    /// Returns the [alpm-db-desc] data of the implementation.
    ///
    /// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    fn alpm_db_desc(&self) -> Result<DbDescFile, crate::Error>;

    /// Returns the [alpm-db-files] data of the implementation.
    ///
    /// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
    fn alpm_db_files(&self) -> Result<DbFiles, crate::Error>;

    /// Returns the transferable [ALPM-MTREE] data of the implementation.
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    fn alpm_mtree(&self) -> Result<MtreeTransfer, crate::Error>;

    /// Returns the optional [alpm-install-scriptlet] data of the implementation.
    ///
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    fn alpm_install_scriptlet(&self) -> Result<Option<String>, crate::Error>;

    /// Writes the entry to the root directory of an [alpm-db].
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - the [alpm-db-desc] data of the [`WritableEntry`] implementation cannot be retrieved
    /// - the [alpm-db-files] data of the [`WritableEntry`] implementation cannot be retrieved
    /// - the [ALPM-MTREE] data of the [`WritableEntry`] implementation cannot be retrieved
    /// - an error occurs while attempting to retrieve the optional [alpm-install-scriptlet] data
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    /// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    /// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    fn write_to_db_root(&self, path: &Path) -> Result<(), crate::Error> {
        debug!("Load alpm-db-desc");
        let alpm_db_desc = self.alpm_db_desc()?;
        debug!("Load alpm-db-files");
        let alpm_db_files = self.alpm_db_files()?;
        debug!("Load alpm-install-scriptlet");
        let alpm_install_scriptlet = self.alpm_install_scriptlet()?;
        debug!("Load alpm-mtree");
        let alpm_mtree = self.alpm_mtree()?;

        let entry_dir = path.join(self.entry_name().to_string());
        let desc = entry_dir.join(ALPM_DB_DESC_FILE_NAME);
        let files = entry_dir.join(ALPM_DB_FILES_FILE_NAME);
        let mtree = entry_dir.join(ALPM_MTREE_FILE_NAME);
        let install_scriptlet = entry_dir.join(ALPM_INSTALL_SCRIPTLET_FILE_NAME);

        if entry_dir.exists() {
            return Err(Error::EntryExists {
                name: self.entry_name().clone(),
            }
            .into());
        }
        create_dir(&entry_dir).map_err(|source| crate::Error::IoPath {
            path: entry_dir.clone(),
            context: t!("error-io-path-entry-dir-create"),
            source,
        })?;
        set_permissions(&entry_dir, Permissions::from_mode(Entry::ENTRY_DIR_MODE)).map_err(
            |source| crate::Error::IoPath {
                path: entry_dir.clone(),
                context: t!("error-io-path-db-set-permissions-on-dir"),
                source,
            },
        )?;

        let components = {
            let mut components = vec![
                (
                    create_entry_component(desc)?,
                    alpm_db_desc.to_string().into_bytes(),
                ),
                (
                    create_entry_component(files)?,
                    alpm_db_files.to_string().into_bytes(),
                ),
                (create_entry_component(mtree)?, alpm_mtree.data()),
            ];

            if let Some(alpm_install_scriptlet) = alpm_install_scriptlet {
                components.push((
                    create_entry_component(install_scriptlet)?,
                    alpm_install_scriptlet.into_bytes(),
                ))
            }

            components
        };

        for (mut file, data) in components {
            file.write_all(&data)
                .map_err(|source| crate::Error::IoPath {
                    path: entry_dir.clone(),
                    context: t!("error-io-path-db-write-component"),
                    source,
                })?
        }

        Ok(())
    }
}
