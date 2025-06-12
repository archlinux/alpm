//! Functions for ALPM-MTREE file creation.
//!
//! [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html

use std::path::{Path, PathBuf};

use crate::file::common::{BsdtarOptions, create_mtree_file_from_input_dir};

/// Creates a new [ALPM-MTREEv1] file from an input directory and returns its path.
///
/// Calls [`create_mtree_file_from_input_dir`] with [bsdtar] options specific to [ALPM-MTREEv1].
///
/// # Errors
///
/// Returns an error if calling [`create_mtree_file_from_input_dir`] fails.
///
/// [ALPM-MTREEv1]: https://alpm.archlinux.page/specifications/ALPM-MTREEv1.5.html
/// [bsdtar]: https://man.archlinux.org/man/bsdtar.1
pub fn create_mtree_v1_from_input_dir(path: impl AsRef<Path>) -> Result<PathBuf, crate::Error> {
    create_mtree_file_from_input_dir(path, BsdtarOptions::MtreeV1)
}

/// Creates a new [ALPM-MTREEv2] file from an input directory and returns its path.
///
/// Calls [`create_mtree_file_from_input_dir`] with [bsdtar] options specific to [ALPM-MTREEv2].
///
/// # Errors
///
/// Returns an error if calling [`create_mtree_file_from_input_dir`] fails.
///
/// [ALPM-MTREEv2]: https://alpm.archlinux.page/specifications/ALPM-MTREEv2.5.html
/// [bsdtar]: https://man.archlinux.org/man/bsdtar.1
pub fn create_mtree_v2_from_input_dir(path: impl AsRef<Path>) -> Result<PathBuf, crate::Error> {
    create_mtree_file_from_input_dir(path, BsdtarOptions::MtreeV2)
}
