//! Settings for a compression encoder.

use alpm_types::CompressionAlgorithmFileExtension;

use crate::compression::{
    Bzip2CompressionLevel,
    GzipCompressionLevel,
    XzCompressionLevel,
    ZstdCompressionLevel,
};

/// The amount of threads to use when compressing using zstd.
///
/// The default (1) adheres to the one selected by the [zstd] executable.
/// If the custom amount of `0` is used, all available physical CPU cores are used.
///
/// [zstd]: https://man.archlinux.org/man/zstd.1
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ZstdThreads(pub(crate) u32);

impl ZstdThreads {
    /// Creates a new [`ZstdThreads`] from a [`u32`].
    pub fn new(threads: u32) -> Self {
        Self(threads)
    }

    /// Creates a new [`ZstdThreads`] which uses all physical CPU cores.
    ///
    /// This is short for calling [`ZstdThreads::new`] with `0`.
    pub fn all() -> Self {
        Self(0)
    }
}

impl Default for ZstdThreads {
    /// Returns the default thread value (1) when compressing with zstd.
    ///
    /// The default adheres to the one selected by the [zstd] executable.
    ///
    /// [zstd]: https://man.archlinux.org/man/zstd.1
    fn default() -> Self {
        Self(1)
    }
}

/// Settings for a compression encoder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompressionSettings {
    /// Settings for the bzip2 compression algorithm.
    Bzip2 {
        /// The used compression level.
        compression_level: Bzip2CompressionLevel,
    },

    /// Settings for the gzip compression algorithm.
    Gzip {
        /// The used compression level.
        compression_level: GzipCompressionLevel,
    },

    /// Settings for the xz compression algorithm.
    Xz {
        /// The used compression level.
        compression_level: XzCompressionLevel,
    },

    /// Settings for the zstandard compression algorithm.
    Zstd {
        /// The used compression level.
        compression_level: ZstdCompressionLevel,
        /// The amount of threads to use when compressing.
        threads: ZstdThreads,
    },
}

impl Default for CompressionSettings {
    /// Returns [`CompressionSettings::Zstd`].
    ///
    /// Defaults for `compression_level` and `threads` follow that of the [zstd] executable.
    ///
    /// [zstd]: https://man.archlinux.org/man/zstd.1
    fn default() -> Self {
        Self::Zstd {
            compression_level: ZstdCompressionLevel::default(),
            threads: ZstdThreads::default(),
        }
    }
}

impl From<&CompressionSettings> for CompressionAlgorithmFileExtension {
    /// Creates a [`CompressionAlgorithmFileExtension`] from a [`CompressionSettings`].
    fn from(value: &CompressionSettings) -> Self {
        match value {
            CompressionSettings::Bzip2 { .. } => CompressionAlgorithmFileExtension::Bzip2,
            CompressionSettings::Gzip { .. } => CompressionAlgorithmFileExtension::Gzip,
            CompressionSettings::Xz { .. } => CompressionAlgorithmFileExtension::Xz,
            CompressionSettings::Zstd { .. } => CompressionAlgorithmFileExtension::Zstd,
        }
    }
}

#[cfg(test)]
mod tests {
    use testresult::TestResult;

    use super::*;

    /// Ensures that the default [`CompressionSettings`] are those for zstd.
    #[test]
    fn default_compression_settings() -> TestResult {
        assert!(matches!(
            CompressionSettings::default(),
            CompressionSettings::Zstd {
                compression_level: _,
                threads: _,
            }
        ));
        Ok(())
    }
}
