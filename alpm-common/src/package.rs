//! Data related to package file contents.

/// The name of an [alpm-install-scriptlet] file in an [alpm-package].
///
/// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
pub const INSTALL_SCRIPTLET_FILENAME: &str = ".INSTALL";

/// The name of a required metadata file in an [alpm-package].
///
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
#[derive(Clone, Copy, Debug, strum::Display, Eq, PartialEq)]
pub enum MetadataFileName {
    /// The [BUILDINFO] file.
    ///
    /// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
    #[strum(to_string = ".BUILDINFO")]
    BuildInfo,

    /// The [ALPM-MTREE] file.
    ///
    /// [ALPM-MTREE]: ahttps://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    #[strum(to_string = ".MTREE")]
    Mtree,

    /// The [PKGINFO] file.
    ///
    /// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
    #[strum(to_string = ".PKGINFO")]
    PackageInfo,
}
