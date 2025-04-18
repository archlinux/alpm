//! Compression related types.

use std::str::FromStr;

use crate::PackageError;

/// The supported compression type for a package file.
///
/// A compression type is usually used as a file suffix in a package name and indicates the
/// compression algorithm in use.
#[derive(
    strum::AsRefStr,
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    strum::Display,
    strum::IntoStaticStr,
    PartialEq,
)]
pub enum CompressionType {
    /// The package is compressed using the zstandard compression algorithm.
    #[strum(to_string = "bz2")]
    Bzip2,

    /// The package is compressed using the gzip compression algorithm.
    #[strum(to_string = "gz")]
    Gzip,

    /// The package is not compressed.
    #[strum(to_string = "")]
    None,

    /// The package is compressed using the XZ compression algorithm.
    #[strum(to_string = "xz")]
    Xz,

    /// The package is compressed using the zstandard compression algorithm.
    #[default]
    #[strum(to_string = "zst")]
    Zstandard,
}

impl FromStr for CompressionType {
    type Err = crate::Error;

    /// Creates a [`CompressionType`] from a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if `s` does not match the string representation of any variant of
    /// [`CompressionType`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "" => CompressionType::None,
            "bz2" => CompressionType::Bzip2,
            "gz" => CompressionType::Gzip,
            "xz" => CompressionType::Xz,
            "zst" => CompressionType::Zstandard,
            _ => {
                return Err(PackageError::InvalidCompressionType {
                    compression_type: s.to_string(),
                }
                .into());
            }
        })
    }
}
