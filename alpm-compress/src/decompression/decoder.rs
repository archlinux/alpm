//! Decoder for decompression which supports multiple backends.

use std::{
    fmt::Debug,
    fs::File,
    io::{BufReader, Read},
};

use bzip2::bufread::BzDecoder;
use flate2::bufread::GzDecoder;
use liblzma::bufread::XzDecoder;
use zstd::Decoder;

use crate::{Error, decompression::DecompressionSettings};

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

    /// No compression.
    None(BufReader<File>),
}

impl CompressionDecoder<'_> {
    /// Creates a new [`CompressionDecoder`].
    ///
    /// Uses a [`File`] to stream from and initializes a specific backend based on the provided
    /// [`DecompressionSettings`].
    ///
    /// # Errors
    ///
    /// Returns an error if creating the decoder for zstd compression fails
    /// (all other decoder initializations are infallible).
    pub fn new(file: File, settings: DecompressionSettings) -> Result<Self, Error> {
        match settings {
            DecompressionSettings::Bzip2 => Ok(Self::Bzip2(BzDecoder::new(BufReader::new(file)))),
            DecompressionSettings::Gzip => Ok(Self::Gzip(GzDecoder::new(BufReader::new(file)))),
            DecompressionSettings::Xz => Ok(Self::Xz(XzDecoder::new(BufReader::new(file)))),
            DecompressionSettings::Zstd => Ok(Self::Zstd(
                Decoder::new(file).map_err(Error::CreateZstandardDecoder)?,
            )),
            DecompressionSettings::None => Ok(Self::None(BufReader::new(file))),
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
                CompressionDecoder::None(_) => "None",
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
            CompressionDecoder::None(reader) => reader.read(buf),
        }
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        match self {
            CompressionDecoder::Bzip2(decoder) => decoder.read_to_end(buf),
            CompressionDecoder::Gzip(decoder) => decoder.read_to_end(buf),
            CompressionDecoder::Xz(decoder) => decoder.read_to_end(buf),
            CompressionDecoder::Zstd(decoder) => decoder.read_to_end(buf),
            CompressionDecoder::None(reader) => reader.read_to_end(buf),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Seek, Write};

    use rstest::rstest;
    use tempfile::tempfile;
    use testresult::TestResult;

    use super::*;
    use crate::compression::{
        Bzip2CompressionLevel,
        CompressionEncoder,
        CompressionSettings,
        GzipCompressionLevel,
        XzCompressionLevel,
        ZstdCompressionLevel,
        ZstdThreads,
    };

    /// Ensures that the [`CompressionDecoder`] can decompress data compressed by
    /// [`CompressionEncoder`].
    #[rstest]
    #[case::bzip2(DecompressionSettings::Bzip2, CompressionSettings::Bzip2 {
        compression_level: Bzip2CompressionLevel::default()
    })]
    #[case::gzip(DecompressionSettings::Gzip, CompressionSettings::Gzip {
        compression_level: GzipCompressionLevel::default()
    })]
    #[case::xz(DecompressionSettings::Xz, CompressionSettings::Xz {
        compression_level: XzCompressionLevel::default()
    })]
    #[case::zstd(DecompressionSettings::Zstd, CompressionSettings::Zstd {
        compression_level: ZstdCompressionLevel::default(),
        threads: ZstdThreads::new(0),
    })]
    #[case::no_compression(DecompressionSettings::None, CompressionSettings::None)]
    fn test_compression_decoder_roundtrip(
        #[case] decompression_settings: DecompressionSettings,
        #[case] compression_settings: CompressionSettings,
    ) -> TestResult {
        // Prepare some sample data
        let input_data = b"alpm4ever";

        // Compress it
        let mut file = tempfile()?;
        {
            let mut encoder = CompressionEncoder::new(file.try_clone()?, &compression_settings)?;
            encoder.write_all(input_data)?;
            encoder.flush()?;
            encoder.finish()?;
        }

        // Rewind the file
        file.rewind()?;

        // Decompress it
        let mut decoder = CompressionDecoder::new(file, decompression_settings)?;
        let mut output = Vec::new();
        decoder.read_to_end(&mut output)?;

        // Check data integrity
        assert_eq!(output, input_data);
        Ok(())
    }

    #[rstest]
    #[case::bzip2(DecompressionSettings::Bzip2)]
    #[case::gzip(DecompressionSettings::Gzip)]
    #[case::xz(DecompressionSettings::Xz)]
    #[case::zstd(DecompressionSettings::Zstd)]
    #[case::no_compression(DecompressionSettings::None)]
    fn test_compression_decoder_debug(#[case] settings: DecompressionSettings) -> TestResult {
        let file = tempfile()?;
        let decoder = CompressionDecoder::new(file, settings)?;
        let debug_str = format!("{decoder:?}");
        assert!(debug_str.contains("CompressionDecoder"));
        Ok(())
    }
}
