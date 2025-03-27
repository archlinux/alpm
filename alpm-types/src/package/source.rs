//! Types related to package sources.

/// The name of a [PKGBUILD] file in a package source repository.
///
/// [PKGBUILD]: https://man.archlinux.org/man/PKGBUILD.5
pub const PKGBUILD_FILE_NAME: &str = "PKGBUILD";

/// The name of a [SRCINFO] file in a package source repository.
///
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
pub const SRCINFO_FILE_NAME: &str = ".SRCINFO";
