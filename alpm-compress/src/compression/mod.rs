//! Compression handling.

use std::{fmt::Debug, fs::File, io::Write};

use alpm_types::CompressionAlgorithmFileExtension;
use bzip2::write::BzEncoder;
use flate2::write::GzEncoder;
use liblzma::write::XzEncoder;
use zstd::Encoder;

use crate::error::Error;

pub mod level;

use level::{
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
pub struct ZstdThreads(u32);

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

/// Creates and configures an [`Encoder`].
///
/// Uses a dedicated `compression_level` and amount of `threads` to construct and configure an
/// encoder for zstd compression.
/// The `settings` are merely used for additional context in cases of error.
///
/// # Errors
///
/// Returns an error if
///
/// - the encoder cannot be created using the `file` and `compression_level`,
/// - the encoder cannot be configured to use checksums at the end of each frame,
/// - the amount of physical CPU cores can not be turned into a `u32`,
/// - or multithreading can not be enabled based on the provided `threads` settings.
fn create_zstd_encoder(
    file: File,
    compression_level: &ZstdCompressionLevel,
    threads: &ZstdThreads,
    settings: &CompressionSettings,
) -> Result<Encoder<'static, File>, Error> {
    let mut encoder = Encoder::new(file, compression_level.into()).map_err(|source| {
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

    // Get amount of threads to use.
    let threads = match threads {
        // Use available physical CPU cores if the special value `0` is used.
        // NOTE: For the zstd executable `0` means "use all available threads", while for the zstd
        // crate this means "disable multithreading".
        ZstdThreads(0) => {
            u32::try_from(num_cpus::get_physical()).map_err(Error::IntegerConversion)?
        }
        ZstdThreads(threads) => *threads,
    };

    // Use multi-threading if it is available.
    encoder
        .multithread(threads)
        .map_err(|source| Error::CreateZstandardEncoder {
            context: "setting checksums to be added",
            compression_settings: settings.clone(),
            source,
        })?;

    Ok(encoder)
}

/// Encoder for compression which supports multiple backends.
///
/// Wraps [`BzEncoder`], [`GzEncoder`], [`XzEncoder`] and [`Encoder`].
/// Provides a unified [`Write`] implementation across all of them.
pub enum CompressionEncoder<'a> {
    /// The bzip2 compression encoder.
    Bzip2(BzEncoder<File>),

    /// The gzip compression encoder.
    Gzip(GzEncoder<File>),

    /// The xz compression encoder.
    Xz(XzEncoder<File>),

    /// The zstd compression encoder.
    Zstd(Encoder<'a, File>),
}

impl CompressionEncoder<'_> {
    /// Creates a new [`CompressionEncoder`].
    ///
    /// Uses a [`File`] to stream to and initializes a specific backend based on the provided
    /// [`CompressionSettings`].
    ///
    /// # Errors
    ///
    /// Returns an error if creating the encoder for zstd compression fails.
    /// All other encoder initializations are infallible.
    pub fn new(file: File, settings: &CompressionSettings) -> Result<Self, Error> {
        Ok(match settings {
            CompressionSettings::Bzip2 { compression_level } => Self::Bzip2(BzEncoder::new(
                file,
                bzip2::Compression::new(compression_level.into()),
            )),
            CompressionSettings::Gzip { compression_level } => Self::Gzip(GzEncoder::new(
                file,
                flate2::Compression::new(compression_level.into()),
            )),
            CompressionSettings::Xz { compression_level } => {
                Self::Xz(XzEncoder::new_parallel(file, compression_level.into()))
            }
            CompressionSettings::Zstd {
                compression_level,
                threads,
            } => Self::Zstd(create_zstd_encoder(
                file,
                compression_level,
                threads,
                settings,
            )?),
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
            CompressionEncoder::Zstd(encoder) => {
                encoder.finish().map_err(|source| Error::FinishEncoder {
                    compression_type: CompressionAlgorithmFileExtension::Zstd,
                    source,
                })
            }
        }
    }
}

impl Debug for CompressionEncoder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CompressionEncoder({})",
            match self {
                CompressionEncoder::Bzip2(_) => "Bzip2",
                CompressionEncoder::Gzip(_) => "Gzip",
                CompressionEncoder::Xz(_) => "Xz",
                CompressionEncoder::Zstd(_) => "Zstd",
            }
        )
    }
}

impl Write for CompressionEncoder<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            CompressionEncoder::Bzip2(encoder) => encoder.write(buf),
            CompressionEncoder::Gzip(encoder) => encoder.write(buf),
            CompressionEncoder::Xz(encoder) => encoder.write(buf),
            CompressionEncoder::Zstd(encoder) => encoder.write(buf),
        }
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        match self {
            CompressionEncoder::Bzip2(encoder) => encoder.write_vectored(bufs),
            CompressionEncoder::Gzip(encoder) => encoder.write_vectored(bufs),
            CompressionEncoder::Xz(encoder) => encoder.write_vectored(bufs),
            CompressionEncoder::Zstd(encoder) => encoder.write_vectored(bufs),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            CompressionEncoder::Bzip2(encoder) => encoder.flush(),
            CompressionEncoder::Gzip(encoder) => encoder.flush(),
            CompressionEncoder::Xz(encoder) => encoder.flush(),
            CompressionEncoder::Zstd(encoder) => encoder.flush(),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match self {
            CompressionEncoder::Bzip2(encoder) => encoder.write_all(buf),
            CompressionEncoder::Gzip(encoder) => encoder.write_all(buf),
            CompressionEncoder::Xz(encoder) => encoder.write_all(buf),
            CompressionEncoder::Zstd(encoder) => encoder.write_all(buf),
        }
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        match self {
            CompressionEncoder::Bzip2(encoder) => encoder.write_fmt(fmt),
            CompressionEncoder::Gzip(encoder) => encoder.write_fmt(fmt),
            CompressionEncoder::Xz(encoder) => encoder.write_fmt(fmt),
            CompressionEncoder::Zstd(encoder) => encoder.write_fmt(fmt),
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
    #[case::zstd_all_threads(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::new(0) })]
    #[case::zstd_one_thread(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::new(1) })]
    #[case::zstd_crazy_threads(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::new(99999) })]
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
    #[case::zstd_all_threads(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::new(0) })]
    #[case::zstd_one_thread(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::new(1) })]
    #[case::zstd_crazy_threads(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::new(99999) })]
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
    #[case::zstd_all_threads(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::new(0) })]
    #[case::zstd_one_thread(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::new(1) })]
    #[case::zstd_crazy_threads(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::new(99999) })]
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
    #[case::zstd_all_threads(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::new(0) })]
    #[case::zstd_one_thread(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::new(1) })]
    #[case::zstd_crazy_threads(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::new(99999) })]
    fn test_compression_encoder_write_fmt(#[case] settings: CompressionSettings) -> TestResult {
        let file = tempfile()?;
        let mut encoder = CompressionEncoder::new(file, &settings)?;
        let ref_encoder = encoder.by_ref();

        ref_encoder.write_fmt(format_args!("{:.*}", 2, 1.234567))?;

        ref_encoder.flush()?;

        Ok(())
    }
}
