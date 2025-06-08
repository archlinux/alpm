//! A custom parser for INI-style file formats.

use std::collections::BTreeMap;

use serde::Deserialize;
use winnow::{
    ModalResult,
    Parser,
    ascii::{newline, space0, till_line_ending},
    combinator::{
        alt,
        cut_err,
        eof,
        opt,
        preceded,
        repeat,
        repeat_till,
        separated_pair,
        terminated,
    },
    error::{StrContext, StrContextValue},
    token::none_of,
};

use super::de::Error;

const INVALID_KEY_NAME_SYMBOLS: [char; 3] = ['=', ' ', '\n'];

/// Representation of parsed items.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Item {
    /// A single value.
    Value(String),
    /// A list of values.
    List(Vec<String>),
}

impl Item {
    /// Returns a string slice representing a single value.
    ///
    /// # Errors
    ///
    /// Returns an error if the [`Item`] represents a list of values.
    pub fn value_or_error(&self) -> Result<&str, Error> {
        match self {
            Item::Value(value) => Ok(value),
            Item::List(_) => Err(Error::InvalidState),
        }
    }
}

/// Representation of a parsed line.
#[derive(Debug)]
enum ParsedLine<'s> {
    /// A key value pair.
    KeyValue { key: &'s str, value: &'s str },
    /// A comment.
    Comment(&'s str),
}

/// Take all chars, until we hit a char that isn't allowed in a key.
fn key(input: &mut &str) -> ModalResult<()> {
    repeat(1.., none_of(INVALID_KEY_NAME_SYMBOLS)).parse_next(input)
}

/// Parse a single key value pair.
/// The delimiter includes two surrounding spaces, i.e. ` = `.
///
/// ## Examples
///
/// ```ini
/// key = value
/// ```
fn key_value<'s>(input: &mut &'s str) -> ModalResult<(&'s str, &'s str)> {
    separated_pair(
        cut_err(key.take())
            .context(StrContext::Label("key"))
            .context(StrContext::Expected(StrContextValue::Description(
                "a key followed by a ` = ` delimiter.",
            ))),
        cut_err((" ", "=", " "))
            .context(StrContext::Label("delimiter"))
            .context(StrContext::Expected(StrContextValue::Description(
                "a '=' that delimits the key value pair, surrounded by a single space.",
            ))),
        till_line_ending,
    )
    .parse_next(input)
}

/// One or multiple newlines.
/// This also handles the case where there might be multiple lines with spaces.
fn newlines(input: &mut &str) -> ModalResult<()> {
    repeat(0.., (newline, space0)).parse_next(input)
}

/// Parse a comment (a line starting with `#`).
fn comment<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    preceded('#', till_line_ending).parse_next(input)
}

/// Parse a single line consisting of a key value pair or a comment, followed by 0 or more newlines.
fn line<'s>(input: &mut &'s str) -> ModalResult<ParsedLine<'s>> {
    alt((
        terminated(comment, opt(newlines)).map(ParsedLine::Comment),
        terminated(key_value, opt(newlines))
            .map(|(key, value)| ParsedLine::KeyValue { key, value }),
    ))
    .parse_next(input)
}

/// Parse multiple lines.
fn lines<'s>(input: &mut &'s str) -> ModalResult<Vec<ParsedLine<'s>>> {
    let (value, _terminator) = repeat_till(0.., line, eof).parse_next(input)?;

    Ok(value)
}

/// Parse the content of a whole ini file.
pub fn ini_file(input: &mut &str) -> ModalResult<BTreeMap<String, Item>> {
    let mut items: BTreeMap<String, Vec<String>> = BTreeMap::new();

    // Ignore any preceding newlines at the start of the file.
    let parsed_lines = preceded(newlines, lines).parse_next(input)?;
    for parsed_line in parsed_lines {
        match parsed_line {
            ParsedLine::KeyValue { key, value } => {
                let values = items.entry(key.to_string()).or_default();
                values.push(value.to_string());
            }
            ParsedLine::Comment(_v) => {}
        }
    }

    // Collapse the list of all items into their final representation.
    //
    // Keys that only occur a single time are interpreted as a single item.
    // Keys that occur multiple times are interpreted as a list.
    Ok(items
        .into_iter()
        .map(|(key, mut values)| {
            if values.len() == 1 {
                (key, Item::Value(values.remove(0)))
            } else {
                (key, Item::List(values))
            }
        })
        .collect())
}

#[cfg(test)]
mod test {
    use testresult::TestResult;

    use super::*;

    static TEST_NEWLINES_INPUT: &str = "

foo = bar

test = nice

";

    /// Make sure that newlines at any place are just ignored.
    #[test]
    fn test_newlines() -> TestResult<()> {
        let results = ini_file(&mut TEST_NEWLINES_INPUT.to_string().as_str())?;

        let mut expected = BTreeMap::new();
        expected.insert("foo".to_string(), Item::Value("bar".to_string()));
        expected.insert("test".to_string(), Item::Value("nice".to_string()));

        assert_eq!(expected, results);

        Ok(())
    }

    static TEST_LISTS_INPUT: &str = "foo = bar

test = very
test = nice
test = indeed";

    /// Ensure that parsing lists works.
    #[test]
    fn test_lists() -> TestResult<()> {
        let results = ini_file(&mut TEST_LISTS_INPUT.to_string().as_str())?;

        let mut expected = BTreeMap::new();
        expected.insert("foo".to_string(), Item::Value("bar".to_string()));
        expected.insert(
            "test".to_string(),
            Item::List(vec![
                "very".to_string(),
                "nice".to_string(),
                "indeed".to_string(),
            ]),
        );

        assert_eq!(expected, results);

        Ok(())
    }

    static TEST_COMMENT_INPUT: &str = "
# Hey
# This is a comment
foo = bar
# This is another comment
bar = baz
# And another one";

    /// Ensure that comments are ignored.
    #[test]
    fn test_comments() -> TestResult<()> {
        let results = ini_file(&mut TEST_COMMENT_INPUT.to_string().as_str())?;

        let mut expected = BTreeMap::new();
        expected.insert("foo".to_string(), Item::Value("bar".to_string()));
        expected.insert("bar".to_string(), Item::Value("baz".to_string()));

        assert_eq!(expected, results);

        Ok(())
    }
}
