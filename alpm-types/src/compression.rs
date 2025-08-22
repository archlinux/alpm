//! File compression related types.

use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString, IntoStaticStr, VariantNames};

/// The file extension of a compression algorithm.
///
/// Compression may be used for a set of different files in the ALPM context (e.g. [alpm-package],
/// alpm-source-package, alpm-repo-db).
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

impl TryFrom<&Path> for CompressionAlgorithmFileExtension {
    type Error = crate::Error;

    /// Creates a [`CompressionAlgorithmFileExtension`] from a [`Path`] by extracting the file
    /// extension.
    ///
    /// # Errors
    ///
    /// Returns an error if the file extension does not match a
    /// [`CompressionAlgorithmFileExtension`] variant.
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| Self::from_str(ext).ok())
            .ok_or(strum::ParseError::VariantNotFound.into())
    }
}

impl TryFrom<PathBuf> for CompressionAlgorithmFileExtension {
    type Error = crate::Error;

    /// Creates a [`CompressionAlgorithmFileExtension`] from a [`PathBuf`] by extracting the file
    /// extension.
    ///
    /// Delegates to [`TryFrom<&Path>`][`TryFrom::try_from`].
    ///
    /// # Errors
    ///
    /// Returns an error if the file extension does not match a
    /// [`CompressionAlgorithmFileExtension`] variant.
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        path.as_path().try_into()
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;

    #[rstest]
    #[case("Z", CompressionAlgorithmFileExtension::Compress)]
    #[case("bz2", CompressionAlgorithmFileExtension::Bzip2)]
    #[case("gz", CompressionAlgorithmFileExtension::Gzip)]
    #[case("lrz", CompressionAlgorithmFileExtension::Lrzip)]
    #[case("lz", CompressionAlgorithmFileExtension::Lzip)]
    #[case("lz4", CompressionAlgorithmFileExtension::Lz4)]
    #[case("lzo", CompressionAlgorithmFileExtension::Lzop)]
    #[case("xz", CompressionAlgorithmFileExtension::Xz)]
    #[case("zst", CompressionAlgorithmFileExtension::Zstd)]
    fn compression_algorithm_file_extension_from_str(
        #[case] input: &str,
        #[case] expected: CompressionAlgorithmFileExtension,
    ) {
        let parsed = CompressionAlgorithmFileExtension::from_str(input).unwrap();
        assert_eq!(parsed, expected);
    }

    #[rstest]
    #[case("archive.Z", CompressionAlgorithmFileExtension::Compress)]
    #[case("data.bz2", CompressionAlgorithmFileExtension::Bzip2)]
    #[case("doc.gz", CompressionAlgorithmFileExtension::Gzip)]
    #[case("video.lrz", CompressionAlgorithmFileExtension::Lrzip)]
    #[case("binary.lz", CompressionAlgorithmFileExtension::Lzip)]
    #[case("dump.lz4", CompressionAlgorithmFileExtension::Lz4)]
    #[case("image.lzo", CompressionAlgorithmFileExtension::Lzop)]
    #[case("package.xz", CompressionAlgorithmFileExtension::Xz)]
    #[case("/var/cache/repo.zst", CompressionAlgorithmFileExtension::Zstd)]
    fn compression_algorithm_file_extension_try_from_path(
        #[case] filename: &str,
        #[case] expected: CompressionAlgorithmFileExtension,
    ) -> testresult::TestResult {
        let path = PathBuf::from(filename);
        let parsed = CompressionAlgorithmFileExtension::try_from(path)?;
        assert_eq!(parsed, expected);
        Ok(())
    }

    #[rstest]
    #[case("file.txt")]
    #[case("unknown.abc")]
    #[case("noext")]
    fn invalid_compression_file_extension(#[case] filename: &str) {
        let path = Path::new(filename);
        let error = CompressionAlgorithmFileExtension::try_from(path).unwrap_err();
        assert!(matches!(error, crate::Error::InvalidVariant(_)));
    }
}
