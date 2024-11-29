//! A custom INI parser and deserializer.
//!
//! This module provides functionality for parsing and deserializing INI-style configuration files,
//! where each line is expected to follow the format `key=value`.
//!
//! It supports keys with single values as well as keys that appear multiple times, which are
//! represented as sequences of values.
//!
//! # Example
//!
//! ```
//! use alpm_parsers::custom_ini;
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize)]
//! struct Data {
//!     num: u64,
//!     text: String,
//!     list: Vec<String>,
//! }
//!
//! fn main() -> custom_ini::Result<()> {
//!     let content = "
//!         num=42
//!         text=foo
//!         list=bar
//!         list=baz
//!         list=qux
//!     ";
//!
//!     let data = custom_ini::from_str::<Data>(content)?;
//!
//!     assert_eq!(data.num, 42);
//!     assert_eq!(data.text, "foo");
//!     assert_eq!(data.list, vec!["bar", "baz", "qux"]);
//!
//!     Ok(())
//! }
//! ```
use std::collections::BTreeMap;
use std::fmt::{self, Display};
use std::marker::PhantomData;
use std::str::{FromStr, ParseBoolError};
use std::{error, num, str};

use serde::de::value::SeqDeserializer;
use serde::{
    de::{self, DeserializeOwned, IntoDeserializer, Visitor},
    forward_to_deserialize_any,
    Deserialize,
};

#[derive(Debug, Clone)]
pub enum Error {
    /// Parsing error
    ///
    /// Encountering this is probably due to a syntax error in the input.
    Parse(String),

    /// Deserialization error
    ///
    /// Passed through error message from the type being deserialized.
    Custom(String),

    /// Internal consistency error
    ///
    /// Encountering this is probably misuse of the deserialization API or a bug in serde-ini.
    UnexpectedEof,

    /// Internal consistency error
    ///
    /// Encountering this is probably misuse of the deserialization API or a bug in serde-ini.
    InvalidState,
}

impl From<num::ParseIntError> for Error {
    fn from(e: num::ParseIntError) -> Self {
        Error::Custom(e.to_string())
    }
}

impl From<num::ParseFloatError> for Error {
    fn from(e: num::ParseFloatError) -> Self {
        Error::Custom(e.to_string())
    }
}

impl From<ParseBoolError> for Error {
    fn from(e: ParseBoolError) -> Self {
        Error::Custom(e.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Custom(msg) => write!(f, "{msg}"),
            Error::Parse(msg) => write!(f, "{msg}"),
            Error::UnexpectedEof => write!(f, "internal consistency error: unexpected EOF"),
            Error::InvalidState => write!(f, "internal consistency error"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "deserialization error"
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Representation of parsed items.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Item {
    Value(String),
    List(Vec<String>),
}

impl Item {
    pub fn value_or_error(&self) -> Result<&str> {
        match self {
            Item::Value(value) => Ok(value),
            Item::List(_) => Err(Error::InvalidState),
        }
    }
}

impl IntoDeserializer<'_, Error> for Item {
    type Deserializer = ItemDeserializer<Error>;

    fn into_deserializer(self) -> Self::Deserializer {
        ItemDeserializer::new(self)
    }
}

/// A deserializer for parsing a list of `Item` objects.
struct Deserializer {
    input: BTreeMap<String, Item>,
}

// Create a new deserializer from a string.
//
/// Parses a string of key-value pairs into a list of `Item` values.
///
/// Each line should be in the format `key=value`.
/// If a key appears multiple times, its values are collected into a `List`.
impl<'a> TryFrom<&'a str> for Deserializer {
    type Error = Error;

    fn try_from(contents: &'a str) -> Result<Self> {
        let mut input = BTreeMap::new();
        let mut items: Vec<(String, Vec<String>)> = Vec::new();

        for line in contents.lines().filter(|line| !line.trim().is_empty()) {
            let mut parts = line.splitn(2, '=');
            let (Some(key), Some(value)) = (parts.next(), parts.next()) else {
                return Err(Error::Custom(format!("invalid line: {line}")));
            };
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            if let Some((_, existing_values)) = items.iter_mut().find(|(k, _)| *k == key) {
                existing_values.push(value);
            } else {
                items.push((key, vec![value]));
            }
        }

        items.into_iter().for_each(|(key, values)| {
            if values.len() == 1 {
                input.insert(key, Item::Value(values[0].clone()));
            } else {
                input.insert(key, Item::List(values.clone()));
            }
        });

        Ok(Deserializer { input })
    }
}

impl<'de> de::Deserializer<'de> for &mut Deserializer {
    type Error = Error;

    fn is_human_readable(&self) -> bool {
        true
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_map(self.input.clone().into_deserializer())
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes
        byte_buf unit unit_struct newtype_struct tuple_struct
        struct identifier ignored_any enum option map tuple seq
    }
}

pub struct ItemDeserializer<E> {
    item: Item,
    marker: PhantomData<E>,
}

impl<E> ItemDeserializer<E> {
    pub fn new(item: Item) -> Self {
        ItemDeserializer {
            item,
            marker: PhantomData,
        }
    }
}

impl<'de> de::Deserializer<'de> for ItemDeserializer<Error> {
    type Error = Error;

    fn is_human_readable(&self) -> bool {
        true
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.item {
            Item::Value(value) => visitor.visit_str(value),
            Item::List(vec) => visitor.visit_seq(vec.clone().into_deserializer()),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        // There are 2 important cases here:
        let de = match self.item {
            // 1. A single value is deserialized as a list of 1 element.
            Item::Value(value) => {
                SeqDeserializer::new(vec![SeqItemDeserializer(value.clone())].into_iter())
            }
            // 2. List of values is deserialized as a sequence of multiple elements.
            Item::List(values) => {
                let mut items = Vec::new();
                for value in values.clone() {
                    items.push(SeqItemDeserializer(value.clone()));
                }
                SeqDeserializer::new(items.into_iter())
            }
        };
        visitor
            .visit_seq(de)
            .map_err(|e| Error::Custom(e.to_string()))
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_bool(FromStr::from_str(self.item.value_or_error()?)?)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i8(FromStr::from_str(self.item.value_or_error()?)?)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i16(FromStr::from_str(self.item.value_or_error()?)?)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i32(FromStr::from_str(self.item.value_or_error()?)?)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i64(FromStr::from_str(self.item.value_or_error()?)?)
    }

    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i128(FromStr::from_str(self.item.value_or_error()?)?)
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u8(FromStr::from_str(self.item.value_or_error()?)?)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u16(FromStr::from_str(self.item.value_or_error()?)?)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u32(FromStr::from_str(self.item.value_or_error()?)?)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u64(FromStr::from_str(self.item.value_or_error()?)?)
    }

    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u128(FromStr::from_str(self.item.value_or_error()?)?)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f32(FromStr::from_str(self.item.value_or_error()?)?)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f64(FromStr::from_str(self.item.value_or_error()?)?)
    }

    forward_to_deserialize_any! {
        char str string bytes
        byte_buf unit unit_struct newtype_struct tuple tuple_struct
        struct identifier ignored_any enum option map
    }
}

/// A deserializer for individual sequence values.
struct SeqItemDeserializer(String);

impl<'de> de::Deserializer<'de> for SeqItemDeserializer {
    type Error = serde::de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(&self.0)
    }

    fn deserialize_u64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.0.parse().unwrap())
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 f32 f64 char str string bytes
        byte_buf unit unit_struct newtype_struct tuple tuple_struct
        map struct identifier ignored_any enum option seq
    }
}

impl IntoDeserializer<'_> for SeqItemDeserializer {
    type Deserializer = SeqItemDeserializer;
    fn into_deserializer(self) -> Self::Deserializer {
        SeqItemDeserializer(self.0)
    }
}

pub fn from_str<T: DeserializeOwned>(s: &str) -> Result<T> {
    let mut de = Deserializer::try_from(s)?;
    let value = Deserialize::deserialize(&mut de)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Deserialize, Clone, PartialEq, Default, Debug)]
    struct TestModel {
        builddate: i64,
        builddir: String,
        buildenv: Vec<String>,
        format: String,
        installed: Vec<String>,
        options: Vec<String>,
        packager: String,
        pkgarch: String,
        pkgbase: String,
        pkgbuild_sha256sum: String,
        pkgname: String,
        pkgver: String,
    }

    const TEST_INPUT: &str = "
        builddate = 1
        builddir = /build
        buildenv = envfoo
        buildenv = envbar
        format = 1
        installed = bar-1.2.3-1-any
        installed = beh-2.2.3-4-any
        options = some_option
        options = !other_option
        options = !other_optionaaaaa
        packager = Foobar McFooface <foobar@mcfooface.org>
        pkgarch = any
        pkgbase = foo
        pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
        pkgname = foo
        pkgver = 1:1.0.0-1";

    fn expected() -> TestModel {
        TestModel {
            builddate: 1,
            builddir: "/build".into(),
            buildenv: vec!["envfoo".into(), "envbar".into()],
            format: "1".into(),
            installed: vec!["bar-1.2.3-1-any".into(), "beh-2.2.3-4-any".into()],
            options: vec![
                "some_option".into(),
                "!other_option".into(),
                "!other_optionaaaaa".into(),
            ],
            packager: "Foobar McFooface <foobar@mcfooface.org>".into(),
            pkgarch: "any".into(),
            pkgbase: "foo".into(),
            pkgbuild_sha256sum: "b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c"
                .into(),
            pkgname: "foo".into(),
            pkgver: "1:1.0.0-1".into(),
        }
    }

    #[test]
    fn deserialize() {
        let v = from_str::<TestModel>(TEST_INPUT).unwrap();
        assert_eq!(expected(), v);
    }

    #[derive(Deserialize, Clone, PartialEq, Default, Debug)]
    struct TypeTestModel {
        i64: i64,
        i32: i32,
        u64: u64,
        u32: u32,
        list: Vec<String>,
        u64_list: Vec<u64>,
        bool: bool,
    }

    const TYPE_TEST_INPUT: &str = "
        i64= -64
        i32= -32
        u64= 64
        u32= 32
        list= a
        list= b
        list= c
        u64_list= 1
        u64_list= 2
        u64_list= 3
        bool= true";
    #[test]
    fn deserialize_types() {
        let value = from_str::<TypeTestModel>(TYPE_TEST_INPUT).unwrap();
        assert_eq!(
            TypeTestModel {
                i64: -64,
                i32: -32,
                u64: 64,
                u32: 32,
                list: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                u64_list: vec![1, 2, 3],
                bool: true
            },
            value
        );
    }
}
