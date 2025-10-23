//! Error handling.

use std::{fmt::Debug, num::TryFromIntError};

use alpm_types::CompressionAlgorithmFileExtension;
use fluent_i18n::t;

use crate::compression::CompressionSettings;

/// An error that can occur when using compression.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error occurred while creating a Zstandard encoder.
    #[error("{msg}", msg = t!("error-create-zstd-encoder", {
        "context" => context,
        "compression_settings" => format!("{compression_settings:?}"),
        "source" => source.to_string()
    }))]
    CreateZstandardEncoder {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "Error creating a Zstandard encoder while
        /// {context} with {compression_settings}".
        context: String,
        /// The compression settings chosen for the encoder.
        compression_settings: CompressionSettings,
        /// The source error.
        source: std::io::Error,
    },

    /// An error occurred while creating a Zstandard decoder.
    #[error("{msg}", msg = t!("error-create-zstd-decoder", { "source" => .0.to_string() }))]
    CreateZstandardDecoder(#[source] std::io::Error),

    /// An error occurred while finishing a compression encoder.
    #[error("{msg}", msg = t!("error-finish-encoder", {
        "compression_type" => compression_type.to_string(),
        "source" => source.to_string()
    }))]
    FinishEncoder {
        /// The compression chosen for the encoder.
        compression_type: CompressionAlgorithmFileExtension,
        /// The source error
        source: std::io::Error,
    },

    /// An error occurred while trying to get the available parallelism.
    #[error("{msg}", msg = t!("error-get-parallelism", { "source" => .0.to_string() }))]
    GetParallelism(#[source] std::io::Error),

    /// An error occurred while trying to convert an integer.
    #[error("{msg}", msg = t!("error-integer-conversion", { "source" => .0.to_string() }))]
    IntegerConversion(#[source] TryFromIntError),

    /// An I/O error occurred while reading.
    #[error("{msg}", msg = t!("error-io-read", {
        "context" => context,
        "source" => source.to_string()
    }))]
    IoRead {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O read error while ".
        context: String,
        /// The source error.
        source: std::io::Error,
    },

    /// An I/O error occurred while writing.
    #[error("{msg}", msg = t!("error-io-write", {
        "context" => context,
        "source" => source.to_string()
    }))]
    IoWrite {
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O write error while ".
        context: String,
        /// The source error.
        source: std::io::Error,
    },

    /// A compression level is not valid.
    #[error("{msg}", msg = t!("error-invalid-compression-level", {
        "level" => level.to_string(),
        "min" => min.to_string(),
        "max" => max.to_string()
    }))]
    InvalidCompressionLevel {
        /// The invalid compression level.
        level: u8,
        /// The minimum valid compression level.
        min: u8,
        /// The maximum valid compression level.
        max: u8,
    },

    /// A compression algorithm file extension is not known.
    #[error("{msg}", msg = t!("error-unknown-compression-extension", { "source" => .0.to_string() }))]
    UnknownCompressionAlgorithmFileExtension(#[source] alpm_types::Error),

    /// An unsupported compression algorithm was used.
    #[error("{msg}", msg = t!("error-unsupported-compression", { "value" => value }))]
    UnsupportedCompressionAlgorithm {
        /// The unsupported compression algorithm.
        value: String,
    },
}
