//! This module provides a custom winnow error type that keeps track of nested context and spans.
//!
//! [`ParseStack`] keeps a stack of nested layers (one per [`LayerParser`])
//! together with the span positions that were being parsed.
//!
//! The error type requires the usage of winnow's [`LocatingSlice`], which allows our errors to be
//! span-aware.
//!
//! [`LayerParser`]s are created by using the [`LayerExt`] trait.
//!
//! Context messages are attached via the [`ContextExt`] functions.
//!
//! Winnow's [`Parser::context`] with [`StrContext`] is supported as well for backwards
//! compatibility. Once migrated, this backwards compatibility will be phased out.
//!
//![`Parser::context`]: winnow::Parser::context
//![`StrContext`]: winnow::error::StrContext

mod context;
mod context_parser;
mod layer;
mod layer_parser;
mod parse_stack;
mod render;

pub use context_parser::{ContextExt, ContextParser};
pub use layer_parser::{LayerExt, LayerParser};
pub use parse_stack::ParseStack;
use winnow::{LocatingSlice, ModalResult};

/// A convenience type alias around LocatingSlice.
pub type Input<'i> = LocatingSlice<&'i str>;

/// Return type alias for parsers that uses the ParseStack error type.
pub type PResult<'i, T> = ModalResult<T, ParseStack<'i>>;
