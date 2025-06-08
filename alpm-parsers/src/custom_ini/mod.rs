//! Custom INI parser.

mod de;
pub mod parser;

pub use de::{Error, ItemDeserializer, Result, from_str};
