//! Convert untyped and unchecked [BridgeOutput] into a well-formed [SourceInfoV1].

mod package;
mod package_base;

use std::collections::HashMap;

use alpm_srcinfo::SourceInfoV1;
use alpm_types::{Architecture, Name};
use package::handle_packages;
use package_base::handle_package_base;
use winnow::{
    Parser,
    error::{ContextError, ErrMode, ParseError},
};

use super::{
    error::BridgeError,
    parser::{BridgeOutput, Keyword, Value},
};

impl TryFrom<BridgeOutput> for SourceInfoV1 {
    type Error = BridgeError;

    /// Convert [BridgeOutput] into a [SourceInfoV1].
    ///
    /// # Errors
    ///
    /// - Required fields are not set.
    /// - Package functions exist for non-declared split-packages.
    /// - A `package` function exists in a split-package.
    /// - Values cannot be parsed into their respective ALPM type.
    /// - Multiple values for singular-value fields.
    /// - Duplicate architectures
    /// - Architecture is cleared in package overrides
    /// - A architecture suffix is set on keywords that don't support it.
    fn try_from(mut value: BridgeOutput) -> Result<Self, Self::Error> {
        let mut name = None;
        // Check if there's a `pkgbase` section, which hints that this is a split package.
        let pkgbase_keyword = Keyword::simple("pkgbase");
        if let Some(value) = value.package_base.remove(&pkgbase_keyword) {
            name = Some(parse_value(&pkgbase_keyword, &value, Name::parser)?);
        }

        // Get the list of all packages that are declared.
        let pkgname_keyword = Keyword::simple("pkgname");
        let names = ensure_keyword_exists(&pkgname_keyword, &mut value.package_base)?;
        let names = parse_value_array(&pkgname_keyword, &names, Name::parser)?;

        // Use the `pkgbase` name by default, otherwise fallback to the first `pkgname` entry.
        let name = match name {
            Some(name) => name,
            None => {
                // The first package name is used as the name for the pkgbase section.
                names.first().cloned().ok_or(BridgeError::NoName)?
            }
        };

        let base = handle_package_base(name.clone(), value.package_base)?;

        // Go through all declared functions and ensure that the package functions are also
        // declared via `pkgname`. If one of them is not, this is a bug.
        for name in value.functions {
            let Some(name) = name.0 else { continue };

            let name =
                Name::parser
                    .parse(&name)
                    .map_err(|err| BridgeError::InvalidPackageName {
                        name: name.clone(),
                        error: err.into(),
                    })?;

            if !names.contains(&name) {
                return Err(BridgeError::UndeclaredPackageName(name.to_string()));
            }
        }

        let packages = handle_packages(name, names, value.packages)?;

        Ok(SourceInfoV1 { base, packages })
    }
}

/// Make sure a given keyword exists, **remove** it from the map and return it.
///
/// This is a helper function to ensure expected values are set while throwing context-rich
/// errors if they don't.
fn ensure_keyword_exists(
    keyword: &Keyword,
    map: &mut HashMap<Keyword, Value>,
) -> Result<Value, BridgeError> {
    match map.remove(keyword) {
        Some(value) => Ok(value),
        None => Err(BridgeError::MissingRequiredKeyword {
            keyword: keyword.clone(),
        }),
    }
}

/// Make sure no architecture is provided for a keyword that doesn't allow it.
///
/// Throw an error, if an architecture is set.
fn ensure_no_suffix(
    keyword: &Keyword,
    architecture: Option<Architecture>,
) -> Result<(), BridgeError> {
    if let Some(arch) = architecture {
        return Err(BridgeError::UnexpectedArchitecture {
            keyword: keyword.clone(),
            suffix: arch,
        });
    }

    Ok(())
}

/// Make sure a given value exists and contains a [`Value::Single`].
///
/// Throw an error, if it contains an [`Value::Array`].
fn ensure_single_value<'a>(keyword: &Keyword, value: &'a Value) -> Result<&'a String, BridgeError> {
    match value {
        Value::Single(item) => Ok(item),
        Value::Array(values) => Err(BridgeError::UnexpectedArray {
            keyword: keyword.clone(),
            values: values.clone(),
        }),
    }
}

/// Takes a [`Value`], ensures it's a [`Value::Single`] and then applies the parser function to it.
///
/// Throws a error for the given keyword when the value cannot be parsed or the value is an array.
fn parse_value<'a, O, P: Parser<&'a str, O, ErrMode<ContextError>>>(
    keyword: &Keyword,
    value: &'a Value,
    mut parser: P,
) -> Result<O, BridgeError> {
    let input = ensure_single_value(keyword, value)?;
    Ok(parser.parse(input).map_err(|err| (keyword.clone(), err))?)
}

/// Takes a [`Value`], ensures it's a [`Value::Single`] and then applies the parser function to it.
/// If the input is **empty**, `None` is returned before any parsing is executed.
///
/// Throws a error for the given keyword when the value cannot be parsed or the value is an array.
fn parse_optional_value<'a, O, P: Parser<&'a str, O, ErrMode<ContextError>>>(
    keyword: &Keyword,
    value: &'a Value,
    mut parser: P,
) -> Result<Option<O>, BridgeError> {
    let input = ensure_single_value(keyword, value)?;

    if input.trim().is_empty() {
        return Ok(None);
    }

    Ok(Some(
        parser.parse(input).map_err(|err| (keyword.clone(), err))?,
    ))
}

/// Takes a [`Value`], and applies the parser function on all elements.
/// Does not differentiate between [`Value::Single`] and [`Value::Array`] variants, as PKGBUILD
/// allows both for array values.
///
/// Throws a error for the given keyword when the value cannot be parsed.
fn parse_value_array<'a, O, P: Parser<&'a str, O, ErrMode<ContextError>>>(
    keyword: &Keyword,
    value: &'a Value,
    mut parser: P,
) -> Result<Vec<O>, BridgeError> {
    let input = value.as_vec();
    Ok(input
        .into_iter()
        .map(|item| parser.parse(item).map_err(|err| (keyword.clone(), err)))
        .collect::<Result<Vec<O>, (Keyword, ParseError<&'a str, ContextError>)>>()?)
}
