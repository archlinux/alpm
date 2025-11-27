//! Traits used for the parsers.

use winnow::{
    ModalResult,
    Parser,
    ascii::line_ending,
    combinator::{alt, eof, peek, terminated},
    error::{ContextError, ErrMode},
};

/// A internal helper function that parses either a line ending or eof.
/// This is used in the parser's until_line_ending* functions.
/// It needs to be a static function as parsers in the `ParserUntilInclusive` trait must be
/// `Clone`.
fn line_ending_or_eof<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
    alt((line_ending, eof)).parse_next(input)
}

/// A trait for types that implement a parser.
///
/// The [`AlpmParser::parser`] function is expected to only consume the characters that're part of
/// its own structure/format.
///
/// For checks, such as "parse till EOF" or "parse until delimiter", check the [`ParserUntil`] and
/// [`ParserUntilInclusive`] traits, which provide auto implementations for AlpmParser.
///
/// # Examples
///
/// ```rust
/// use alpm_parsers::traits::AlpmParser;
/// use winnow::{
///     ModalResult,
///     Parser,
///     error::{ContextError, ErrMode, StrContext},
///     token::take_while,
/// };
///
/// # fn main() -> testresult::TestResult {
///
/// #[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
/// struct Alphanumeric(String);
///
/// impl AlpmParser for Alphanumeric {
///     fn parser(input: &mut &str) -> ModalResult<Self> {
///         take_while(1.., |c: char| c.is_alphanumeric())
///             .map(|s: &str| Alphanumeric(s.to_string()))
///             .parse_next(input)
///     }
/// }
///
/// assert_eq!(
///     Alphanumeric::parser.parse_peek("abc123\nnext"),
///     Ok(("\nnext", Alphanumeric("abc123".to_string())))
/// );
///
/// # Ok(())
/// # }
/// ```
pub trait AlpmParser: Sized {
    /// Returns a [`Parser`] that parses `Self` from the given input stream.
    ///
    /// This parser is expected to not consume anything but the tokens needed to parse and build
    /// `Self`.
    fn parser(input: &mut &str) -> ModalResult<Self>;

    /// Attaches an winnow error context to the parser when encountering a parse error on the
    /// `_until` check.
    ///
    /// This is useful to, for example, show consistent error messages whenever a character is
    /// encountered that shouldn't be in the character set of the value to parse.
    ///
    /// This function is only used by the [`ParserUntil`] and the [`ParserUntilInclusive`] traits.
    fn delimiter_error_context<'a, O, P>(
        parser: P,
    ) -> impl Parser<&'a str, O, ErrMode<ContextError>>
    where
        P: Parser<&'a str, O, ErrMode<ContextError>>,
    {
        parser
    }
}

/// A trait for types that can be parsed until a given delimiter parser matches.
///
/// Allows wrapping non-delimiter-aware type parsers, to make them delimiter-aware for inline usage
/// in file parsers. Type parsers, such as PackageVersion, do not have any knowledge about the
/// surrounding file format or context during parsing. This wrapper allows type parsers to stay pure
/// and agnostic to the surrounding format, while allowing file format parsers to specify a
/// delimiter, such as `\n` or `,`, providing the necessary context to parse the type inside the
/// format in question.
///
/// # Note
///
/// Produced parsers are expected to **NOT** consume the delimiter.
/// Use [`ParserUntilInclusive`] to create parsers that consume the delimiter.
///
/// This comes with several implications:
/// - Delimiters must not be part of the allowed set of tokens. Otherwise there's no way to
///   distinguish between delimiter and content.
/// - The `ParserUntil` trait auto-implemented for every type that implements [`AlpmParser`].
/// - In the case that a file format supports escaping, that file format must implement their own
///   escaping-aware variants of parsers for the affected types.
///
/// # Examples
///
/// ```rust
/// use alpm_parsers::traits::{AlpmParser, ParserUntil};
/// use winnow::{
///     ModalResult,
///     Parser,
///     error::{ContextError, ErrMode, StrContext},
///     token::take_while,
/// };
///
/// # fn main() -> testresult::TestResult {
///
/// #[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
/// struct Alphanumeric(String);
///
/// impl AlpmParser for Alphanumeric {
///     fn parser(input: &mut &str) -> ModalResult<Self> {
///         take_while(1.., |c: char| c.is_alphanumeric())
///             .map(|s: &str| Alphanumeric(s.to_string()))
///             .parse_next(input)
///     }
///
///     fn delimiter_error_context<'a, O, P>(
///         parser: P,
///     ) -> impl Parser<&'a str, O, ErrMode<ContextError>>
///     where
///         P: Parser<&'a str, O, ErrMode<ContextError>>,
///     {
///         parser.context(StrContext::Label("alphanumeric characters"))
///     }
/// }
///
/// // The parser succeeds with a alphanumeric string that is followed by a newline.
/// assert_eq!(
///     Alphanumeric::parser_until_line_ending.parse_peek("abc123\nnext"),
///     Ok(("\nnext", Alphanumeric("abc123".to_string())))
/// );
///
/// // An alphanumeric string followed by a non-alphanumeric character that isn't a
/// // newline will fail.
/// assert!(matches!(
///     Alphanumeric::parser_until_line_ending.parse_peek("abc123{\nnext"),
///     Err(_),
/// ));
///
/// // If we expect it to be a `{` though, it works just as expected.
/// assert_eq!(
///     Alphanumeric::parser_until("{").parse_peek("abc123{\nnext"),
///     Ok(("{\nnext", Alphanumeric("abc123".to_string())))
/// );
///
/// # Ok(())
/// # }
/// ```
pub trait ParserUntil: Sized {
    /// Returns a [`Parser`] that parses `Self` until the given `delimiter` parser
    /// matches.
    ///
    /// Consumes the delimiter.
    fn parser_until<'a, P>(delimiter: P) -> impl Parser<&'a str, Self, ErrMode<ContextError>>
    where
        P: Parser<&'a str, &'a str, ErrMode<ContextError>>;

    /// Returns a [`Parser`] that parses the whole input.
    ///
    /// Returns a [`Parser`] that requires the input to be fully consumed by the inner parser.
    #[inline]
    fn parser_until_eof(input: &mut &str) -> ModalResult<Self> {
        Self::parser_until(eof).parse_next(input)
    }

    /// Returns a [`Parser`] that parses until the end of line (either `\n` or `\r\n`) or `eof`.
    /// The line ending is not consumed.
    ///
    /// Delegates to [`Self::parser_until`] with the [`line_ending`] delimiter.
    #[inline]
    fn parser_until_line_ending(input: &mut &str) -> ModalResult<Self> {
        Self::parser_until(line_ending_or_eof).parse_next(input)
    }
}

impl<U: AlpmParser> ParserUntil for U {
    /// Returns a [`Parser`] that parses `Self` until the given `delimiter` parser
    /// matches.
    ///
    /// Consumes the delimiter.
    fn parser_until<'a, P>(delimiter: P) -> impl Parser<&'a str, Self, ErrMode<ContextError>>
    where
        P: Parser<&'a str, &'a str, ErrMode<ContextError>>,
    {
        terminated(U::parser, Self::delimiter_error_context(peek(delimiter)))
    }
}

/// A trait for types that can be parsed until a given delimiter parser matches, while consuming
/// that delimiter.
///
/// Allows creating non-delimiter-aware type parsers, that can be used inline in file parsers.
///
/// Automatically implemented for all types implementing [`ParserUntil`] and thereby transitively
/// [`AlpmParser`].
///
/// # Examples
///
/// ```rust
/// use alpm_parsers::traits::{AlpmParser, ParserUntil, ParserUntilInclusive};
/// use winnow::{
///     ModalResult,
///     Parser,
///     error::{ContextError, ErrMode},
///     token::take_while,
/// };
///
/// # fn main() -> testresult::TestResult {
///
/// #[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
/// struct Alphanumeric(String);
///
/// // Implementing `ParserUntil` auto-implements `ParserUntilInclusive`
/// impl AlpmParser for Alphanumeric {
///     fn parser(input: &mut &str) -> ModalResult<Self> {
///         take_while(1.., |c: char| c.is_alphanumeric())
///             .map(|s: &str| Alphanumeric(s.to_string()))
///             .parse_next(input)
///     }
/// }
///
/// assert_eq!(
///     Alphanumeric::parser_until_line_ending_inclusive.parse_peek("abc123\nnext"),
///     Ok(("next", Alphanumeric("abc123".to_string())))
/// );
///
/// # Ok(())
/// # }
/// ```
pub trait ParserUntilInclusive: Sized {
    /// Returns a [`Parser`] that parses the whole input and consumes the given `delimiter` parser.
    fn parser_until_inclusive<'a, P>(
        delimiter: P,
    ) -> impl Parser<&'a str, Self, ErrMode<ContextError>>
    where
        P: Parser<&'a str, &'a str, ErrMode<ContextError>>;

    /// Returns a [`Parser`] that parses until the end of line (`\n`, `\r\n` or `EOF`) and consumes
    /// the line ending.
    ///
    /// Delegates to [`Self::parser_until_inclusive`] with the [`line_ending`] delimiter.
    #[inline]
    fn parser_until_line_ending_inclusive(input: &mut &str) -> ModalResult<Self> {
        Self::parser_until_inclusive(line_ending_or_eof).parse_next(input)
    }
}

impl<U: ParserUntil> ParserUntilInclusive for U {
    /// Returns a [`Parser`] that parses the whole input and consumes the given `delimiter` parser.
    ///
    /// Delegates to [`ParserUntil::parser_until`] and consumes the delimiter.
    #[inline]
    fn parser_until_inclusive<'a, P>(
        delimiter: P,
    ) -> impl Parser<&'a str, Self, ErrMode<ContextError>>
    where
        P: Parser<&'a str, &'a str, ErrMode<ContextError>>,
    {
        // Define the actual parser closure.
        // The delimiter is moved into the closure and borrowed via `by_ref()` on each call.
        let mut delimiter = delimiter;
        move |input: &mut &'a str| {
            let parsed = U::parser_until(delimiter.by_ref()).parse_next(input)?;
            // Since `U::parser_until` succeeded, we **know** that the delimiter exists.
            // The following does thereby not fail and does not need context.
            let _ = delimiter.by_ref().parse_next(input)?;
            Ok(parsed)
        }
    }
}
