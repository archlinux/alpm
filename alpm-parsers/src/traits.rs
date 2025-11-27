//! Traits used for the parsers.

use winnow::{
    ModalResult,
    Parser,
    ascii::line_ending,
    combinator::{alt, eof, peek, terminated},
    error::{ContextError, ErrMode},
};

/// Parses either a line ending or eof from an `input`.
///
/// # Errors
///
/// Returns an error if `input` contains neither a line ending nor eof.
fn line_ending_or_eof<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
    alt((line_ending, eof)).parse_next(input)
}

/// A trait for types that implement a parser.
///
/// The [`AlpmParser::parser`] function is expected to only consume the characters that are part of
/// its own structure/format.
///
/// For checks, such as "parse till EOF" or "parse until delimiter", check the [`ParserUntil`] and
/// [`ParserUntilInclusive`] traits, which provide auto implementations for [`AlpmParser`].
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
/// # Ok(())
/// # }
/// ```
pub trait AlpmParser: Sized {
    /// Returns a [`Parser`] that parses `Self` from the given input stream.
    ///
    /// This parser is expected to not consume anything but the tokens needed to parse and build
    /// `Self`.
    fn parser(input: &mut &str) -> ModalResult<Self>;

    /// Attaches a winnow error context to the parser when a parse error is encountered.
    ///
    /// # Note
    ///
    /// This is useful to, for example, show consistent error messages whenever a character is
    /// encountered that should not be in the character set of the value to parse (see
    /// [`ParserUntil::parser_until`]).
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
/// in file parsers. Type parsers, such as `PackageVersion`, do not have any knowledge about the
/// surrounding file format or context during parsing. This trait allows type parsers to stay pure
/// and agnostic to the surrounding format, while allowing file format parsers to specify a
/// delimiter, such as `\n` or `,`, providing the necessary context to parse the type inside the
/// format in question.
///
/// The `ParserUntil` trait is auto-implemented for every type implementing [`AlpmParser`].
///
/// The [`ParserUntilInclusive`] trait is auto-implemented for every type implementing
/// `ParserUntil` and provides functions to also consume the specified delimiter.
///
/// # Note
///
/// Produced parsers are expected to **NOT** consume the delimiter.
/// To properly enforce this constraint several conventions must be upheld:
///
/// - Delimiters must not be part of the allowed set of tokens. Otherwise there's no way to
///   distinguish between delimiter and content.
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
/// #[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
/// struct Alphanumeric(String);
///
/// // Implementing `AlpmParser` automatically implements `ParserUntil`.
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
/// // Parsing the full string with `parser_until_eof` works as expected.
/// assert_eq!(
///     Alphanumeric::parser_until_eof.parse_peek("abc123"),
///     Ok(("", Alphanumeric("abc123".to_string())))
/// );
///
/// // If there's anything but the end of the input, the parser fails.
/// assert!(matches!(
///     Alphanumeric::parser_until_eof.parse_peek("abc123_"),
///     Err(_)
/// ));
/// # Ok(())
/// # }
/// ```
pub trait ParserUntil: Sized {
    /// Returns a [`Parser`] that parses `Self` until the given `delimiter` parser
    /// matches.
    ///
    /// # Note
    ///
    /// Consumes the `delimiter`.
    fn parser_until<'a, P>(delimiter: P) -> impl Parser<&'a str, Self, ErrMode<ContextError>>
    where
        P: Parser<&'a str, &'a str, ErrMode<ContextError>>;

    /// Returns a [`Parser`] that parses an entire `input`.
    ///
    /// Delegates to [`Self::parser_until`] with the [`eof`] delimiter.
    ///
    /// # Note
    ///
    /// The returned [`Parser`] is required to fully consume `input` (including [`eof`]).
    #[inline]
    fn parser_until_eof(input: &mut &str) -> ModalResult<Self> {
        Self::parser_until(eof).parse_next(input)
    }

    /// Returns a [`Parser`] that parses until the end of line or [`eof`].
    ///
    /// Delegates to [`Self::parser_until`] with the [`line_ending`] or [`eof`] delimiter.
    /// A line ending is either `\n` or `\r\n`.
    ///
    /// # Note
    ///
    /// The line ending is not consumed.
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
/// #[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
/// struct Alphanumeric(String);
///
/// // Implementing `AlpmParser` automatically implements `ParserUntilInclusive`:
/// // `ParserUntil` is auto-implemented over `AlpmParser` and `ParserUntilInclusive` is auto-implemented over `ParserUntil`.
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

    /// Returns a [`Parser`] that parses until the end of line and fully consumes it.
    ///
    /// Delegates to [`Self::parser_until_inclusive`] with the [`line_ending`] and [`eof`]
    /// delimiter.
    ///
    /// A line ending is `\n`, `\r\n` or [`eof`] and consumed as well.
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
