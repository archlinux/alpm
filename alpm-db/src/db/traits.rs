//! Traits relevant for interfacing with an [alpm-db].
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::fmt::Debug;

use alpm_mtree::mtree::MtreeTransfer;
use alpm_types::EntryName;

use crate::{desc::DbDescFile, files::DbFiles};

/// An interface for retrieving data for an entry that should be persisted in an [alpm-db].
///
/// An [`Entry`][crate::db::Entry] consists of a directory and several component files.
/// As such, the [`EntrySource`] interface provides access to all relevant data to create an entry
/// in an [alpm-db]:
///
/// - the name of the entry
/// - the [alpm-db-desc] data
/// - the [alpm-db-files] data
/// - the [ALPM-MTREE] file
/// - the optional [alpm-install-scriptlet] data
///
/// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
/// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
/// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
/// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
pub trait EntrySource: Debug {
    /// Returns a reference to the [`EntryName`] of the implementation.
    fn entry_name(&self) -> EntryName;

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
}
