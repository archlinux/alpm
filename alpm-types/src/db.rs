//! Metadata file names for [alpm-db] entries.
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

/// File name of the [alpm-db-desc] metadata in an entry directory.
///
/// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
pub const DESC_FILE_NAME: &str = "desc";

/// File name of the [alpm-db-files] metadata in an entry directory.
///
/// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
pub const FILES_FILE_NAME: &str = "files";

/// File name of the [alpm-mtree] metadata in an entry directory.
///
/// [alpm-mtree]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
pub const MTREE_FILE_NAME: &str = "mtree";
