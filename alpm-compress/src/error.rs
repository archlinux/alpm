//! Error handling.

use std::{fmt::Debug, num::TryFromIntError};

use alpm_types::CompressionAlgorithmFileExtension;

use crate::compression::CompressionSettings;

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

    /// An I/O error occurred while reading.
    #[error("I/O read error while {context}:\n{source}")]
    IoRead {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O read error while ".
        context: &'static str,
        /// The source error.
        source: std::io::Error,
    },

    /// An I/O error occurred while writing.
    #[error("I/O write error while {context}:\n{source}")]
    IoWrite {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O write error while ".
        context: &'static str,
        /// The source error.
        source: std::io::Error,
    },

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

    /// A compression algorithm file extension is not known.
    #[error("Unknown compression algorithm file extension:\n{0}")]
    UnknownCompressionAlgorithmFileExtension(#[source] alpm_types::Error),

    /// An unsupported compression algorithm was used.
    #[error("Unsupported compression algorithm: {value}")]
    UnsupportedCompressionAlgorithm {
        /// The unsupported compression algorithm.
        value: String,
    },
}
