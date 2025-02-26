//! Package compression.

use alpm_types::CompressionType;

#[cfg(doc)]
use crate::package::Package;

/// Bzip2 compression level.
///
/// The default adheres to the one selected by the [bzip2] executable.
///
/// [bzip2]: https://man.archlinux.org/man/bzip2.1
#[derive(Clone, Debug, Default, Eq, strum::Display, num_enum::IntoPrimitive, PartialEq)]
#[repr(u8)]
pub enum Bzip2Compression {
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
    Level4 = 4,
    Level5 = 5,
    Level6 = 6,
    Level7 = 7,
    Level8 = 8,
    #[default]
    Level9 = 9,
}

impl From<Bzip2Compression> for bzip2::Compression {
    /// Creates a [`bzip2::Compression`] from a [`Bzip2Compression`].
    fn from(value: Bzip2Compression) -> Self {
        let level: u8 = value.into();
        bzip2::Compression::new(level.into())
    }
}

/// Gzip compression level.
///
/// The default adheres to the one selected by the [gzip] executable.
///
/// [gzip]: https://man.archlinux.org/man/gzip.1
#[derive(
    strum::AsRefStr,
    Clone,
    Debug,
    Default,
    Eq,
    strum::Display,
    strum::IntoStaticStr,
    PartialEq,
    num_enum::IntoPrimitive,
)]
#[repr(u8)]
pub enum GzipCompression {
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
    Level4 = 4,
    Level5 = 5,
    #[default]
    Level6 = 6,
    Level7 = 7,
    Level8 = 8,
    Level9 = 9,
}

impl From<GzipCompression> for flate2::Compression {
    /// Creates a [`flate2::Compression`] from a [`GzipCompression`].
    fn from(value: GzipCompression) -> Self {
        let level: u8 = value.into();
        Self::new(level.into())
    }
}

/// Xz compression level.
///
/// The default adheres to the one selected by the [xz] executable.
///
/// [xz]: https://man.archlinux.org/man/xz.1
#[derive(
    strum::AsRefStr,
    Clone,
    Debug,
    Default,
    Eq,
    strum::Display,
    strum::IntoStaticStr,
    PartialEq,
    num_enum::IntoPrimitive,
)]
#[repr(u8)]
pub enum XzCompression {
    Level0 = 0,
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
    Level4 = 4,
    Level5 = 5,
    #[default]
    Level6 = 6,
    Level7 = 7,
    Level8 = 8,
    Level9 = 9,
}

impl From<XzCompression> for u32 {
    /// Creates a [`u32`] from an [`XzCompression`].
    fn from(value: XzCompression) -> Self {
        let level: u8 = value.into();
        Self::from(level)
    }
}

/// Zstandard compression level.
///
/// The default adheres to the one selected by the [zstd] executable.
///
/// [zstd]: https://man.archlinux.org/man/zstd.1
#[derive(
    strum::AsRefStr,
    Clone,
    Debug,
    Default,
    Eq,
    strum::Display,
    strum::IntoStaticStr,
    PartialEq,
    num_enum::IntoPrimitive,
)]
#[repr(u8)]
pub enum ZstandardCompression {
    Level0 = 0,
    Level1 = 1,
    Level2 = 2,
    #[default]
    Level3 = 3,
    Level4 = 4,
    Level5 = 5,
    Level6 = 6,
    Level7 = 7,
    Level8 = 8,
    Level9 = 9,
    Level10 = 10,
    Level11 = 11,
    Level12 = 12,
    Level13 = 13,
    Level14 = 14,
    Level15 = 15,
    Level16 = 16,
    Level17 = 17,
    Level18 = 18,
    Level19 = 19,
    Level20 = 20,
    Level21 = 21,
    Level22 = 22,
}

impl From<ZstandardCompression> for i32 {
    fn from(value: ZstandardCompression) -> Self {
        let level: u8 = value.into();
        Self::from(level)
    }
}

/// The settings for the compression algorithm used for a [`Package`].
#[derive(strum::AsRefStr, Clone, Debug, Eq, strum::Display, strum::IntoStaticStr, PartialEq)]
pub enum CompressionSettings {
    /// The package is compressed using the zstandard compression algorithm.
    #[strum(to_string = "bz2")]
    Bzip2 {
        /// The used compression level.
        compression_level: Bzip2Compression,
    },

    /// The package is compressed using the gzip compression algorithm.
    #[strum(to_string = "gz")]
    Gzip {
        /// The used compression level.
        compression_level: GzipCompression,
    },

    /// The package is not compressed.
    #[strum(to_string = "")]
    None,

    /// The package is compressed using the XZ compression algorithm.
    #[strum(to_string = "xz")]
    Xz {
        /// The used compression level.
        compression_level: XzCompression,
    },

    /// The package is compressed using the zstandard compression algorithm.
    #[strum(to_string = "zst")]
    Zstandard {
        /// The used compression level.
        compression_level: ZstandardCompression,
    },
}

impl Default for CompressionSettings {
    /// Returns [`CompressionSettings::Zstandard`] using the [`ZstandardCompression::default`].
    fn default() -> Self {
        Self::Zstandard {
            compression_level: ZstandardCompression::default(),
        }
    }
}

impl From<CompressionSettings> for CompressionType {
    /// Creates a [`CompressionType`] from a [`CompressionSettings`].
    fn from(value: CompressionSettings) -> Self {
        match value {
            CompressionSettings::Bzip2 { .. } => CompressionType::Bzip2,
            CompressionSettings::Gzip { .. } => CompressionType::Gzip,
            CompressionSettings::None => CompressionType::None,
            CompressionSettings::Xz { .. } => CompressionType::Xz,
            CompressionSettings::Zstandard { .. } => CompressionType::Zstandard,
        }
    }
}
