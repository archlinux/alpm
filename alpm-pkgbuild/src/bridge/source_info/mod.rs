//! Convert untyped and unchecked [`BridgeOutput`] into a well-formed [`SourceInfoV1`].

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

    /// Creates a [`SourceInfoV1`] from a [`BridgeOutput`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - a required field is not set,
    /// - a `package` functions exists, but does not correspond to a declared [alpm-split-package],
    /// - a `package` function without an [alpm-package-name] suffix exists in an
    ///   [alpm-split-package] setup,
    /// - a value cannot be turned into its [`alpm_types`] equivalent,
    /// - multiple values exist for a field that only accepts a singular value,
    /// - an [alpm-architecture] is duplicated,
    /// - an [alpm-architecture] is cleared in `package` function,
    /// - or an [alpm-architecture] suffix is set on a keyword that does not support it.
    ///
    /// [alpm-architecture]: https://alpm.archlinux.page/specifications/alpm-architecture.7.html
    /// [alpm-package-name]: https://alpm.archlinux.page/specifications/alpm-package-name.7.html
    /// [alpm-split-package]: https://alpm.archlinux.page/specifications/alpm-split-package.7.html
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

/// Ensures a [`Keyword`] exists in a [`HashMap`], removes it and returns it.
///
/// This is a helper function to ensure expected values are set while throwing context-rich
/// errors if they don't.
///
/// # Errors
///
/// Returns an error if `keyword` is not a key in `map`.
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

/// Ensures that a combination of a [`Keyword`] and an optional [`Architecture`] does not use an
/// [`Architecture`].
///
/// # Errors
///
/// Returns an error, if `architecture` provides an [`Architecture`].
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

/// Ensures that a combination of [`Keyword`] and [`Value`] uses a [`Value::Single`] and returns the
/// value.
///
/// # Errors
///
/// Returns an error, if `value` is a [`Value::Array`].
fn ensure_single_value<'a>(keyword: &Keyword, value: &'a Value) -> Result<&'a String, BridgeError> {
    match value {
        Value::Single(item) => Ok(item),
        Value::Array(values) => Err(BridgeError::UnexpectedArray {
            keyword: keyword.clone(),
            values: values.clone(),
        }),
    }
}

/// Ensures that a combination of [`Keyword`] and [`Value`] uses a [`Value::Single`] and parses the
/// value as a specific type.
///
/// # Errors
///
/// Returns a error for `keyword` if `value` is not [`Value::Single`] or cannot be parsed as the
/// specific type.
fn parse_value<'a, O, P: Parser<&'a str, O, ErrMode<ContextError>>>(
    keyword: &Keyword,
    value: &'a Value,
    mut parser: P,
) -> Result<O, BridgeError> {
    let input = ensure_single_value(keyword, value)?;
    Ok(parser.parse(input).map_err(|err| (keyword.clone(), err))?)
}

/// Ensures a combination of [`Keyword`] and [`Value`] uses a [`Value::Single`] and parses the value
/// as a specific, but optional type.
///
/// Returns [`None`] if `value` is **empty**.
///
/// # Errors
///
/// Returns a error for `keyword` if `value` is not [`Value::Single`] or cannot be parsed as the
/// specific type.
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

/// Parses a [`Value`] as a [`Vec`] of specific types.
///
/// Does not differentiate between [`Value::Single`] and [`Value::Array`] variants, as a
/// [`PKGBUILD`] allows either for array values.
///
/// # Errors
///
/// Returns a error for `keyword` if `value` cannot be parsed.
///
/// [`PKGBUILD`]: https://man.archlinux.org/man/PKGBUILD.5
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
