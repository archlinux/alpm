#![doc = include_str!("../README.md")]

pub mod custom_ini;
pub mod error;
pub mod macros;
pub mod traits;

/// This prelude contains all important types needed to both create parsers in the alpm workspace.
/// It's also convenient to import by consumers, as it also contains all traits necessary for
/// interacting with our parsers.
pub mod prelude {
    pub use winnow::{
        Parser,
        combinator::impls::Context,
        error::{StrContext, StrContextValue},
    };

    pub use crate::{
        error::{Input, LayerExt, LayerParser, PResult},
        traits::{AlpmParser, ParserUntil, ParserUntilInclusive},
    };
}
