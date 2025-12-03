//! Traits used for the parsers.

use winnow::{
    Parser,
    ascii::line_ending,
    combinator::eof,
    error::{ContextError, ErrMode},
};
use winnow::combinator::alt;

/// A trait for types that can be parsed until a given delimiter parser matches.
///
/// Allows creating non-delimiter-aware type parsers, that can be used inline in file parsers.
///
/// # Note
///
/// Produced parsers are expected to **NOT** consume the delimiter.
/// Use [`ParserUntilInclusive`] to create parsers that consume the delimiter.
///
/// # Examples
///
/// ```rust
/// use alpm_parsers::traits::ParserUntil;
/// use winnow::{
///     ModalResult,
///     Parser,
///     combinator::{peek, terminated},
///     error::{ContextError, ErrMode},
///     token::take_while,
/// };
///
/// # fn main() -> testresult::TestResult {
///
/// #[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
/// struct Alphanumeric(String);
///
/// impl ParserUntil for Alphanumeric {
///     fn parser_until<'a, O, P>(delimiter: P) -> impl Parser<&'a str, Self, ErrMode<ContextError>>
///     where
///         P: Parser<&'a str, O, ErrMode<ContextError>> + Clone,
///     {
///         terminated(
///             take_while(1.., |c: char| c.is_alphanumeric())
///                 .map(|s: &str| Alphanumeric(s.to_string())),
///             peek(delimiter),
///         )
///     }
/// }
///
/// assert_eq!(
///     Alphanumeric::parser_until_line_ending().parse_peek("abc123\nnext"),
///     Ok(("\nnext", Alphanumeric("abc123".to_string())))
/// );
///
/// # Ok(())
/// # }
/// ```
pub trait ParserUntil: Sized {
    /// Returns a [`Parser`] that parses until the given `delimiter` parser matches.
    ///
    /// Does not consume the delimiter.
    fn parser_until<'a, O, P>(delimiter: P) -> impl Parser<&'a str, Self, ErrMode<ContextError>>
    where
        P: Parser<&'a str, O, ErrMode<ContextError>> + Clone;

    /// Returns a [`Parser`] that parses the whole input.
    ///
    /// Delegates to [`Self::parser_until`] with the [`eof`] delimiter.
    #[inline]
    fn parser_until_eof<'a>() -> impl Parser<&'a str, Self, ErrMode<ContextError>> {
        Self::parser_until(eof)
    }

    /// Returns a [`Parser`] that parses until the end of line (either `\n` or `\r\n`) or eof.
    ///
    /// Does not consume the line ending.
    /// 
    /// Delegates to [`Self::parser_until`] with the [`line_ending`] delimiter.
    #[inline]
    fn parser_until_line_ending<'a>() -> impl Parser<&'a str, Self, ErrMode<ContextError>> {
        alt((
            Self::parser_until(line_ending),
            Self::parser_until_eof()
        ))
    }
}

/// A trait for types that can be parsed until a given delimiter parser matches.
///
/// Allows creating non-delimiter-aware type parsers, that can be used inline in file parsers.
///
/// Automatically implemented for all types implementing [`ParserUntil`].
///
/// # Note
///
/// Produced parsers are expected to consume the delimiter.
/// Use [`ParserUntil`] to create parsers that do not consume the delimiter.
///
/// # Examples
///
/// ```rust
/// use alpm_parsers::traits::{ParserUntil, ParserUntilInclusive};
/// use winnow::{
///     ModalResult,
///     Parser,
///     combinator::{peek, terminated},
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
/// impl ParserUntil for Alphanumeric {
///     fn parser_until<'a, O, P>(delimiter: P) -> impl Parser<&'a str, Self, ErrMode<ContextError>>
///     where
///         P: Parser<&'a str, O, ErrMode<ContextError>> + Clone,
///     {
///         terminated(
///             take_while(1.., |c: char| c.is_alphanumeric())
///                 .map(|s: &str| Alphanumeric(s.to_string())),
///             peek(delimiter),
///         )
///     }
/// }
///
/// assert_eq!(
///     Alphanumeric::parser_until_line_ending_inclusive().parse_peek("abc123\nnext"),
///     Ok(("next", Alphanumeric("abc123".to_string())))
/// );
///
/// # Ok(())
/// # }
/// ```
pub trait ParserUntilInclusive: Sized {
    /// Returns a [`Parser`] that parses the whole input and consumes the given `delimiter` parser.
    fn parser_until_inclusive<'a, O, P>(
        delimiter: P,
    ) -> impl Parser<&'a str, Self, ErrMode<ContextError>>
    where
        P: Parser<&'a str, O, ErrMode<ContextError>> + Clone;

    /// Returns a [`Parser`] that parses until the end of line (either `\n` or `\r\n`) or eof 
    /// and consumes the line ending.
    ///
    /// Delegates to [`Self::parser_until_inclusive`] with the [`line_ending`] delimiter.
    #[inline]
    fn parser_until_line_ending_inclusive<'a>() -> impl Parser<&'a str, Self, ErrMode<ContextError>>
    {
        alt((
            Self::parser_until_inclusive(line_ending),
            Self::parser_until_inclusive(eof),
        ))
    }
}

impl<U: ParserUntil> ParserUntilInclusive for U {
    /// Returns a [`Parser`] that parses the whole input and consumes the given `delimiter` parser.
    ///
    /// Delegates to [`ParserUntil::parser_until`] and consumes the delimiter.
    #[inline]
    fn parser_until_inclusive<'a, O, P>(
        delimiter: P,
    ) -> impl Parser<&'a str, Self, ErrMode<ContextError>>
    where
        P: Parser<&'a str, O, ErrMode<ContextError>> + Clone,
    {
        move |input: &mut &'a str| {
            let parsed = U::parser_until(delimiter.clone()).parse_next(input)?;
            let _ = delimiter.clone().parse_next(input)?;
            Ok(parsed)
        }
    }
}
