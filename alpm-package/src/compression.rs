//! Package compression.

use std::{fs::File, io::Write, thread::available_parallelism};

use alpm_types::CompressionAlgorithmFileExtension;
use bzip2::write::BzEncoder;
use flate2::write::GzEncoder;
use liblzma::write::XzEncoder;
use zstd::Encoder;

#[cfg(doc)]
use crate::package::Package;

/// An error that can occur when dealing with alpm-package.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error occurred while creating a Zstandard encoder.
    #[error(
        "Error creating a Zstandard encoder while {context} with {compression_settings}:\n{source}"
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
        /// The amount of threads to use when compressing.
        ///
        /// If set to [`None`], the available parallelism is used (see [`available_parallelism`]).
        /// If set to [`Some`] value, the value is used (at maximum the available parallelism).
        threads: Option<u32>,
    },
}

impl Default for CompressionSettings {
    /// Returns [`CompressionSettings::Zstandard`].
    ///
    /// Defaults for `compression_level` and `threads` follow that of the [zstd] executable.
    ///
    /// [zstd]: https://man.archlinux.org/man/zstd.1
    fn default() -> Self {
        Self::Zstandard {
            compression_level: ZstandardCompression::default(),
            threads: Some(0),
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
            CompressionSettings::Zstandard { .. } => CompressionAlgorithmFileExtension::Zstd,
        }
    }
}

/// Encoder for compression which supports multiple backends.
///
/// Wraps around [`BzEncoder`], [`GzEncoder`], [`XzEncoder`] and [`Encoder`].
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
            CompressionSettings::Zstandard {
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
                let threads: u32 = if let Some(threads_ref) = threads {
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
