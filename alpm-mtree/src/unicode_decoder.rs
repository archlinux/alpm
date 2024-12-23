use std::char;

use crate::error::Error;

/// Represents the state of the UTF-8 decoder.
enum DecoderState {
    // The decoder is currently not inside of a UTF-8 sequence.
    None,
    // We encountered an escape character `\`, but don't know yet if this is actually an escaped
    // UTF-8 sequence.
    FirstByte {
        triplet: String,
    },
    Sequence {
        /// The string since the first time we hit a backslash.
        parsed_chars: String,
        /// All triplets of the current sequence that have been successfully converted to a byte.
        bytes: [u8; 4],
        /// The amount of bytes that are expected for this char.
        /// This is represented by the amount of leading bits on the first byte.
        expected_bytes: usize,
        /// The chars of the current triplet that is being looked at.
        current_byte: String,
    },
}

/// MTREE encodes non-ascii chars as octal triplets.
/// For example, 🌠 translates to `\360\237\214\240`, which is equivalent to
/// `0xf0 0x9f 0x8c 0xa0` hex encoding.
///
/// Each triplet represents a single UTF-8 byte segment, which is a bit odd as we use 3x3=9 bits to
/// represent an 8bit chunk. The first bit is always expected to be `0`.
fn decode_utf8_chars(escaped: String) -> Result<String, Error> {
    // This is the final unescaped string that we'll build char by char.
    let mut unescaped = String::new();

    let mut state = DecoderState::None;

    // Process the string char-by-char.
    for next_char in escaped.chars() {
        match state {
            // We're currently in a no special state, just look at the next char and search for an
            // escape character
            DecoderState::None => {
                // If we don't find one, check the next char.
                if next_char != '\\' {
                    unescaped.push(next_char);
                    continue;
                }

                // In case we find one, start the parsing process!
                state = DecoderState::FirstByte {
                    triplet: String::new(),
                };
            }
            DecoderState::FirstByte { triplet } => ,
            DecoderState::Sequence {
                parsed_chars,
                bytes,
                expected_bytes,
                current_byte,
            } => todo!(),
        }
    }

    // In case a unicode escape sequence was started at the very end of the string, finish it up.
    if let DecoderState::Sequence {
        parsed_chars,
        bytes,
        expected_bytes,
        current_byte,
    } = state
    {}

    Ok(unescaped)
}

/// Convert a octal triplet string into a byte.
///
/// This isn't a trivial conversion as an octal has three bits and an octal triplet has thereby 9
/// bits. The highest bit is not expected to be set, which is something we have to ensure.
/// The current `sequence` is only provided as parameter for better error messages.
///
/// # Errors
///
/// - [Error::InvalidMtreeUnicode] if the highest bit is set even though it shouldn't be set.
fn byte_from_octal_triplet(sequence: &str, triplet: &str) -> Result<u8, Error> {
    u8::from_str_radix(triplet, 8).map_err(|err| {
        Error::InvalidMtreeUnicode(
            sequence.to_string(),
            format!("Failed to convert octal triplet '{triplet}' into bytes:\n{err:?}"),
        )
    })
}

/// Take a single UTF-8 byte sequence and convert it into a char.
///
/// # Errors
///
/// - [Error::InvalidMtreeUnicode] if the bytes don't represent a valid UTF-8 sequence.
fn decode_utf8_char(sequence: &str, bytes: [u8; 4]) -> Result<char, Error> {
    // First of, convert the byte array to a u32, which can be easily converted into a char via
    // Use the native endianes for bytes, as we're also creating the bytes in the native endianes.
    let unicode_integer = u32::from_ne_bytes(bytes);

    char::from_u32(unicode_integer).ok_or_else(|| {
        Error::InvalidMtreeUnicode(
            sequence.to_string(),
            format!("Encountered invalid UTF-8 in escaped byte sequence: {bytes:?}\n"),
        )
    })
}
