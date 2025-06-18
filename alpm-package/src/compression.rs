//! Compression handling.

use std::{
    fmt::{Debug, Display},
    fs::File,
    io::{BufReader, Read, Write},
    num::TryFromIntError,
};

use alpm_types::CompressionAlgorithmFileExtension;
use bzip2::{bufread::BzDecoder, write::BzEncoder};
use flate2::{bufread::GzDecoder, write::GzEncoder};
use liblzma::{bufread::XzDecoder, write::XzEncoder};
use log::trace;
use zstd::{Decoder, Encoder};

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

    /// An error occurred while creating a Zstandard decoder.
    #[error("Error creating a Zstandard decoder:\n{0}")]
    CreateZstandardDecoder(#[source] std::io::Error),

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
    GetParallelism(#[source] std::io::Error),

    /// An error occurred while trying to convert an integer.
    #[error("Error while trying to convert an integer:\n{0}")]
    IntegerConversion(#[source] TryFromIntError),

    /// A compression level is not valid.
    #[error("Invalid compression level {level} (must be in the range {min} - {max})")]
    InvalidCompressionLevel {
        /// The invalid compression level.
        level: u8,
        /// The minimum valid compression level.
        min: u8,
        /// The maximum valid compression level.
        max: u8,
    },

    /// An unsupported compression algorithm was used.
    #[error("Unsupported compression algorithm: {value}")]
    UnsupportedCompressionAlgorithm {
        /// The unsupported compression algorithm.
        value: String,
    },
}

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

/// A macro to define a compression level struct.
///
/// Accepts the `name` of the compression level struct, its `min`, `max` and `default` values, the
/// `compression` executable it relates to and a `url`, that defines a man page for the
/// `compression` executable.
macro_rules! define_compression_level {
    (
        $name:ident,
        Min => $min:expr,
        Max => $max:expr,
        Default => $default:expr,
        $compression:literal,
        $url:literal
    ) => {
        #[doc = concat!("Compression level for ", $compression, " compression.")]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name(u8);

        impl $name {
            #[doc = concat!("Creates a new [`", stringify!($name), "`] from a [`u8`].")]
            ///
            #[doc = concat!("The `level` must be in the range of [`", stringify!($name), "::min`] and [`", stringify!($name), "::max`].")]
            ///
            /// # Errors
            ///
            #[doc = concat!("Returns an error if the value is not in the range of [`", stringify!($name), "::min`] and [`", stringify!($name), "::max`].")]
            pub fn new(level: u8) -> Result<Self, Error> {
                trace!(concat!("Creating new compression level for ", $compression, " compression with {{level}}"));
                if !($name::min()..=$name::max()).contains(&level) {
                    return Err(Error::InvalidCompressionLevel {
                        level,
                        min: $name::min(),
                        max: $name::max(),
                    });
                }
                Ok(Self(level))
            }

            #[doc = concat!("Returns the default level (`", stringify!($default), "`) for [`", stringify!($name), "`].")]
            ///
            #[doc = concat!("The default level adheres to the one selected by the [", $compression, "] executable.")]
            ///
            #[doc = concat!("[", $compression, "]: ", $url)]
            pub const fn default_level() -> u8 {
                $default
            }

            #[doc = concat!("Returns the minimum allowed level (`", stringify!($min), "`) for [`", stringify!($name), "`].")]
            pub const fn min() -> u8 {
                $min
            }

            #[doc = concat!("Returns the maximum allowed level (`", stringify!($max), "`) for [`", stringify!($name), "`].")]
            pub const fn max() -> u8 {
                $max
            }
        }

        impl Default for $name {
            #[doc = concat!("Returns the default [`", stringify!($name), "`].")]
            ///
            #[doc = concat!("Delegates to [`", stringify!($name), "::default_level`] for retrieving the default compression level.")]
            fn default() -> Self {
                Self($name::default_level())
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<&$name> for i32 {
            fn from(value: &$name) -> Self {
                i32::from(value.0)
            }
        }

        impl From<&$name> for u32 {
            fn from(value: &$name) -> Self {
                 u32::from(value.0)
            }
        }

        impl TryFrom<i8> for $name {
            type Error = Error;

            fn try_from(value: i8) -> Result<Self, Error> {
                 $name::new(u8::try_from(value).map_err(Error::IntegerConversion)?)
            }
        }

        impl TryFrom<i16> for $name {
            type Error = Error;

            fn try_from(value: i16) -> Result<Self, Error> {
                 $name::new(u8::try_from(value).map_err(Error::IntegerConversion)?)
            }
        }

        impl TryFrom<i32> for $name {
            type Error = Error;

            fn try_from(value: i32) -> Result<Self, Error> {
                 $name::new(u8::try_from(value).map_err(Error::IntegerConversion)?)
            }
        }

        impl TryFrom<i64> for $name {
            type Error = Error;

            fn try_from(value: i64) -> Result<Self, Error> {
                 $name::new(u8::try_from(value).map_err(Error::IntegerConversion)?)
            }
        }

        impl TryFrom<u8> for $name {
            type Error = Error;

            fn try_from(value: u8) -> Result<Self, Error> {
                 $name::new(value)
            }
        }

        impl TryFrom<u16> for $name {
            type Error = Error;

            fn try_from(value: u16) -> Result<Self, Error> {
                 $name::new(u8::try_from(value).map_err(Error::IntegerConversion)?)
            }
        }

        impl TryFrom<u32> for $name {
            type Error = Error;

            fn try_from(value: u32) -> Result<Self, Error> {
                 $name::new(u8::try_from(value).map_err(Error::IntegerConversion)?)
            }
        }

        impl TryFrom<u64> for $name {
            type Error = Error;

            fn try_from(value: u64) -> Result<Self, Error> {
                 $name::new(u8::try_from(value).map_err(Error::IntegerConversion)?)
            }
        }
    };
}

// Create the bzip2 compression level struct.
define_compression_level!(
    Bzip2CompressionLevel,
    Min => 1,
    Max => 9,
    Default => 9,
    "bzip2",
    "https://man.archlinux.org/man/bzip2.1"
);

// Create the gzip compression level struct.
define_compression_level!(
    GzipCompressionLevel,
    Min => 1,
    Max => 9,
    Default => 6,
    "gzip",
    "https://man.archlinux.org/man/gzip.1"
);

// Create the xz compression level struct.
define_compression_level!(
    XzCompressionLevel,
    Min => 0,
    Max => 9,
    Default => 6,
    "xz",
    "https://man.archlinux.org/man/xz.1"
);

// Create the zstd compression level struct.
define_compression_level!(
    ZstdCompressionLevel,
    Min => 0,
    Max => 22,
    Default => 3,
    "zstd",
    "https://man.archlinux.org/man/zstd.1"
);

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

/// Decoder for decompression which supports multiple backends.
///
/// Wraps [`BzDecoder`], [`GzDecoder`], [`XzDecoder`] and [`Decoder`]
/// and provides a unified [`Read`] implementation across all of them.
pub enum CompressionDecoder<'a> {
    /// The bzip2 decompression decoder.
    Bzip2(BzDecoder<BufReader<File>>),

    /// The gzip decompression decoder.
    Gzip(GzDecoder<BufReader<File>>),

    /// The xz decompression decoder.
    Xz(XzDecoder<BufReader<File>>),

    /// The zstd decompression decoder.
    Zstd(Decoder<'a, BufReader<File>>),
}

impl CompressionDecoder<'_> {
    /// Creates a new [`CompressionDecoder`].
    ///
    /// Uses a [`File`] to stream from and initializes a specific backend based on the provided
    /// [`CompressionAlgorithm`].
    ///
    /// # Errors
    ///
    /// Returns an error if creating the decoder for zstd compression fails
    /// (all other decoder initializations are infallible).
    pub fn new(file: File, algorithm: CompressionAlgorithm) -> Result<Self, Error> {
        match algorithm {
            CompressionAlgorithm::Bzip2 => Ok(Self::Bzip2(BzDecoder::new(BufReader::new(file)))),
            CompressionAlgorithm::Gzip => Ok(Self::Gzip(GzDecoder::new(BufReader::new(file)))),
            CompressionAlgorithm::Xz => Ok(Self::Xz(XzDecoder::new(BufReader::new(file)))),
            CompressionAlgorithm::Zstd => Ok(Self::Zstd(
                Decoder::new(file).map_err(Error::CreateZstandardDecoder)?,
            )),
        }
    }
}

impl Debug for CompressionDecoder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CompressionDecoder({})",
            match self {
                CompressionDecoder::Bzip2(_) => "Bzip2",
                CompressionDecoder::Gzip(_) => "Gzip",
                CompressionDecoder::Xz(_) => "Xz",
                CompressionDecoder::Zstd(_) => "Zstd",
            }
        )
    }
}

impl Read for CompressionDecoder<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            CompressionDecoder::Bzip2(decoder) => decoder.read(buf),
            CompressionDecoder::Gzip(decoder) => decoder.read(buf),
            CompressionDecoder::Xz(decoder) => decoder.read(buf),
            CompressionDecoder::Zstd(decoder) => decoder.read(buf),
        }
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        match self {
            CompressionDecoder::Bzip2(decoder) => decoder.read_to_end(buf),
            CompressionDecoder::Gzip(decoder) => decoder.read_to_end(buf),
            CompressionDecoder::Xz(decoder) => decoder.read_to_end(buf),
            CompressionDecoder::Zstd(decoder) => decoder.read_to_end(buf),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{IoSlice, Seek};

    use proptest::{proptest, test_runner::Config as ProptestConfig};
    use rstest::rstest;
    use tempfile::tempfile;
    use testresult::TestResult;

    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn valid_bzip2_compression_level_try_from_i8(input in 1..=9i8) {
            assert!(Bzip2CompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_bzip2_compression_level_try_from_i16(input in 1..=9i16) {
            assert!(Bzip2CompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_bzip2_compression_level_try_from_i32(input in 1..=9i32) {
            assert!(Bzip2CompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_bzip2_compression_level_try_from_i64(input in 1..=9i64) {
            assert!(Bzip2CompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_bzip2_compression_level_try_from_u8(input in 1..=9u8) {
            assert!(Bzip2CompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_bzip2_compression_level_try_from_u16(input in 1..=9u16) {
            assert!(Bzip2CompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_bzip2_compression_level_try_from_u32(input in 1..=9u32) {
            assert!(Bzip2CompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_bzip2_compression_level_try_from_u64(input in 1..=9u64) {
            assert!(Bzip2CompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_gzip_compression_level_try_from_i8(input in 1..=9i8) {
            assert!(GzipCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_gzip_compression_level_try_from_i16(input in 1..=9i16) {
            assert!(GzipCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_gzip_compression_level_try_from_i32(input in 1..=9i32) {
            assert!(GzipCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_gzip_compression_level_try_from_i64(input in 1..=9i64) {
            assert!(GzipCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_gzip_compression_level_try_from_u8(input in 1..=9u8) {
            assert!(GzipCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_gzip_compression_level_try_from_u16(input in 1..=9u16) {
            assert!(GzipCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_gzip_compression_level_try_from_u32(input in 1..=9u32) {
            assert!(GzipCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_gzip_compression_level_try_from_u64(input in 1..=9u64) {
            assert!(GzipCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_xz_compression_level_try_from_i8(input in 0..=9i8) {
            assert!(XzCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_xz_compression_level_try_from_i16(input in 0..=9i16) {
            assert!(XzCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_xz_compression_level_try_from_i32(input in 0..=9i32) {
            assert!(XzCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_xz_compression_level_try_from_i64(input in 0..=9i64) {
            assert!(XzCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_xz_compression_level_try_from_u8(input in 0..=9u8) {
            assert!(XzCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_xz_compression_level_try_from_u16(input in 0..=9u16) {
            assert!(XzCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_xz_compression_level_try_from_u32(input in 0..=9u32) {
            assert!(XzCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_xz_compression_level_try_from_u64(input in 0..=9u64) {
            assert!(XzCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_zstd_compression_level_try_from_i8(input in 0..=22i8) {
            assert!(ZstdCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_zstd_compression_level_try_from_i16(input in 0..=22i16) {
            assert!(ZstdCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_zstd_compression_level_try_from_i32(input in 0..=22i32) {
            assert!(ZstdCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_zstd_compression_level_try_from_i64(input in 0..=22i64) {
            assert!(ZstdCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_zstd_compression_level_try_from_u8(input in 0..=22u8) {
            assert!(ZstdCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_zstd_compression_level_try_from_u16(input in 0..=22u16) {
            assert!(ZstdCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_zstd_compression_level_try_from_u32(input in 0..=22u32) {
            assert!(ZstdCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_zstd_compression_level_try_from_u64(input in 0..=22u64) {
            assert!(ZstdCompressionLevel::try_from(input).is_ok());
        }
    }

    #[rstest]
    #[case::too_large(Bzip2CompressionLevel::max() + 1)]
    #[case::too_small(Bzip2CompressionLevel::min() - 1)]
    fn create_bzip2_compression_level_fails(#[case] level: u8) -> TestResult {
        if let Ok(level) = Bzip2CompressionLevel::new(level) {
            return Err(format!("Should not have succeeded but created level: {level}").into());
        }

        Ok(())
    }

    #[test]
    fn create_bzip2_compression_level_succeeds() -> TestResult {
        if let Err(error) = Bzip2CompressionLevel::new(6) {
            return Err(format!("Should have succeeded but raised error:\n{error}").into());
        }

        Ok(())
    }

    #[rstest]
    #[case::too_large(GzipCompressionLevel::max() + 1)]
    #[case::too_small(GzipCompressionLevel::min() - 1)]
    fn create_gzip_compression_level_fails(#[case] level: u8) -> TestResult {
        if let Ok(level) = GzipCompressionLevel::new(level) {
            return Err(format!("Should not have succeeded but created level: {level}").into());
        }

        Ok(())
    }

    #[test]
    fn create_gzip_compression_level_succeeds() -> TestResult {
        if let Err(error) = GzipCompressionLevel::new(6) {
            return Err(format!("Should have succeeded but raised error:\n{error}").into());
        }

        Ok(())
    }

    #[test]
    fn create_xz_compression_level_fails() -> TestResult {
        if let Ok(level) = XzCompressionLevel::new(XzCompressionLevel::max() + 1) {
            return Err(format!("Should not have succeeded but created level: {level}").into());
        }

        Ok(())
    }

    #[test]
    fn create_xz_compression_level_succeeds() -> TestResult {
        if let Err(error) = XzCompressionLevel::new(6) {
            return Err(format!("Should have succeeded but raised error:\n{error}").into());
        }

        Ok(())
    }

    #[test]
    fn create_zstd_compression_level_fails() -> TestResult {
        if let Ok(level) = ZstdCompressionLevel::new(ZstdCompressionLevel::max() + 1) {
            return Err(format!("Should not have succeeded but created level: {level}").into());
        }

        Ok(())
    }

    #[test]
    fn create_zstd_compression_level_succeeds() -> TestResult {
        if let Err(error) = ZstdCompressionLevel::new(6) {
            return Err(format!("Should have succeeded but raised error:\n{error}").into());
        }

        Ok(())
    }

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

    /// Ensures that the [`CompressionDecoder`] can decompress data compressed by
    /// [`CompressionEncoder`].
    #[rstest]
    #[case::bzip2(CompressionAlgorithm::Bzip2, CompressionSettings::Bzip2 {
        compression_level: Bzip2CompressionLevel::default()
    })]
    #[case::gzip(CompressionAlgorithm::Gzip, CompressionSettings::Gzip {
        compression_level: GzipCompressionLevel::default()
    })]
    #[case::xz(CompressionAlgorithm::Xz, CompressionSettings::Xz {
        compression_level: XzCompressionLevel::default()
    })]
    #[case::zstd(CompressionAlgorithm::Zstd, CompressionSettings::Zstd {
        compression_level: ZstdCompressionLevel::default(),
        threads: ZstdThreads::new(0),
    })]
    fn test_compression_decoder_roundtrip(
        #[case] algorithm: CompressionAlgorithm,
        #[case] settings: CompressionSettings,
    ) -> TestResult {
        // Prepare some sample data
        let input_data = b"alpm4ever";

        // Compress it
        let mut file = tempfile()?;
        {
            let mut encoder = CompressionEncoder::new(file.try_clone()?, &settings)?;
            encoder.write_all(input_data)?;
            encoder.flush()?;
            encoder.finish()?;
        }

        // Rewind the file
        file.rewind()?;

        // Decompress it
        let mut decoder = CompressionDecoder::new(file, algorithm)?;
        let mut output = Vec::new();
        decoder.read_to_end(&mut output)?;

        // Check data integrity
        assert_eq!(output, input_data);
        Ok(())
    }

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

    #[rstest]
    #[case::bzip2(CompressionAlgorithm::Bzip2)]
    #[case::gzip(CompressionAlgorithm::Gzip)]
    #[case::xz(CompressionAlgorithm::Xz)]
    #[case::zstd(CompressionAlgorithm::Zstd)]
    fn test_compression_decoder_debug(#[case] algorithm: CompressionAlgorithm) -> TestResult {
        let file = tempfile()?;
        let decoder = CompressionDecoder::new(file, algorithm)?;
        let debug_str = format!("{decoder:?}");
        assert!(debug_str.contains("CompressionDecoder"));
        Ok(())
    }
}
