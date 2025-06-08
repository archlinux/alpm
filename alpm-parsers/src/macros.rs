//! Macros used with the winnow parser.

/// Take an array of `dyn Iterator<Item = &'static str>` and return a closure that flattens the
/// outer array to map the inner values  onto [winnow::error::StrContextValue::StringLiteral].
///
/// # Example
///
/// ```
/// use alpm_parsers::iter_str_context;
/// use winnow::{
///     ModalResult,
///     Parser,
///     combinator::{alt, cut_err},
/// };
/// /// A parser with a single list of keywords
/// fn parser_single<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
///     let first_list = ["first", "second", "third"];
///     alt(first_list)
///         .context_with(iter_str_context!([first_list]))
///         .parse_next(input)
/// }
///
/// const FIRST_LIST: [&str; 3] = ["first", "second", "third"];
/// const SECOND_LIST: [&str; 2] = ["fourth", "fifth"];
///
/// fn first_parser<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
///     alt(FIRST_LIST).parse_next(input)
/// }
///
/// fn second_parser<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
///     alt(SECOND_LIST).parse_next(input)
/// }
///
/// /// Can be used like this on static arrays of varying length.
/// fn parser_multi<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
///     cut_err(alt((first_parser, second_parser)))
///         .context_with(iter_str_context!([
///             FIRST_LIST.to_vec(),
///             SECOND_LIST.to_vec()
///         ]))
///         .parse_next(input)
/// }
///
/// /// And as follows on local arrays of varying length.
/// fn parser_multi_alt<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
///     let first_list = ["first", "second", "third"];
///     let second_list = ["fourth", "fifth"];
///     alt((first_parser, second_parser))
///         .context_with(iter_str_context!([
///             first_list.to_vec(),
///             second_list.to_vec()
///         ]))
///         .parse_next(input)
/// }
///
/// assert!(parser_single.parse("second").is_ok());
/// assert!(parser_multi.parse("fourth").is_ok());
/// assert!(parser_multi_alt.parse("fourth").is_ok());
/// ```
#[macro_export]
macro_rules! iter_str_context {
    ($iter:expr) => {
        || {
            use winnow::error::{StrContext, StrContextValue};
            $iter
                .into_iter()
                .flatten()
                .map(|s| StrContext::Expected(StrContextValue::StringLiteral(s)))
        }
    };
}

/// Take a `dyn Iterator<Item = &char>` and return a closure that calls `.iter()` and maps
/// the values onto [winnow::error::StrContextValue::CharLiteral].
///
/// # Example
///
/// ```
/// use alpm_parsers::iter_char_context;
/// use winnow::{ModalResult, Parser, combinator::cut_err, token::one_of};
///
/// fn parser(input: &mut &str) -> ModalResult<char> {
///     let accepted_characters = ['a', 'b', 'c'];
///     cut_err(one_of(accepted_characters))
///         .context_with(iter_char_context!(accepted_characters))
///         .parse_next(input)
/// }
///
/// assert!(parser.parse("a").is_ok());
/// ```
#[macro_export]
macro_rules! iter_char_context {
    ($iter:expr) => {
        || {
            use winnow::error::{StrContext, StrContextValue};
            $iter
                .iter()
                .map(|c| StrContext::Expected(StrContextValue::CharLiteral(*c)))
        }
    };
}
