//! A [`ContextParser`] that attaches context messages to a parser.
//!
//! This is our own `String`-capable version winnow's [`Context`] parser and its respective
//! [`ContextExt`] trait to expose functions for adding context on all parsers.
//!
//! [`Context`]: winnow::combinator::impls::Context

use std::borrow::Borrow;

use winnow::{Parser, error::ErrMode};

use super::{Input, PResult, ParseStack, context::StringContext};

/// A parser that attaches context to the [`ParseStack`] in case of an error.
///
/// This is our equivalent to winnow's [`Context`] parser.
///
/// [`Context`]: winnow::combinator::impls::Context
#[derive(Debug)]
pub struct ContextParser<P> {
    pub(crate) parser: P,
    pub(crate) contexts: Vec<StringContext>,
}

impl<'i, O, P> Parser<Input<'i>, O, ErrMode<ParseStack<'i>>> for ContextParser<P>
where
    P: Parser<Input<'i>, O, ErrMode<ParseStack<'i>>>,
{
    #[inline]
    fn parse_next(&mut self, i: &mut Input<'i>) -> PResult<'i, O> {
        self.parser.parse_next(i).map_err(|e| {
            e.map(|mut stack| {
                stack.pending.extend(self.contexts.iter().cloned());
                stack
            })
        })
    }
}

/// The trait that adds context functions to all parsers over [`Input`].
///
/// These functions are the [`String`]-capable counterparts to [`Parser::context`].
///
/// # Note
///
/// Just like [`Parser::context`] calls, any attached context becomes part of the **current** layer,
/// which is closed by the **next** [`LayerExt::layer`].
///
/// # Example
///
/// ```rust
/// use alpm_parsers::prelude::*;
/// use winnow::ascii::digit1;
///
/// # fn main() -> testresult::TestResult {
/// // Context messages may be built at runtime.
/// let expected = format!("Hoped for exactly {} decimal digit", 1);
/// let mut parser = digit1
///     .label("version number")
///     .description(expected)
///     .layer("version");
///
/// parser.parse(Input::new("42"))?;
/// # Ok(())
/// # }
/// ```
///
/// [`Parser::context`]: winnow::Parser::context
/// [`LayerExt::layer`]: crate::error::LayerExt::layer
pub trait ContextExt<'i, O>: Parser<Input<'i>, O, ErrMode<ParseStack<'i>>> + Sized {
    /// Set a label that describes what is currently being parsed.
    fn label(self, label: impl Into<String>) -> ContextParser<Self> {
        ContextParser {
            parser: self,
            contexts: vec![StringContext::Label(label.into())],
        }
    }

    /// Add a `char` literal to describe what is expected at the failing position.
    fn expected_char(self, expected: char) -> ContextParser<Self> {
        ContextParser {
            parser: self,
            contexts: vec![StringContext::ExpectedChar(expected)],
        }
    }

    /// Helper function to add multiple chars like [`Self::expected_char`].
    fn expected_chars(self, expected: impl IntoIterator<Item = char>) -> ContextParser<Self> {
        ContextParser {
            parser: self,
            contexts: expected
                .into_iter()
                .map(StringContext::ExpectedChar)
                .collect(),
        }
    }

    /// Add a string literal that is expected at the failing position.
    fn expected_string(self, expected: &'static str) -> ContextParser<Self> {
        ContextParser {
            parser: self,
            contexts: vec![StringContext::ExpectedString(expected)],
        }
    }

    /// Helper function to add multiple strings literals like [`Self::expected_string`].
    fn expected_strings<I>(self, expected: I) -> ContextParser<Self>
    where
        I: IntoIterator,
        // NOTE: This is a **bit** of a hack.
        // strum's `VALUES` slices are of type `&'static [&'static str]`.
        // Calling a normal `.iter()` on it will result in the Item type being
        // `&'static &'static str`.
        //
        // However, with generics, the type must match **exactly**, so `I::Item: &'static str`
        // won't work. `Borrow` now saves the day by allowing deref coercion via the Borrow trait.
        // That way, we can pass `MyEnum::VALUES` directly instead of having to do a
        // `MyEnum::VALUES.iter().copied()` dance every time around.
        I::Item: Borrow<&'static str>,
    {
        ContextParser {
            parser: self,
            // Read the `NOTE` above on why we need this.
            #[expect(clippy::explicit_auto_deref)]
            contexts: expected
                .into_iter()
                .map(|c| StringContext::ExpectedString(*c.borrow()))
                .collect(),
        }
    }

    /// Add a free-form text that describes what is expected at the failing position.
    fn expected_text(self, expected: impl Into<String>) -> ContextParser<Self> {
        ContextParser {
            parser: self,
            contexts: vec![StringContext::ExpectedText(expected.into())],
        }
    }

    /// Add a free-form description of what went wrong at the failing position.
    ///
    /// Multiple description entries are joined without any spacing or characters.
    fn description(self, description: impl Into<String>) -> ContextParser<Self> {
        ContextParser {
            parser: self,
            contexts: vec![StringContext::Description(description.into())],
        }
    }
}

impl<'i, O, P> ContextExt<'i, O> for P where P: Parser<Input<'i>, O, ErrMode<ParseStack<'i>>> {}
