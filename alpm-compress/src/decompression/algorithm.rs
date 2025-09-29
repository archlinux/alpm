use std::fmt::Debug;

use alpm_types::CompressionAlgorithmFileExtension;

use crate::{compression::CompressionSettings, error::Error};

/// A supported compression algorithm.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompressionAlgorithm {
    /// The bzip2 compression algorithm.
    Bzip2,
    /// The gzip compression algorithm.
    Gzip,
    /// The xz compression algorithm.
    Xz,
    /// The zstandard compression algorithm.
    Zstd,
}

impl TryFrom<CompressionAlgorithmFileExtension> for CompressionAlgorithm {
    type Error = Error;

    /// Converts a [`CompressionAlgorithmFileExtension`] into a [`CompressionAlgorithm`].
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

impl From<&CompressionSettings> for CompressionAlgorithm {
    /// Converts a [`CompressionSettings`] into a [`CompressionAlgorithm`].
    fn from(value: &CompressionSettings) -> Self {
        match value {
            CompressionSettings::Bzip2 { .. } => CompressionAlgorithm::Bzip2,
            CompressionSettings::Gzip { .. } => CompressionAlgorithm::Gzip,
            CompressionSettings::Xz { .. } => CompressionAlgorithm::Xz,
            CompressionSettings::Zstd { .. } => CompressionAlgorithm::Zstd,
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use testresult::TestResult;

    use super::*;
    use crate::compression::{
        ZstdThreads,
        level::{
            Bzip2CompressionLevel,
            GzipCompressionLevel,
            XzCompressionLevel,
            ZstdCompressionLevel,
        },
    };

    /// Ensures that the conversion from [`CompressionAlgorithmFileExtension`] to
    /// [`CompressionAlgorithm`] works as expected.
    #[rstest]
    #[case(CompressionAlgorithmFileExtension::Bzip2, CompressionAlgorithm::Bzip2)]
    #[case(CompressionAlgorithmFileExtension::Gzip, CompressionAlgorithm::Gzip)]
    #[case(CompressionAlgorithmFileExtension::Xz, CompressionAlgorithm::Xz)]
    #[case(CompressionAlgorithmFileExtension::Zstd, CompressionAlgorithm::Zstd)]
    fn test_compression_algorithm_conversion_success(
        #[case] ext: CompressionAlgorithmFileExtension,
        #[case] expected: CompressionAlgorithm,
    ) -> TestResult {
        let result = CompressionAlgorithm::try_from(ext)?;
        assert_eq!(result, expected);
        Ok(())
    }

    /// Ensures that the conversion from [`CompressionAlgorithmFileExtension`] to
    /// [`CompressionAlgorithm`] fails as expected for unsupported algorithms.
    #[rstest]
    #[case(CompressionAlgorithmFileExtension::Compress, "Z")]
    #[case(CompressionAlgorithmFileExtension::Lrzip, "lrz")]
    #[case(CompressionAlgorithmFileExtension::Lzip, "lz")]
    #[case(CompressionAlgorithmFileExtension::Lz4, "lz4")]
    #[case(CompressionAlgorithmFileExtension::Lzop, "lzo")]
    fn test_compression_algorithm_conversion_failure(
        #[case] ext: CompressionAlgorithmFileExtension,
        #[case] expected_str: &str,
    ) -> TestResult {
        match CompressionAlgorithm::try_from(ext) {
            Ok(algorithm) => Err(format!("Expected failure, but got: {algorithm:?}").into()),
            Err(Error::UnsupportedCompressionAlgorithm { value }) => {
                assert_eq!(value, expected_str);
                Ok(())
            }
            Err(e) => Err(format!("Unexpected error variant: {e:?}").into()),
        }
    }

    /// Ensures that the conversion from [`CompressionSettings`] to
    /// [`CompressionAlgorithm`] works as expected.
    #[rstest]
    #[case::bzip2(CompressionSettings::Bzip2 {
        compression_level: Bzip2CompressionLevel::default()
    }, CompressionAlgorithm::Bzip2)]
    #[case::gzip(CompressionSettings::Gzip {
        compression_level: GzipCompressionLevel::default()
    }, CompressionAlgorithm::Gzip)]
    #[case::xz(CompressionSettings::Xz {
        compression_level: XzCompressionLevel::default()
    }, CompressionAlgorithm::Xz)]
    #[case::zstd(CompressionSettings::Zstd {
        compression_level: ZstdCompressionLevel::default(),
        threads: ZstdThreads::new(4),
    }, CompressionAlgorithm::Zstd)]
    fn test_from_compression_settings_to_algorithm(
        #[case] settings: CompressionSettings,
        #[case] expected: CompressionAlgorithm,
    ) -> TestResult {
        let result = CompressionAlgorithm::from(&settings);
        assert_eq!(result, expected);
        Ok(())
    }
}
