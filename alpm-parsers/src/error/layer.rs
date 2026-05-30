//! A single [`Layer`] of a [`ParseStack`] error.

use winnow::error::StrContext;

#[cfg(doc)]
use crate::error::ParseStack;

/// One level of nesting in our parsing stack.
///
/// A [`Layer`] is used as part of the [`ParseStack`] struct and created when any kind of parsing
/// error backtracks/bubbles up past a [`LayerParser`].
///
/// [`LayerParser`]: crate::error::LayerParser
#[derive(Clone, Debug)]
pub(crate) struct Layer {
    /// Name passed to [`LayerExt::layer`](crate::error::LayerExt).
    pub(crate) name: &'static str,
    /// Absolute byte offset where this layer's parser began.
    ///
    /// [`ParseStack::at`] represents the respective end to this pointer.
    pub(crate) start: usize,
    /// Every [`Parser::context`](winnow::Parser::context) call collected within this layer, in
    /// order they were attached.
    pub(crate) contexts: Vec<StrContext>,
}

/// A small wrapper around [`Layer`], which allows us to conveniently handle anonymous layers.
///
/// Anonymous layers are [`ParseStack::pending`] context that hasn't been added to a layer yet.
/// This means that the outermost parser layer has not been closed.
///
/// Since we still want to show this context to users, we need some form of uniform representation
/// for it.
#[derive(Clone, Copy, Debug)]
pub(crate) enum LayerRef<'a> {
    Anonymous {
        start: usize,
        contexts: &'a [StrContext],
    },
    Named(&'a Layer),
}

impl<'a> LayerRef<'a> {
    /// The byte offset where this layer begins.
    pub(crate) fn start(self) -> usize {
        match self {
            LayerRef::Anonymous { start, .. } => start,
            LayerRef::Named(layer) => layer.start,
        }
    }

    /// The layer's name.
    ///
    /// `None` in case of an anonymous layer.
    pub(crate) fn name(self) -> Option<&'static str> {
        match self {
            LayerRef::Anonymous { .. } => None,
            LayerRef::Named(layer) => Some(layer.name),
        }
    }

    /// [`StrContext::Label`] context messages, joined with commas.
    pub(crate) fn label_message(self) -> Option<String> {
        let message: Vec<&str> = self
            .contexts()
            .iter()
            .filter_map(|c| {
                let StrContext::Label(message) = c else {
                    return None;
                };
                Some(*message)
            })
            .collect();

        if message.is_empty() {
            None
        } else {
            Some(message.join(", "))
        }
    }

    /// [`StrContext::Expected`] context messages, joined with commas.
    pub(crate) fn expected_message(self) -> Option<String> {
        let parts: Vec<String> = self
            .contexts()
            .iter()
            .filter_map(|c| match c {
                StrContext::Expected(value) => Some(value.to_string()),
                _ => None,
            })
            .collect();

        if parts.is_empty() {
            None
        } else {
            Some(parts.join(", "))
        }
    }

    /// Returns the very first [`StrContext::Label`] on this layer.
    pub(crate) fn label(self) -> Option<&'static str> {
        self.contexts().iter().find_map(|c| match c {
            StrContext::Label(n) => Some(*n),
            _ => None,
        })
    }

    fn contexts(self) -> &'a [StrContext] {
        match self {
            LayerRef::Anonymous { contexts, .. } => contexts,
            LayerRef::Named(layer) => &layer.contexts,
        }
    }
}
