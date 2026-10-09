//! Handling of files contained in an [alpm-db].
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::{collections::HashSet, path::Path};

use alpm_types::EntryName;
use strum::AsRefStr;

/// The component of an entry in the [alpm-db].
///
/// A component represents a specific file and file type in the scope of an entry.
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
#[derive(AsRefStr, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EntryComponentFileName {
    /// The [alpm-db-desc] file contained in the database.
    ///
    /// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    #[strum(serialize = "desc")]
    Desc,

    /// The [alpm-db-files] file contained in the database.
    ///
    /// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
    #[strum(serialize = "files")]
    Files,

    /// The _optional_ [alpm-install-scriptlet] file contained in the database.
    ///
    /// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
    #[strum(serialize = "install")]
    InstallScriptlet,

    /// The [ALPM-MTREE] file contained in the database.
    ///
    /// [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    #[strum(serialize = "mtree")]
    Mtree,
}

/// The representation of an entry in an [alpm-db].
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Entry {
    /// The name of the entry.
    name: EntryName,

    /// The filenames of the components contained in the entry.
    components: Vec<EntryComponentFileName>,
}

impl Entry {
    /// Creates a new [`Entry`].
    pub fn new(name: EntryName, components: HashSet<EntryComponentFileName>) -> Self {
        Self {
            name,
            components: Vec::from_iter(components),
        }
    }
}

impl TryFrom<&Path> for Entry {
    type Error = crate::Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let name = EntryName::try_from(value)?;
        // let components =
        Ok(Self {
            name,
            components: Vec::new(),
        })
    }
}
