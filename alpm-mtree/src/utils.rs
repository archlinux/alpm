//! File handling integration.

use std::io::Read;

use flate2::read::GzDecoder;

use crate::Error;

/// Two magic bytes that occur at the beginning of gzip files and can be used to detect whether a
/// file is gzip compressed.
/// Spec: <https://datatracker.ietf.org/doc/html/rfc1952#page-6>
pub const GZIP_MAGIC_NUMBER: [u8; 2] = [0x1f, 0x8b];

/// Creates a [`String`] from a byte vector which may be gzip compressed.
///
/// If `buffer` contains gzip compressed data, it decompressed before converting it into a
/// `String`.
/// Detects whether `buffer` contains gzip compressed data by checking if it is longer than two
/// bytes and whether the first two bytes are the [`GZIP_MAGIC_NUMBER`].
///
/// # Errors
///
/// Returns an error if
/// - `buffer` contains invalid gzip compressed data
/// - or `buffer` can not be converted to `String`.
pub fn mtree_buffer_to_string(buffer: Vec<u8>) -> Result<String, Error> {
    if buffer.len() >= 2 && [buffer[0], buffer[1]] == GZIP_MAGIC_NUMBER {
        let mut decoder = GzDecoder::new(buffer.as_slice());

        let mut content = String::new();
        decoder
            .read_to_string(&mut content)
            .map_err(Error::InvalidGzip)?;
        Ok(content)
    } else {
        Ok(String::from_utf8(buffer)?)
    }
}
