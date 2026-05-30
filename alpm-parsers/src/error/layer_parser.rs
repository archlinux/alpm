//! The [`LayerParser`], which is used to mark beginnings of a new layer.

use winnow::{Parser, error::ErrMode, stream::Location};

use super::{Input, PResult, ParseStack};

/// The parser returned by [`LayerExt::layer`]
#[derive(Debug)]
pub struct LayerParser<P> {
    pub(crate) parser: P,
    pub(crate) name: &'static str,
}

impl<'i, O, P> Parser<Input<'i>, O, ErrMode<ParseStack<'i>>> for LayerParser<P>
where
    P: Parser<Input<'i>, O, ErrMode<ParseStack<'i>>>,
{
    #[inline]
    fn parse_next(&mut self, i: &mut Input<'i>) -> PResult<'i, O> {
        let start = i.current_token_start();
        self.parser
            .parse_next(i)
            .map_err(|e| e.map(|stack| stack.close_layer(self.name, start)))
    }
}

/// Extension trait that adds a [`LayerExt::layer`] function to all parsers
///
/// Calling `layer` allows creation of nested [`LayerParser`] on any parser over the
/// [`Input`] type.
///
/// A layer acts as a boundary of the currently parsed unit of "grammar".
/// All context errors since the last layer (if existing) are bound to this layer.
///
/// # Note
///
/// The order in which [`Parser::context`] and `layer` functions are called is important!
/// A layer contains all `context` calls that were called **beforehand**.
///
/// Any context calls after `layer` will be part of the next layer.
///
/// # Example
///
/// ```rust
/// use alpm_parsers::prelude::*;
/// use winnow::ascii::alphanumeric1;
///
/// # fn main() -> testresult::TestResult {
/// let mut parser = alphanumeric1
///     .context(StrContext::Label("alphanumeric number"))
///     .layer("version");
///
/// parser.parse(Input::new("a1"))?;
/// # Ok(())
/// # }
/// ```
pub trait LayerExt<'i, O>: Parser<Input<'i>, O, ErrMode<ParseStack<'i>>> + Sized {
    /// Wrap this parser in a named nesting layer.
    ///
    /// On failure any pending [`Parser::context`] messages are moved into a `name`d layer,
    /// spanning from where the parser started to where it failed.
    fn layer(self, name: &'static str) -> LayerParser<Self> {
        LayerParser { parser: self, name }
    }
}

impl<'i, O, P> LayerExt<'i, O> for P where P: Parser<Input<'i>, O, ErrMode<ParseStack<'i>>> {}
