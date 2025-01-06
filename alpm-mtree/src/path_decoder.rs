use std::char;

use winnow::{
    combinator::{alt, cut_err, fail, preceded},
    error::{AddContext, ContextError, ErrMode, StrContext, StrContextValue},
    stream::{Checkpoint, Stream},
    token::take_while,
    PResult,
    Parser,
};

/// Decodes UTF-8 characters from a string using MTREE-specific escape sequences.
///
/// MTREE uses various decodings.
/// 1. the VIS_CSTYLE encoding of `strsvis(3)`, which encodes a specific set of characters. Of
///    these, only the following control characters are allowed in filenames:
///    - \s Space
///    - \t Tab
///    - \r Carriage Return
///    - \n Line Feed
/// 2. `#` is encoded as `\#` to differentiate between comments.
/// 3. For all other chars, octal triplets in the style of `\360\237\214\240` are used. Check
///    [`unicode_char`] for more info.
///
/// # Solution
///
/// To effectively decode this pattern we use winnow instead of a handwritten parser, mostly to
/// have convenient backtracking and error messages in case we encounter invalid escape
/// sequences or malformed escaped UTF-8.
pub fn decode_utf8_chars(input: &mut &str) -> PResult<String> {
    // This is the string we'll accumulated the decoded path into.
    let mut path = String::new();

    loop {
        // Parse the string until we hit a `\`
        let part = take_while(0.., |c| c != '\\').parse_next(input)?;
        path.push_str(part);

        if input.is_empty() {
            break;
        }

        // We hit a `\`. See if it's an expected escape sequence.
        // If none of the expected sequences are encountered, fail and throw an error.
        let escaped = alt((
            "\\s".map(|s: &str| s.to_string()),
            "\\t".map(|s: &str| s.to_string()),
            "\\r".map(|s: &str| s.to_string()),
            "\\n".map(|s: &str| s.to_string()),
            "\\#".map(|s: &str| s.to_string()),
            unicode_char,
            fail.context(StrContext::Label("escape sequence"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "VIS_CSTYLE encoding or encoded octal triplets for unicode chars.",
                ))),
        ))
        .parse_next(input)?;

        let unescaped = match escaped.as_str() {
            "\\s" => " ".to_string(),
            "\\t" => "\t".to_string(),
            "\\r" => "\r".to_string(),
            "\\n" => "\n".to_string(),
            "\\#" => "#".to_string(),
            _ => escaped,
        };

        path.push_str(&unescaped);
    }

    Ok(path)
}

/// Parse and convert a single octal triplet string into a byte.
///
/// This isn't a trivial conversion as an octal has three bits and an octal triplet has thereby 9
/// bits. The highest bit is expected to be always `0`. This is ensured via the conversion to `u8`,
/// which would otherwise overflow and throw an error.
fn octal_triplet(input: &mut &str) -> PResult<u8> {
    preceded('\\', take_while(3, |c: char| c.is_digit(8)))
        .verify_map(|octals| u8::from_str_radix(octals, 8).ok())
        .parse_next(input)
}

/// Parse and decode a unicode char that's encoded as octal triplets.
///
/// For example, 🌠 translates to `\360\237\214\240`, which is equivalent to
/// `0xf0 0x9f 0x8c 0xa0` hex encoding.
///
/// Each triplet represents a single UTF-8 byte segment, check [`octal_triplet`] for more details.
fn unicode_char(input: &mut &str) -> PResult<String> {
    // A unicode char can consist of up to 4 bytes, which is what we use this buffer for.
    let mut unicode_bytes = Vec::new();

    // Create a checkpoint in case there's an error while decoding the whole
    // byte sequence in the very end.
    let checkpoint = input.checkpoint();

    // Parse the first octal triplet into bytes.
    // If the input isn't an octal triplet, we hit an unknown encoding and return a backtrack error
    // for a better error message on a higher level.
    let first = octal_triplet(input)?;

    unicode_bytes.push(first);

    // Get the number of leading ones, which determines the amount of following
    // bytes in this unicode char. This amount of leading ones can be one of `[0, 2, 3, 4]`.
    // Other values are forbidden.
    let leading_ones: usize = first.leading_ones() as usize;

    // If there're no leading ones this char is a single byte UTF-8 char.
    if leading_ones == 0 {
        return bytes_to_string(input, checkpoint, unicode_bytes);
    }

    // Make sure that we didn't get an invalid amount of leading zeroes
    if leading_ones > 4 || leading_ones == 1 {
        let mut error = ContextError::new();
        error = error.add_context(
            input,
            &checkpoint,
            StrContext::Label("amount of leading zeroes in first UTF-8 byte"),
        );
        return Err(ErrMode::Cut(error));
    }

    // Due to the amount of leading ones, we know how many bytes we have to expect.
    // Parse the amount of expected bytes and throw an error if that didn't work out.
    for _ in 1..leading_ones {
        let byte = cut_err(octal_triplet)
            .context(StrContext::Label("utf8 encoded byte"))
            .context(StrContext::Expected(StrContextValue::Description(
                "octal triplet encoded unicode byte.",
            )))
            .parse_next(input)?;

        unicode_bytes.push(byte);
    }

    // Read the bytes to string, which might result in another parser error.
    bytes_to_string(input, checkpoint, unicode_bytes)
}

/// Take the UTF-8 byte sequence and parse it into a `String`.
///
/// # Errors
///
/// Returns a custom parse error if we encounter an invalid escaped UTF-8 sequence.
fn bytes_to_string(
    input: &mut &str,
    checkpoint: Checkpoint<&str, &str>,
    bytes: Vec<u8>,
) -> PResult<String> {
    match String::from_utf8(bytes) {
        Ok(decoded) => Ok(decoded),
        Err(_) => {
            let mut error = ContextError::new();
            error = error.add_context(input, &checkpoint, StrContext::Label("UTF-8 byte sequence"));
            Err(ErrMode::Cut(error))
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(r"hello\sworld", "hello world")]
    #[case(r"\#", "#")]
    #[case(r"\n", "\n")]
    #[case(r"\r", "\r")]
    #[case(r"\360\237\214\240", "🌠")]
    #[case(
        r"./test\360\237\214\240\342\232\231\302\247\134test\360\237\214\240t\342\232\231e\302\247s\134t",
        "./test🌠⚙§\\test🌠t⚙e§s\\t"
    )]
    fn test_decode_utf8_chars(#[case] input: &str, #[case] expected: &str) {
        let input = input.to_string();
        let result = decode_utf8_chars(&mut input.as_str());
        assert_eq!(result, Ok(expected.to_string()));
    }

    #[rstest]
    // Unknown escape sequence
    #[case(r"invalid\escape")]
    // First octal triplet will result in u8 int overflow.
    #[case(r"\460\237\214\240")]
    // 4 byte segments are expected, 3 are passed.
    #[case(r"\360\237\214")]
    // 5 leading zeroes in first byte.
    #[case(r"\370\237\214\240")]
    fn test_decode_utf8_chars_invalid_escape(#[case] input: &str) {
        let input = input.to_string();
        let result = decode_utf8_chars(&mut input.as_str());
        assert!(result.is_err());
    }
}
