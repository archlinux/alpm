//! File type handling.

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString, IntoStaticStr};

/// The identifier of a file type used in ALPM.
///
/// These identifiers are used in the file names of file types such as binary packages (see
/// [alpm-package]), source packages and repository sync databases (see alpm-repo-db).
///
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumString,
    Eq,
    IntoStaticStr,
    PartialEq,
    Serialize,
)]
#[serde(untagged)]
pub enum FileTypeIdentifier {
    /// The identifier for [alpm-package] files.
    ///
    /// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
    #[serde(rename = "pkg")]
    #[strum(to_string = "pkg")]
    BinaryPackage,

    /// The identifier for alpm-repo-db files.
    #[serde(rename = "db")]
    #[strum(to_string = "db")]
    RepositorySyncDatabase,

    /// The identifier for source package files.
    #[serde(rename = "src")]
    #[strum(to_string = "src")]
    SourcePackage,
}
