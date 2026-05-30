//! The [`ParseStack`] error type.

use winnow::{
    error::{AddContext, FromExternalError, ParserError, StrContext},
    stream::{Location, Stream},
};

use crate::error::{
    Input,
    layer::{Layer, LayerRef},
};
#[cfg(doc)]
use crate::error::{LayerExt, LayerParser};

/// Get a reference to the full original source from an [`Input`].
///
/// [`Input`] fully stores initial input internally. By copying its reference and
/// resetting the pointer to the start, we get the the original string slice that is being parsed.
///
/// This is needed, as the offsets of each layer are relative to the start of the full string
/// slice.
fn full_source<'i>(input: &Input<'i>) -> &'i str {
    let mut full = *input;
    full.reset_to_start();
    *full
}

/// A custom nested and span-aware parser error.
///
/// This error type is used across the Alpm project and provides well-formated and detailed parsing
/// errors. It allows addition of context per layer/type. For more information on the formatting see
/// the `Display` impl of `ParseStack`.
#[derive(Clone, Debug)]
pub struct ParseStack<'i> {
    /// Byte offset of the deepest failure within the input.
    pub(crate) at: usize,
    /// Set when the final failure came from an external error.
    pub(crate) external: Option<String>,
    /// [`Parser::context`](winnow::Parser::context) calls that are not yet captured by a layer
    /// boundary.
    pub(crate) pending: Vec<StrContext>,
    /// All closed nesting layers, innermost first.
    pub(crate) layers: Vec<Layer>,
    /// The full original source text. Required for rendering.
    pub(crate) source: &'i str,
}

impl<'i> ParseStack<'i> {
    /// Move all pending context calls into a new layer named `name`.
    ///
    /// This is called by `LayerParser::parse_next` in case an error unwinds.
    pub(crate) fn close_layer(mut self, name: &'static str, start: usize) -> Self {
        let contexts = std::mem::take(&mut self.pending);
        self.layers.push(Layer {
            name,
            start,
            contexts,
        });
        self
    }

    /// Returns the innermost layer.
    ///
    /// Pending context belongs to the outermost "anonymous" layer that's not yet named.
    /// If no named layer can be found, but there's some pending context, we return the pending
    /// context as an anonymous layer as a fallback.
    pub(crate) fn innermost(&self) -> LayerRef<'_> {
        if let Some(layer) = self.layers.first() {
            LayerRef::Named(layer)
        } else {
            self.anonymous_layer()
        }
    }

    /// Returns the stack from outermost to innermost.
    ///
    /// Pending context is positioned first as an anonymous outermost layer.
    pub(crate) fn layer_stack(&self) -> Vec<LayerRef<'_>> {
        let mut layers = Vec::new();
        if !self.pending.is_empty() {
            layers.push(self.anonymous_layer());
        }
        layers.extend(self.layers.iter().rev().map(LayerRef::Named));
        layers
    }

    /// Returns the first `StrContext::Expected` context we can find, from the innermost outwards.
    ///
    /// If there's no hit on the innermost layer, we just take next best context we can find to
    /// at least provide some information to the user.
    pub(crate) fn first_expected(&self) -> Option<String> {
        self.pending
            .iter()
            .chain(self.layers.iter().flat_map(|l| l.contexts.iter()))
            .find_map(|c| match c {
                StrContext::Expected(v) => Some(v.to_string()),
                _ => None,
            })
    }

    /// The "headline" for the error.
    ///
    /// Tries to get the best match available in the following order:
    /// - [`StrContext::Label`] name of the innermost layer
    /// - Name of first non-anonymous string
    /// - A simple fallback to the static string `"invalid"`.
    pub(crate) fn headline(&self) -> String {
        self.innermost()
            .label()
            .map(ToString::to_string)
            .or_else(|| self.layers.first().map(|layer| layer.name.to_string()))
            .unwrap_or_else(|| "input".to_owned())
    }

    /// Take the current pending context and return it as an anonymous layer.
    fn anonymous_layer(&self) -> LayerRef<'_> {
        LayerRef::Anonymous {
            start: self.at,
            contexts: &self.pending,
        }
    }
}

// The following trait implementations are wiring logic that's necessary to make our error type a
// valid winnow error.

impl<'i> ParserError<Input<'i>> for ParseStack<'i> {
    type Inner = Self;

    fn from_input(input: &Input<'i>) -> Self {
        ParseStack {
            at: input.current_token_start(),
            external: None,
            pending: Vec::new(),
            layers: Vec::new(),
            source: full_source(input),
        }
    }

    /// When handling multiple parsing branches (`alt`), keep the failure that reached further into
    /// the input.
    // Note: This is somewhat of an experiment, but I assume that that branch should usually carry
    // the more useful message, as it progressed further.
    fn or(self, other: Self) -> Self {
        if other.at >= self.at { other } else { self }
    }

    fn into_inner(self) -> Result<Self::Inner, Self> {
        Ok(self)
    }
}

impl<'i> AddContext<Input<'i>, StrContext> for ParseStack<'i> {
    /// Add a new context entry to current layer.
    ///
    /// Entries are stage inside `Self` until the current layer is closed.
    fn add_context(
        mut self,
        _input: &Input<'i>,
        _token_start: &<Input<'i> as Stream>::Checkpoint,
        context: StrContext,
    ) -> Self {
        self.pending.push(context);
        self
    }
}

impl<'i, E: std::error::Error + Send + Sync + 'static> FromExternalError<Input<'i>, E>
    for ParseStack<'i>
{
    fn from_external_error(input: &Input<'i>, e: E) -> Self {
        ParseStack {
            at: input.current_token_start(),
            external: Some(e.to_string()),
            pending: Vec::new(),
            layers: Vec::new(),
            source: full_source(input),
        }
    }
}

impl std::error::Error for ParseStack<'_> {}
