//! Convert parsed [BridgeOutput::packages] output into [Package]s.

use std::{collections::HashMap, str::FromStr};

use alpm_parsers::iter_str_context;
use alpm_srcinfo::source_info::{
    parser::{RelationKeyword, SharedMetaKeyword},
    v1::package::{Override, Package, PackageArchitecture},
};
use alpm_types::{
    Architecture,
    Backup,
    Changelog,
    Group,
    Install,
    License,
    MakepkgOption,
    Name,
    OptionalDependency,
    PackageDescription,
    PackageRelation,
    RelationOrSoname,
    Url,
};
use strum::VariantNames;
use winnow::{
    ModalResult,
    Parser,
    combinator::{alt, cut_err},
    error::{ContextError, ErrMode, ParseError, StrContext},
    token::rest,
};

use super::ensure_no_suffix;
#[cfg(doc)]
use crate::bridge::parser::BridgeOutput;
use crate::bridge::{
    error::BridgeError,
    parser::{ClearableValue, Keyword, RawPackageName},
};

/// Convert parsed [BridgeOutput::packages] output into [Package]s.
///
/// # Enforced Invariants
/// - All scoped package variables must have a respective entry in `pkgbase.pkgname`.
///
/// # Errors
/// - A `package` function exists in a split-package.
/// - Values cannot be parsed into their respective ALPM type.
/// - Duplicate architectures
/// - Multiple values for singular-value fields.
/// - Architecture is cleared in package overrides
/// - A architecture suffix is set on keywords that don't support it.
pub(crate) fn handle_packages(
    base_package: Name,
    valid_packages: Vec<Name>,
    raw_values: HashMap<RawPackageName, HashMap<Keyword, ClearableValue>>,
) -> Result<Vec<Package>, BridgeError> {
    let mut package_map: HashMap<Name, Package> = HashMap::new();

    for (name, values) in raw_values {
        // Check if the variable is assigned to a specific split package.
        // If it isn't, use the name of the base package instead, which is the default.
        let name = if let Some(name) = name.0 {
            Name::parser
                .parse(&name)
                .map_err(|err| BridgeError::InvalidPackageName {
                    name: name.clone(),
                    error: err.into(),
                })?
        } else {
            // If this is a literal `package` function we have to make sure that this isn't a split
            // package! Split package `package` functions must have a `_$name` suffix.
            if valid_packages.len() > 1 {
                return Err(BridgeError::ExtraPackageFunction(base_package));
            }

            base_package.clone()
        };

        // Make sure the package has been declared in the package base section.
        if !valid_packages.contains(&name) {
            return Err(BridgeError::UndeclaredPackageName(name.to_string()));
        }

        // Get the package on which the properties should be set.
        let package = package_map.entry(name.clone()).or_insert(name.into());

        handle_package(package, values)?;
    }

    // Convert the package map into a vector that follows the same order as the `pkgbase`
    let mut packages = Vec::new();
    for name in valid_packages {
        let Some(package) = package_map.remove(&name) else {
            // Create a empty package entry for any packages that don't have any variable set and
            // thereby haven't been initialized yet.
            packages.push(name.into());
            continue;
        };

        packages.push(package);
    }

    Ok(packages)
}

/// The combination of all keywords that're valid in the scope of a `package` section.
enum PackageKeywords {
    Relation(RelationKeyword),
    SharedMeta(SharedMetaKeyword),
}

impl PackageKeywords {
    /// Recognizes any of the [`PackageKeywords`] in an input string slice.
    ///
    /// Does not consume input and stops after any keyword matches.
    pub fn parser(input: &mut &str) -> ModalResult<PackageKeywords> {
        cut_err(alt((
            RelationKeyword::parser.map(PackageKeywords::Relation),
            SharedMetaKeyword::parser.map(PackageKeywords::SharedMeta),
        )))
        .context(StrContext::Label("package base property type"))
        .context_with(iter_str_context!([
            RelationKeyword::VARIANTS,
            SharedMetaKeyword::VARIANTS,
        ]))
        .parse_next(input)
    }
}

/// Make sure a given value exists and contains a [`ClearableValue::Single`].
///
/// Throw an error, if it contains an [`ClearableValue::Array`].
fn ensure_single_clearable_value<'a>(
    keyword: &Keyword,
    value: &'a ClearableValue,
) -> Result<&'a Option<String>, BridgeError> {
    match value {
        ClearableValue::Single(value) => Ok(value),
        ClearableValue::Array(values) => Err(BridgeError::UnexpectedArray {
            keyword: keyword.clone(),
            values: values.clone().unwrap_or_default().clone(),
        }),
    }
}

/// Takes a [`ClearableValue`], ensures it's a [`ClearableValue::Single`] and then applies the
/// parser function to it.
///
/// Throws a error for the given keyword when the value cannot be parsed or the value is an array.
fn parse_clearable_value<'a, O, P: Parser<&'a str, O, ErrMode<ContextError>>>(
    keyword: &Keyword,
    value: &'a ClearableValue,
    mut parser: P,
) -> Result<Override<O>, BridgeError> {
    // Make sure we have no array
    let value = ensure_single_clearable_value(keyword, value)?;

    // If the value is `None`, it indicates a cleared value.
    let Some(value) = value else {
        return Ok(Override::Clear);
    };

    let parsed_value = parser.parse(value).map_err(|err| (keyword.clone(), err))?;

    Ok(Override::Yes {
        value: parsed_value,
    })
}

/// Takes a [`ClearableValue`], and applies the parser function on all elements.
/// Does not differentiate between [`ClearableValue::Single`] and [`ClearableValue::Array`]
/// variants, as PKGBUILD allows both for array values.
///
/// Throws a error for the given keyword when the value cannot be parsed.
fn parse_clearable_value_array<'a, O, P: Parser<&'a str, O, ErrMode<ContextError>>>(
    keyword: &Keyword,
    value: &'a ClearableValue,
    mut parser: P,
) -> Result<Override<Vec<O>>, BridgeError> {
    let values = match value {
        ClearableValue::Single(value) => {
            let Some(value) = value else {
                return Ok(Override::Clear);
            };
            let value = parser.parse(value).map_err(|err| (keyword.clone(), err))?;

            vec![value]
        }
        ClearableValue::Array(values) => {
            let Some(values) = values else {
                return Ok(Override::Clear);
            };

            values
                .iter()
                .map(|item| parser.parse(item).map_err(|err| (keyword.clone(), err)))
                .collect::<Result<Vec<O>, (Keyword, ParseError<&'a str, ContextError>)>>()?
        }
    };

    Ok(Override::Yes { value: values })
}

/// Handles all potentially architecture specific Vector entries in the [`handle_package`] function.
///
/// If no architecture is encountered, it simply adds the value on the [`Package`] itself.
/// Otherwise, it's added to the respective [`Package::architecture_properties`].
macro_rules! package_value_array {
    (
        $keyword:expr,
        $value:expr,
        $package:ident,
        $field_name:ident,
        $architecture:ident,
        $parser:expr,
    ) => {
        // Check if the property is architecture specific.
        // If so, we have to perform some checks and preparation
        if let Some(architecture) = $architecture {
            // Make sure the architecture specific properties are initialized.
            let architecture_properties = $package
                .architecture_properties
                .entry(architecture)
                .or_insert(PackageArchitecture::default());

            // Set the architecture specific value.
            architecture_properties.$field_name =
                parse_clearable_value_array($keyword, $value, $parser)?;
        } else {
            $package.$field_name = parse_clearable_value_array($keyword, $value, $parser)?;
        }
    };
}

fn handle_package(
    package: &mut Package,
    values: HashMap<Keyword, ClearableValue>,
) -> Result<(), BridgeError> {
    for (raw_keyword, value) in values {
        // Parse the keyword
        let keyword = PackageKeywords::parser
            .parse(&raw_keyword.keyword)
            .map_err(|err| (raw_keyword.clone(), err))?;

        // Parse the architecture suffix if it exists.
        let architecture = match &raw_keyword.suffix {
            Some(suffix) => {
                let arch = Architecture::parser
                    .parse(suffix)
                    .map_err(|err| (raw_keyword.clone(), err))?;
                Some(arch)
            }
            None => None,
        };

        match keyword {
            PackageKeywords::Relation(keyword) => match keyword {
                RelationKeyword::Depends => package_value_array!(
                    &raw_keyword,
                    &value,
                    package,
                    dependencies,
                    architecture,
                    RelationOrSoname::parser,
                ),
                RelationKeyword::OptDepends => package_value_array!(
                    &raw_keyword,
                    &value,
                    package,
                    optional_dependencies,
                    architecture,
                    OptionalDependency::parser,
                ),
                RelationKeyword::Provides => package_value_array!(
                    &raw_keyword,
                    &value,
                    package,
                    provides,
                    architecture,
                    RelationOrSoname::parser,
                ),
                RelationKeyword::Conflicts => package_value_array!(
                    &raw_keyword,
                    &value,
                    package,
                    conflicts,
                    architecture,
                    PackageRelation::parser,
                ),
                RelationKeyword::Replaces => package_value_array!(
                    &raw_keyword,
                    &value,
                    package,
                    replaces,
                    architecture,
                    PackageRelation::parser,
                ),
            },
            PackageKeywords::SharedMeta(keyword) => match keyword {
                SharedMetaKeyword::PkgDesc => {
                    ensure_no_suffix(&raw_keyword, architecture)?;
                    package.description = parse_clearable_value(
                        &raw_keyword,
                        &value,
                        rest.try_map(PackageDescription::from_str),
                    )?;
                }
                SharedMetaKeyword::Url => {
                    ensure_no_suffix(&raw_keyword, architecture)?;
                    package.url =
                        parse_clearable_value(&raw_keyword, &value, rest.try_map(Url::from_str))?;
                }
                SharedMetaKeyword::License => {
                    ensure_no_suffix(&raw_keyword, architecture)?;
                    package.licenses = parse_clearable_value_array(
                        &raw_keyword,
                        &value,
                        rest.try_map(License::from_str),
                    )?;
                }
                SharedMetaKeyword::Arch => {
                    ensure_no_suffix(&raw_keyword, architecture)?;
                    let archs = parse_clearable_value_array(
                        &raw_keyword,
                        &value,
                        rest.try_map(Architecture::from_str),
                    )?;

                    // Architectures are a bit special as they **don't** allow to be cleared.
                    package.architectures = match archs {
                        Override::No => None,
                        Override::Clear => {
                            return Err(BridgeError::NoName);
                        }
                        Override::Yes { value } => {
                            let mut architectures = Vec::new();
                            // Make sure all architectures are unique.
                            for arch in value {
                                if architectures.contains(&arch) {
                                    return Err(BridgeError::LogicError {
                                        keyword: raw_keyword.clone(),
                                        message: format!("Found duplicate architecture {arch}"),
                                    });
                                }
                                architectures.push(arch);
                            }
                            Some(architectures)
                        }
                    };
                }
                SharedMetaKeyword::Changelog => {
                    ensure_no_suffix(&raw_keyword, architecture)?;
                    package.changelog = parse_clearable_value(
                        &raw_keyword,
                        &value,
                        rest.try_map(Changelog::from_str),
                    )?;
                }
                SharedMetaKeyword::Install => {
                    ensure_no_suffix(&raw_keyword, architecture)?;
                    package.install = parse_clearable_value(
                        &raw_keyword,
                        &value,
                        rest.try_map(Install::from_str),
                    )?;
                }
                SharedMetaKeyword::Groups => {
                    ensure_no_suffix(&raw_keyword, architecture)?;
                    package.groups = parse_clearable_value_array(
                        &raw_keyword,
                        &value,
                        rest.try_map(Group::from_str),
                    )?;
                }
                SharedMetaKeyword::Options => {
                    ensure_no_suffix(&raw_keyword, architecture)?;
                    package.options = parse_clearable_value_array(
                        &raw_keyword,
                        &value,
                        rest.try_map(MakepkgOption::from_str),
                    )?;
                }
                SharedMetaKeyword::Backup => {
                    ensure_no_suffix(&raw_keyword, architecture)?;
                    package.backups = parse_clearable_value_array(
                        &raw_keyword,
                        &value,
                        rest.try_map(Backup::from_str),
                    )?;
                }
            },
        }
    }

    Ok(())
}
