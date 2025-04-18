//! File compression related types.

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString, IntoStaticStr, VariantNames};

/// The file extension of a compression algorithm.
///
/// Compression may be used for a set of different files in the ALPM context (e.g. [alpm-package],
/// alpm-source-package, alpm-repo-database).
/// Each algorithm uses a distinct file extension.
///
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Display,
    EnumString,
    Eq,
    IntoStaticStr,
    PartialEq,
    Serialize,
    VariantNames,
)]
#[serde(untagged)]
pub enum CompressionAlgorithmFileExtension {
    /// The file extension for files compressed using the [compress] compression algorithm.
    ///
    /// [compress]: https://man.archlinux.org/man/compress.1
    #[serde(rename = "Z")]
    #[strum(to_string = "Z")]
    Compress,

    /// The file extension for files compressed using the [bzip2] compression algorithm.
    ///
    /// [bzip2]: https://man.archlinux.org/man/bzip2.1
    #[serde(rename = "bz2")]
    #[strum(to_string = "bz2")]
    Bzip2,

    /// The file extension for files compressed using the [gzip] compression algorithm.
    ///
    /// [gzip]: https://man.archlinux.org/man/gzip.1
    #[serde(rename = "gz")]
    #[strum(to_string = "gz")]
    Gzip,

    /// The file extension for files compressed using the [lrzip] compression algorithm.
    ///
    /// [lrzip]: https://man.archlinux.org/man/lrzip.1
    #[serde(rename = "lrz")]
    #[strum(to_string = "lrz")]
    Lrzip,

    /// The file extension for files compressed using the [lzip] compression algorithm.
    ///
    /// [lzip]: https://man.archlinux.org/man/lzip.1
    #[serde(rename = "lz")]
    #[strum(to_string = "lz")]
    Lzip,

    /// The file extension for files compressed using the [lz4] compression algorithm.
    ///
    /// [lz4]: https://man.archlinux.org/man/lz4.1
    #[serde(rename = "lz4")]
    #[strum(to_string = "lz4")]
    Lz4,

    /// The file extension for files compressed using the [lzop] compression algorithm.
    ///
    /// [lzop]: https://man.archlinux.org/man/lzop.1
    #[serde(rename = "lzo")]
    #[strum(to_string = "lzo")]
    Lzop,

    /// The file extension for files compressed using the [xz] compression algorithm.
    ///
    /// [xz]: https://man.archlinux.org/man/xz.1
    #[serde(rename = "xz")]
    #[strum(to_string = "xz")]
    Xz,

    /// The file extension for files compressed using the [zstd] compression algorithm.
    ///
    /// [zstd]: https://man.archlinux.org/man/zstd.1
    #[default]
    #[serde(rename = "zst")]
    #[strum(to_string = "zst")]
    Zstd,
}
