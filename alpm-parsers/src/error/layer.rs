//! A single [`Layer`] of a [`ParseStack`] error.

#[cfg(doc)]
use crate::error::ParseStack;
use crate::error::context::StringContext;

/// One level of nesting in our parsing stack.
///
/// A [`Layer`] is used as part of the [`ParseStack`] struct and created when any kind of parsing
/// error backtracks/bubbles up past a [`LayerParser`].
///
/// All context messages are stored as owned [`String`]s (see [`StringContext`]), so they may be
/// created at runtime, e.g. by our translation system.
///
/// [`LayerParser`]: crate::error::LayerParser
#[derive(Clone, Debug)]
pub(crate) struct Layer {
    /// Name passed to [`LayerExt::layer`](crate::error::LayerExt).
    pub(crate) name: String,
    /// Absolute byte offset where this layer's parser began.
    ///
    /// [`ParseStack::at`] represents the respective end to this pointer.
    pub(crate) start: usize,
    /// Every context call collected within this layer, in order they were attached.
    // Idea: If the string allocations turn out to be too performance intensive, we could think
    // about live-concatenating any context strings. I.e. Layer would have three optional strings,
    // one respective string for layer, description and expected items.
    // This would move some of the formatting to the Layer, but would save **lots** of string
    // allocatins.
    pub(crate) contexts: Vec<StringContext>,
}

/// A small wrapper around [`Layer`], which allows us to conveniently handle anonymous layers.
///
/// Anonymous layers are [`ParseStack::pending`] context that hasn't been added to a layer yet.
/// This means that the outermost parser layer has not been closed.
///
/// Since we still want to show this context to users, we need some form of uniform representation
/// for it.
#[derive(Clone, Copy, Debug)]
pub(crate) enum LayerRef<'layer> {
    Anonymous {
        start: usize,
        contexts: &'layer [StringContext],
    },
    Named(&'layer Layer),
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
    pub(crate) fn name(self) -> Option<&'a str> {
        match self {
            LayerRef::Anonymous { .. } => None,
            LayerRef::Named(layer) => Some(&layer.name),
        }
    }

    /// Returns the optional [`StringContext::Label`] messages, joined with commas.
    ///
    /// Returns [`None`] if there are no [`StringContext::Label`] items.
    pub(crate) fn label_message(self) -> Option<String> {
        let message: Vec<&str> = self
            .contexts()
            .iter()
            .filter_map(StringContext::label)
            .collect();

        if message.is_empty() {
            None
        } else {
            Some(message.join(", "))
        }
    }

    /// All [`StringContext::Description`] items joined in a [`String`], without delimiter.
    ///
    /// Returns [`None`], if no descriptions exist or if they are all empty.
    pub(crate) fn descriptions(self) -> Option<String> {
        let mut description = String::new();
        for context in self.contexts() {
            let StringContext::Description(desc) = context else {
                continue;
            };
            description.push_str(desc);
        }

        Some(description).filter(|d| !d.is_empty())
    }

    /// Returns a list of all [`StringContext`] variants with literal values.
    pub(crate) fn expected_literals(self) -> Vec<String> {
        self.contexts()
            .iter()
            .filter_map(StringContext::expected)
            .collect()
    }

    /// Returns the very first [`StringContext::Label`] on this layer.
    pub(crate) fn label(self) -> Option<&'a str> {
        self.contexts().iter().find_map(StringContext::label)
    }

    /// Returns a list of all [`StringContext`] values.
    fn contexts(self) -> &'a [StringContext] {
        match self {
            LayerRef::Anonymous { contexts, .. } => contexts,
            LayerRef::Named(layer) => &layer.contexts,
        }
    }
}
