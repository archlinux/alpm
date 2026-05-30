//! This module povides a custom winnow error type that keeps track of nested context and spans.
//!
//! [`ParseStack`] keeps a stack of nested layers (one per [`LayerParser`])
//! together with the span positions that were being parsed.
//!
//! The error type requires the usage of winnow's [`LocatingSlice`], which allows our errors to be
//! span-aware.
//!
//! [`LayerParser`]s are created by using the [`LayerExt`] trait.

mod layer;
mod layer_parser;
mod parse_stack;
mod render;

pub use layer_parser::{LayerExt, LayerParser};
pub use parse_stack::ParseStack;
use winnow::{LocatingSlice, ModalResult};

/// A convenience type alias around LocatingSlice.
pub type Input<'i> = LocatingSlice<&'i str>;

/// Return type alias for parsers that uses the ParseStack error type.
pub type PResult<'i, T> = ModalResult<T, ParseStack<'i>>;
