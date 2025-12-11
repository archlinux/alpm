//! Settings for a compression decoder.

use std::{fmt::Debug, path::Path};

use alpm_types::CompressionAlgorithmFileExtension;

use crate::{compression::CompressionSettings, error::Error};

/// Settings for a compression decoder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecompressionSettings {
    /// The bzip2 compression algorithm.
    Bzip2,
    /// The gzip compression algorithm.
    Gzip,
    /// The xz compression algorithm.
    Xz,
    /// The zstandard compression algorithm.
    Zstd,
    /// No compression.
    None,
}

impl TryFrom<CompressionAlgorithmFileExtension> for DecompressionSettings {
    type Error = Error;

    /// Converts a [`CompressionAlgorithmFileExtension`] into a [`DecompressionSettings`].
    fn try_from(value: CompressionAlgorithmFileExtension) -> Result<Self, Self::Error> {
        match value {
            CompressionAlgorithmFileExtension::Bzip2 => Ok(Self::Bzip2),
            CompressionAlgorithmFileExtension::Gzip => Ok(Self::Gzip),
            CompressionAlgorithmFileExtension::Xz => Ok(Self::Xz),
            CompressionAlgorithmFileExtension::Zstd => Ok(Self::Zstd),
            _ => Err(Error::UnsupportedCompressionAlgorithm {
                value: value.to_string(),
            }),
        }
    }
}

impl TryFrom<&Path> for DecompressionSettings {
    type Error = Error;

    /// Converts a [`Path`] into a [`DecompressionSettings`] by extracting the file extension.
    ///
    /// Delegates deducing the compression algorithm to
    /// [`CompressionAlgorithmFileExtension::try_from`] and the final conversion to
    /// [`TryFrom<CompressionAlgorithmFileExtension>`][`TryFrom::try_from`].
    ///
    /// # Errors
    ///
    /// Returns an error if the compression algorithm file extension is unknown or unsupported.
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let ext = CompressionAlgorithmFileExtension::try_from(path)
            .map_err(Error::UnknownCompressionAlgorithmFileExtension)?;
        DecompressionSettings::try_from(ext)
    }
}

impl From<&CompressionSettings> for DecompressionSettings {
    /// Converts a [`CompressionSettings`] into a [`DecompressionSettings`].
    fn from(value: &CompressionSettings) -> Self {
        match value {
            CompressionSettings::Bzip2 { .. } => DecompressionSettings::Bzip2,
            CompressionSettings::Gzip { .. } => DecompressionSettings::Gzip,
            CompressionSettings::Xz { .. } => DecompressionSettings::Xz,
            CompressionSettings::Zstd { .. } => DecompressionSettings::Zstd,
            CompressionSettings::None => DecompressionSettings::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use testresult::TestResult;

    use super::*;
    use crate::compression::{
        Bzip2CompressionLevel,
        GzipCompressionLevel,
        XzCompressionLevel,
        ZstdCompressionLevel,
        ZstdThreads,
    };

    /// Ensures that the conversion from [`CompressionAlgorithmFileExtension`] to
    /// [`DecompressionSettings`] works as expected.
    #[rstest]
    #[case(CompressionAlgorithmFileExtension::Bzip2, DecompressionSettings::Bzip2)]
    #[case(CompressionAlgorithmFileExtension::Gzip, DecompressionSettings::Gzip)]
    #[case(CompressionAlgorithmFileExtension::Xz, DecompressionSettings::Xz)]
    #[case(CompressionAlgorithmFileExtension::Zstd, DecompressionSettings::Zstd)]
    fn test_decompression_settings_conversion_success(
        #[case] ext: CompressionAlgorithmFileExtension,
        #[case] expected: DecompressionSettings,
    ) -> TestResult {
        let result = DecompressionSettings::try_from(ext)?;
        assert_eq!(result, expected);
        Ok(())
    }

    /// Ensures that the conversion from [`CompressionAlgorithmFileExtension`] to
    /// [`DecompressionSettings`] fails as expected for unsupported algorithms.
    #[rstest]
    #[case(CompressionAlgorithmFileExtension::Compress, "Z")]
    #[case(CompressionAlgorithmFileExtension::Lrzip, "lrz")]
    #[case(CompressionAlgorithmFileExtension::Lzip, "lz")]
    #[case(CompressionAlgorithmFileExtension::Lz4, "lz4")]
    #[case(CompressionAlgorithmFileExtension::Lzop, "lzo")]
    fn test_decompression_settings_conversion_failure(
        #[case] ext: CompressionAlgorithmFileExtension,
        #[case] expected_str: &str,
    ) -> TestResult {
        match DecompressionSettings::try_from(ext) {
            Ok(settings) => panic!("Expected failure, but got: {settings:?}"),
            Err(Error::UnsupportedCompressionAlgorithm { value }) => {
                assert_eq!(value, expected_str);
                Ok(())
            }
            Err(e) => panic!("Unexpected error variant: {e:?}"),
        }
    }

    /// Ensures that the conversion from [`CompressionSettings`] to
    /// [`DecompressionSettings`] works as expected.
    #[rstest]
    #[case::bzip2(CompressionSettings::Bzip2 {
        compression_level: Bzip2CompressionLevel::default()
    }, DecompressionSettings::Bzip2)]
    #[case::gzip(CompressionSettings::Gzip {
        compression_level: GzipCompressionLevel::default()
    }, DecompressionSettings::Gzip)]
    #[case::xz(CompressionSettings::Xz {
        compression_level: XzCompressionLevel::default()
    }, DecompressionSettings::Xz)]
    #[case::zstd(CompressionSettings::Zstd {
        compression_level: ZstdCompressionLevel::default(),
        threads: ZstdThreads::new(4),
    }, DecompressionSettings::Zstd)]
    #[case(CompressionSettings::None, DecompressionSettings::None)]
    fn test_from_compression_settings_to_decompression_settings(
        #[case] settings: CompressionSettings,
        #[case] expected: DecompressionSettings,
    ) -> TestResult {
        let result = DecompressionSettings::from(&settings);
        assert_eq!(result, expected);
        Ok(())
    }
}
