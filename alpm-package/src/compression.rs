//! Compression handling.

use std::{fs::File, io::Write, thread::available_parallelism};

use alpm_types::CompressionAlgorithmFileExtension;
use bzip2::write::BzEncoder;
use flate2::write::GzEncoder;
use liblzma::write::XzEncoder;
use num_enum::IntoPrimitive;
use strum::{AsRefStr, Display, IntoStaticStr};
use zstd::Encoder;

/// An error that can occur when using compression.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error occurred while creating a Zstandard encoder.
    #[error(
        "Error creating a Zstandard encoder while {context} with {compression_settings:?}:\n{source}"
    )]
    CreateZstandardEncoder {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "Error creating a Zstandard encoder while
        /// {context} with {compression_settings}".
        context: &'static str,
        /// The compression settings chosen for the encoder.
        compression_settings: CompressionSettings,
        /// The source error.
        source: std::io::Error,
    },

    /// An error occurred while finishing a compression encoder.
    #[error("Error while finishing {compression_type} compression encoder:\n{source}")]
    FinishEncoder {
        /// The compression chosen for the encoder.
        compression_type: CompressionAlgorithmFileExtension,
        /// The source error
        source: std::io::Error,
    },

    /// An error occurred while trying to get the available parallelism.
    #[error("Error while trying to get available parallelism:\n{0}")]
    GetParallelism(#[from] std::io::Error),
}

/// Bzip2 compression level.
///
/// The default adheres to the one selected by the [bzip2] executable.
///
/// [bzip2]: https://man.archlinux.org/man/bzip2.1
#[derive(AsRefStr, Clone, Debug, Default, Display, Eq, IntoPrimitive, IntoStaticStr, PartialEq)]
#[repr(u8)]
pub enum Bzip2CompressionLevel {
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

impl From<Bzip2CompressionLevel> for bzip2::Compression {
    /// Creates a [`bzip2::Compression`] from a [`Bzip2CompressionLevel`].
    fn from(value: Bzip2CompressionLevel) -> Self {
        let level: u8 = value.into();
        bzip2::Compression::new(level.into())
    }
}

/// Gzip compression level.
///
/// The default adheres to the one selected by the [gzip] executable.
///
/// [gzip]: https://man.archlinux.org/man/gzip.1
#[derive(AsRefStr, Clone, Debug, Default, Display, Eq, IntoPrimitive, IntoStaticStr, PartialEq)]
#[repr(u8)]
pub enum GzipCompressionLevel {
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

impl From<GzipCompressionLevel> for flate2::Compression {
    /// Creates a [`flate2::Compression`] from a [`GzipCompressionLevel`].
    fn from(value: GzipCompressionLevel) -> Self {
        let level: u8 = value.into();
        Self::new(level.into())
    }
}

/// Xz compression level.
///
/// The default adheres to the one selected by the [xz] executable.
///
/// [xz]: https://man.archlinux.org/man/xz.1
#[derive(AsRefStr, Clone, Debug, Default, Display, Eq, IntoPrimitive, IntoStaticStr, PartialEq)]
#[repr(u8)]
pub enum XzCompressionLevel {
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

impl From<XzCompressionLevel> for u32 {
    /// Creates a [`u32`] from an [`XzCompressionLevel`].
    fn from(value: XzCompressionLevel) -> Self {
        let level: u8 = value.into();
        Self::from(level)
    }
}

/// Zstandard compression level.
///
/// The default adheres to the one selected by the [zstd] executable.
///
/// [zstd]: https://man.archlinux.org/man/zstd.1
#[derive(AsRefStr, Clone, Debug, Default, Display, Eq, IntoPrimitive, IntoStaticStr, PartialEq)]
#[repr(u8)]
pub enum ZstdCompressionLevel {
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

impl From<ZstdCompressionLevel> for i32 {
    /// Creates an [`i32`] from a [`ZstdCompressionLevel`].
    fn from(value: ZstdCompressionLevel) -> Self {
        let level: u8 = value.into();
        Self::from(level)
    }
}

/// The amount of threads to use when compressing using zstd.
///
/// The default adheres to the one selected by the [zstd] executable.
///
/// [zstd]: https://man.archlinux.org/man/zstd.1
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ZstdThreads {
    /// All available threads are used.
    ///
    /// Available threads are detected using [`available_parallelism`].
    All,
    /// A specific amount of threads is used.
    ///
    /// At a maximum all available threads are used as detected using [`available_parallelism`].
    Custom(u32),
}

impl Default for ZstdThreads {
    /// Returns the default thread value when compressing with zstd.
    ///
    /// The default adheres to the one selected by the [zstd] executable.
    ///
    /// [zstd]: https://man.archlinux.org/man/zstd.1
    fn default() -> Self {
        Self::Custom(0)
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

/// Encoder for compression which supports multiple backends.
///
/// Wraps [`BzEncoder`], [`GzEncoder`], [`XzEncoder`] and [`Encoder`].
/// Provides a unified [`Write`] implementation across all of them.
pub enum CompressionEncoder<'a> {
    Bzip2(BzEncoder<File>),
    Gzip(GzEncoder<File>),
    Xz(XzEncoder<File>),
    Zstandard(Encoder<'a, File>),
}

impl CompressionEncoder<'_> {
    /// Creates a new [`CompressionEncoder`].
    ///
    /// Uses a [`File`] to stream to and initializes a specific backend based on the provided
    /// [`CompressionSettings`].
    ///
    /// # Errors
    ///
    /// Returns an error if creating the encoder for zstandard compression fails.
    pub fn new(file: File, settings: &CompressionSettings) -> Result<Self, Error> {
        Ok(match settings {
            CompressionSettings::Bzip2 { compression_level } => {
                Self::Bzip2(BzEncoder::new(file, compression_level.clone().into()))
            }
            CompressionSettings::Gzip { compression_level } => {
                Self::Gzip(GzEncoder::new(file, compression_level.clone().into()))
            }
            CompressionSettings::Xz { compression_level } => Self::Xz(XzEncoder::new_parallel(
                file,
                compression_level.clone().into(),
            )),
            CompressionSettings::Zstd {
                compression_level,
                threads,
            } => {
                let mut encoder =
                    Encoder::new(file, compression_level.clone().into()).map_err(|source| {
                        Error::CreateZstandardEncoder {
                            context: "initializing",
                            compression_settings: settings.clone(),
                            source,
                        }
                    })?;
                // Include a context checksum at the end of each frame.
                encoder
                    .include_checksum(true)
                    .map_err(|source| Error::CreateZstandardEncoder {
                        context: "setting checksums to be added",
                        compression_settings: settings.clone(),
                        source,
                    })?;

                // Get available threads and lossy-convert from usize to u32.
                let available_threads = available_parallelism()
                    .map_err(Error::GetParallelism)?
                    .get() as u32;
                // Get amount of threads to use.
                let threads: u32 = if let ZstdThreads::Custom(threads_ref) = threads {
                    let mut threads: u32 = 0;
                    threads_ref.clone_into(&mut threads);
                    // Use available threads if threads are set to a higher value.
                    if threads > available_threads {
                        available_threads
                    } else {
                        threads
                    }
                } else {
                    available_threads
                };

                // Use multi-threading if it is available.
                encoder
                    .multithread(threads)
                    .map_err(|source| Error::CreateZstandardEncoder {
                        context: "setting checksums to be added",
                        compression_settings: settings.clone(),
                        source,
                    })?;

                Self::Zstandard(encoder)
            }
        })
    }

    /// Finishes the compression stream.
    ///
    /// # Error
    ///
    /// Returns an error if the wrapped encoder fails.
    pub fn finish(self) -> Result<File, Error> {
        match self {
            CompressionEncoder::Bzip2(encoder) => {
                encoder.finish().map_err(|source| Error::FinishEncoder {
                    compression_type: CompressionAlgorithmFileExtension::Bzip2,
                    source,
                })
            }
            CompressionEncoder::Gzip(encoder) => {
                encoder.finish().map_err(|source| Error::FinishEncoder {
                    compression_type: CompressionAlgorithmFileExtension::Gzip,
                    source,
                })
            }
            CompressionEncoder::Xz(encoder) => {
                encoder.finish().map_err(|source| Error::FinishEncoder {
                    compression_type: CompressionAlgorithmFileExtension::Xz,
                    source,
                })
            }
            CompressionEncoder::Zstandard(encoder) => {
                encoder.finish().map_err(|source| Error::FinishEncoder {
                    compression_type: CompressionAlgorithmFileExtension::Zstd,
                    source,
                })
            }
        }
    }
}

impl Write for CompressionEncoder<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            CompressionEncoder::Bzip2(encoder) => encoder.write(buf),
            CompressionEncoder::Gzip(encoder) => encoder.write(buf),
            CompressionEncoder::Xz(encoder) => encoder.write(buf),
            CompressionEncoder::Zstandard(encoder) => encoder.write(buf),
        }
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        match self {
            CompressionEncoder::Bzip2(encoder) => encoder.write_vectored(bufs),
            CompressionEncoder::Gzip(encoder) => encoder.write_vectored(bufs),
            CompressionEncoder::Xz(encoder) => encoder.write_vectored(bufs),
            CompressionEncoder::Zstandard(encoder) => encoder.write_vectored(bufs),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            CompressionEncoder::Bzip2(encoder) => encoder.flush(),
            CompressionEncoder::Gzip(encoder) => encoder.flush(),
            CompressionEncoder::Xz(encoder) => encoder.flush(),
            CompressionEncoder::Zstandard(encoder) => encoder.flush(),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match self {
            CompressionEncoder::Bzip2(encoder) => encoder.write_all(buf),
            CompressionEncoder::Gzip(encoder) => encoder.write_all(buf),
            CompressionEncoder::Xz(encoder) => encoder.write_all(buf),
            CompressionEncoder::Zstandard(encoder) => encoder.write_all(buf),
        }
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        match self {
            CompressionEncoder::Bzip2(encoder) => encoder.write_fmt(fmt),
            CompressionEncoder::Gzip(encoder) => encoder.write_fmt(fmt),
            CompressionEncoder::Xz(encoder) => encoder.write_fmt(fmt),
            CompressionEncoder::Zstandard(encoder) => encoder.write_fmt(fmt),
        }
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
}

#[cfg(test)]
mod tests {
    use std::io::IoSlice;

    use rstest::rstest;
    use tempfile::tempfile;
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

    /// Ensures that the [`Write::write`] implementation works for each [`CompressionEncoder`].
    #[rstest]
    #[case::bzip2(CompressionSettings::Bzip2 { compression_level: Bzip2CompressionLevel::default()})]
    #[case::gzip(CompressionSettings::Gzip { compression_level: GzipCompressionLevel::default()})]
    #[case::xz(CompressionSettings::Xz { compression_level: XzCompressionLevel::default()})]
    #[case::zstd_all_threads(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::All })]
    #[case::zstd_zero_threads(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::Custom(0) })]
    #[case::zstd_one_thread(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::Custom(1) })]
    #[case::zstd_crazy_threads(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::Custom(99999) })]
    fn test_compression_encoder_write(#[case] settings: CompressionSettings) -> TestResult {
        let file = tempfile()?;
        let mut encoder = CompressionEncoder::new(file, &settings)?;
        let ref_encoder = encoder.by_ref();
        let buf = &[1; 8];

        let mut write_len = 0;
        while write_len < buf.len() {
            let len_written = ref_encoder.write(buf)?;
            write_len += len_written;
        }

        ref_encoder.flush()?;

        Ok(())
    }

    /// Ensures that the [`Write::write_vectored`] implementation works for each
    /// [`CompressionEncoder`].
    #[rstest]
    #[case::bzip2(CompressionSettings::Bzip2 { compression_level: Bzip2CompressionLevel::default()})]
    #[case::gzip(CompressionSettings::Gzip { compression_level: GzipCompressionLevel::default()})]
    #[case::xz(CompressionSettings::Xz { compression_level: XzCompressionLevel::default()})]
    #[case::zstd_all_threads(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::All })]
    #[case::zstd_zero_threads(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::Custom(0) })]
    #[case::zstd_one_thread(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::Custom(1) })]
    #[case::zstd_crazy_threads(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::Custom(99999) })]
    fn test_compression_encoder_write_vectored(
        #[case] settings: CompressionSettings,
    ) -> TestResult {
        let file = tempfile()?;
        let mut encoder = CompressionEncoder::new(file, &settings)?;
        let ref_encoder = encoder.by_ref();

        let data1 = [1; 8];
        let data2 = [15; 8];
        let io_slice1 = IoSlice::new(&data1);
        let io_slice2 = IoSlice::new(&data2);

        let mut write_len = 0;
        while write_len < data1.len() + data2.len() {
            let len_written = ref_encoder.write_vectored(&[io_slice1, io_slice2])?;
            write_len += len_written;
        }

        ref_encoder.flush()?;

        Ok(())
    }

    /// Ensures that the [`Write::write_all`] implementation works for each [`CompressionEncoder`].
    #[rstest]
    #[case::bzip2(CompressionSettings::Bzip2 { compression_level: Bzip2CompressionLevel::default()})]
    #[case::gzip(CompressionSettings::Gzip { compression_level: GzipCompressionLevel::default()})]
    #[case::xz(CompressionSettings::Xz { compression_level: XzCompressionLevel::default()})]
    #[case::zstd_all_threads(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::All })]
    #[case::zstd_zero_threads(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::Custom(0) })]
    #[case::zstd_one_thread(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::Custom(1) })]
    #[case::zstd_crazy_threads(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::Custom(99999) })]
    fn test_compression_encoder_write_all(#[case] settings: CompressionSettings) -> TestResult {
        let file = tempfile()?;
        let mut encoder = CompressionEncoder::new(file, &settings)?;
        let ref_encoder = encoder.by_ref();
        let buf = &[1; 8];

        ref_encoder.write_all(buf)?;

        ref_encoder.flush()?;

        Ok(())
    }

    /// Ensures that the [`Write::write_fmt`] implementation works for each [`CompressionEncoder`].
    #[rstest]
    #[case::bzip2(CompressionSettings::Bzip2 { compression_level: Bzip2CompressionLevel::default()})]
    #[case::gzip(CompressionSettings::Gzip { compression_level: GzipCompressionLevel::default()})]
    #[case::xz(CompressionSettings::Xz { compression_level: XzCompressionLevel::default()})]
    #[case::zstd_all_threads(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::All })]
    #[case::zstd_zero_threads(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::Custom(0) })]
    #[case::zstd_one_thread(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::Custom(1) })]
    #[case::zstd_crazy_threads(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::Custom(99999) })]
    fn test_compression_encoder_write_fmt(#[case] settings: CompressionSettings) -> TestResult {
        let file = tempfile()?;
        let mut encoder = CompressionEncoder::new(file, &settings)?;
        let ref_encoder = encoder.by_ref();

        ref_encoder.write_fmt(format_args!("{:.*}", 2, 1.234567))?;

        ref_encoder.flush()?;

        Ok(())
    }
}
